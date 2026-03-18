//! Lightweight transparent floating overlay for voice input waveform.
//!
//! Uses a custom NSView subclass (`CLXWaveformView`) registered at runtime
//! with a `drawRect:` callback that reads waveform data from a global static
//! and draws via Core Graphics. No WKWebView — pure CG drawing.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

// ── Global shared waveform state ─────────────────────────────────────────────

struct WaveformData {
    levels: Vec<f32>,
    vad_active: bool,
    subtitle: String,
}

static WAVEFORM_DATA: Mutex<WaveformData> = Mutex::new(WaveformData {
    levels: Vec::new(),
    vad_active: false,
    subtitle: String::new(),
});

static VIEW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static LABEL_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// ── ObjC runtime FFI ─────────────────────────────────────────────────────────

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    fn objc_allocateClassPair(sup: *mut c_void, name: *const std::ffi::c_char, extra: usize) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(cls: *mut c_void, sel: *mut c_void, imp: *const c_void, types: *const std::ffi::c_char) -> bool;
    fn dispatch_async_f(queue: *mut c_void, ctx: *mut c_void, work: extern "C" fn(*mut c_void));
    fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
}

extern "C" {
    fn CGContextSaveGState(ctx: *mut c_void);
    fn CGContextRestoreGState(ctx: *mut c_void);
    fn CGContextSetRGBFillColor(ctx: *mut c_void, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSetRGBStrokeColor(ctx: *mut c_void, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSetLineWidth(ctx: *mut c_void, w: f64);
    fn CGContextMoveToPoint(ctx: *mut c_void, x: f64, y: f64);
    fn CGContextAddLineToPoint(ctx: *mut c_void, x: f64, y: f64);
    fn CGContextStrokePath(ctx: *mut c_void);
    fn CGContextSetLineCap(ctx: *mut c_void, cap: i32);
    fn CGContextAddArc(ctx: *mut c_void, x: f64, y: f64, r: f64, sa: f64, ea: f64, cw: i32);
    fn CGContextFillPath(ctx: *mut c_void);
    fn CGContextBeginPath(ctx: *mut c_void);
    fn CGContextClosePath(ctx: *mut c_void);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect { x: f64, y: f64, w: f64, h: f64 }

const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }
unsafe fn msg0(obj: *mut c_void, s: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s)
}
unsafe fn msg1_ptr(obj: *mut c_void, s: *mut c_void, a: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s, a)
}
unsafe fn main_queue() -> *mut c_void {
    dlsym(RTLD_DEFAULT, b"_dispatch_main_q\0".as_ptr() as *const _)
}
unsafe fn nsstring(s: &str) -> *mut c_void {
    let cstr = std::ffi::CString::new(s).unwrap();
    let f: extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls(b"NSString\0"), sel(b"stringWithUTF8String:\0"), cstr.as_ptr())
}

// ── drawRect: callback ───────────────────────────────────────────────────────

extern "C" fn draw_rect(this: *mut c_void, _cmd: *mut c_void, _dirty: NSRect) {
    unsafe {
        let ns_gfx = cls(b"NSGraphicsContext\0");
        let ctx_obj = msg0(ns_gfx, sel(b"currentContext\0"));
        if ctx_obj.is_null() { return; }
        let cg = msg0(ctx_obj, sel(b"CGContext\0"));
        if cg.is_null() { return; }

        // Get bounds
        #[cfg(target_arch = "aarch64")]
        let bounds: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                std::mem::transmute(objc_msgSend as *const ());
            f(this, sel(b"bounds\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let bounds = NSRect { x: 0.0, y: 0.0, w: 400.0, h: 80.0 };

        let (w, h) = (bounds.w, bounds.h);
        let (levels, vad) = {
            let g = WAVEFORM_DATA.lock().unwrap();
            (g.levels.clone(), g.vad_active)
        };

        CGContextSaveGState(cg);

        // No background here — the window itself has the semi-transparent bg.

        // Waveform
        let mid = h / 2.0;
        let mx = 16.0;
        let uw = w - 2.0 * mx;
        let max_amp = mid - 8.0;

        if vad {
            CGContextSetRGBStrokeColor(cg, 0.29, 0.87, 0.5, 0.9); // green
        } else {
            CGContextSetRGBStrokeColor(cg, 0.42, 0.44, 0.49, 0.7); // gray
        }
        CGContextSetLineWidth(cg, 2.0);
        CGContextSetLineCap(cg, 1); // round

        if levels.is_empty() {
            CGContextMoveToPoint(cg, mx, mid);
            CGContextAddLineToPoint(cg, w - mx, mid);
            CGContextStrokePath(cg);
        } else {
            let n = levels.len();
            let step = if n > 1 { uw / (n - 1) as f64 } else { uw };

            // Upper half
            CGContextMoveToPoint(cg, mx, mid);
            for (i, &l) in levels.iter().enumerate() {
                let x = mx + i as f64 * step;
                CGContextAddLineToPoint(cg, x, mid + l.clamp(0.0, 1.0) as f64 * max_amp);
            }
            CGContextStrokePath(cg);

            // Lower half (mirror)
            CGContextMoveToPoint(cg, mx, mid);
            for (i, &l) in levels.iter().enumerate() {
                let x = mx + i as f64 * step;
                CGContextAddLineToPoint(cg, x, mid - l.clamp(0.0, 1.0) as f64 * max_amp);
            }
            CGContextStrokePath(cg);
        }

        CGContextRestoreGState(cg);
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

pub fn init_overlay() {
    unsafe {
        let sup = cls(b"NSView\0");
        if sup.is_null() { return; }
        let new_cls = objc_allocateClassPair(sup, b"CLXWaveformView\0".as_ptr() as *const _, 0);
        if new_cls.is_null() { return; } // already registered
        let types = b"v@:{CGRect={CGPoint=dd}{CGSize=dd}}\0";
        class_addMethod(new_cls, sel(b"drawRect:\0"), draw_rect as *const c_void, types.as_ptr() as *const _);
        objc_registerClassPair(new_cls);
    }
}

pub fn show_overlay() {
    unsafe {
        let q = main_queue();
        if !q.is_null() { dispatch_async_f(q, std::ptr::null_mut(), show_main); }
    }
}

extern "C" fn show_main(_: *mut c_void) {
    unsafe {
        let existing = WINDOW_PTR.load(Ordering::Acquire);
        if !existing.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(existing, sel(b"orderFront:\0"), std::ptr::null_mut());
            return;
        }

        // Get screen size
        let scr = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
        if scr.is_null() { return; }
        #[cfg(target_arch = "aarch64")]
        let sf: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect = std::mem::transmute(objc_msgSend as *const ());
            f(scr, sel(b"frame\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let sf = NSRect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };

        let ow = 600.0_f64;
        let oh = 100.0_f64; // thin waveform (20px) + 3 lines of text (~70px) + padding
        // Position at top-center (AppKit coords: y=0 is bottom, so top = screen_h - oh - margin)
        let rect = NSRect { x: (sf.w - ow) / 2.0, y: sf.h - oh - 40.0, w: ow, h: oh };

        // Create window
        let alloc = msg0(cls(b"NSWindow\0"), sel(b"alloc\0"));
        let win: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(alloc, sel(b"initWithContentRect:styleMask:backing:defer:\0"), rect, 0u64, 2u64, false)
        };
        if win.is_null() { return; }

        // Configure — fully transparent window, text has its own background.
        let f_bool: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
        f_bool(win, sel(b"setOpaque:\0"), false);
        msg1_ptr(win, sel(b"setBackgroundColor:\0"), msg0(cls(b"NSColor\0"), sel(b"clearColor\0")));
        let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
        f_i64(win, sel(b"setLevel:\0"), 3); // floating
        f_bool(win, sel(b"setIgnoresMouseEvents:\0"), true);
        f_bool(win, sel(b"setHasShadow:\0"), false);
        let f_u64: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
        f_u64(win, sel(b"setCollectionBehavior:\0"), 1 | 16); // allSpaces + stationary

        // Create view
        let view_cls = cls(b"CLXWaveformView\0");
        if view_cls.is_null() { return; }
        let view_alloc = msg0(view_cls, sel(b"alloc\0"));
        let vr = NSRect { x: 0.0, y: 0.0, w: ow, h: oh };
        let view: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(view_alloc, sel(b"initWithFrame:\0"), vr)
        };
        if view.is_null() { return; }

        // Use an NSView container so we can add both waveform and label.
        let container_cls = cls(b"NSView\0");
        let container_alloc = msg0(container_cls, sel(b"alloc\0"));
        let container: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(container_alloc, sel(b"initWithFrame:\0"), vr)
        };

        // Waveform view: thin strip at top (20px)
        let wf_rect = NSRect { x: 0.0, y: oh - 20.0, w: ow, h: 20.0 };
        let wf_view: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(view_alloc, sel(b"initWithFrame:\0"), wf_rect)
        };
        msg1_ptr(container, sel(b"addSubview:\0"), wf_view);

        // Subtitle label: bottom 65px (3 lines)
        let label_cls = cls(b"NSTextField\0");
        let label_alloc = msg0(label_cls, sel(b"alloc\0"));
        let label_rect = NSRect { x: 10.0, y: 4.0, w: ow - 20.0, h: oh - 28.0 };
        let label: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(label_alloc, sel(b"initWithFrame:\0"), label_rect)
        };
        if !label.is_null() {
            let f_bool: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
            // Non-editable label with dark semi-transparent background.
            f_bool(label, sel(b"setBezeled:\0"), false);
            f_bool(label, sel(b"setDrawsBackground:\0"), true);
            f_bool(label, sel(b"setEditable:\0"), false);
            f_bool(label, sel(b"setSelectable:\0"), false);
            // Dark background on the text itself (subtitle style).
            let label_bg: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.0, 0.0, 0.0, 0.7)
            };
            msg1_ptr(label, sel(b"setBackgroundColor:\0"), label_bg);
            // Rounded corners on the label via its layer.
            f_bool(label, sel(b"setWantsLayer:\0"), true);
            let label_layer = msg0(label, sel(b"layer\0"));
            if !label_layer.is_null() {
                let f_f64: extern "C" fn(*mut c_void, *mut c_void, f64) = std::mem::transmute(objc_msgSend as *const ());
                f_f64(label_layer, sel(b"setCornerRadius:\0"), 8.0);
                f_bool(label_layer, sel(b"setMasksToBounds:\0"), true);
            }
            // White text.
            let white = msg0(cls(b"NSColor\0"), sel(b"whiteColor\0"));
            msg1_ptr(label, sel(b"setTextColor:\0"), white);
            // Set font size.
            let font_cls = cls(b"NSFont\0");
            let font: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(font_cls, sel(b"systemFontOfSize:\0"), 13.0_f64)
            };
            msg1_ptr(label, sel(b"setFont:\0"), font);
            // Align center.
            let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
            f_i64(label, sel(b"setAlignment:\0"), 1); // NSTextAlignmentCenter = 1
            // Allow up to 3 lines with word wrapping.
            f_i64(label, sel(b"setMaximumNumberOfLines:\0"), 3);
            // NSLineBreakByWordWrapping = 0
            let cell = msg0(label, sel(b"cell\0"));
            if !cell.is_null() {
                f_i64(cell, sel(b"setLineBreakMode:\0"), 0); // word wrap
                f_bool(cell, sel(b"setWraps:\0"), true);
            }
            // Set initial text.
            let initial = nsstring("🎤 Listening...");
            msg1_ptr(label, sel(b"setStringValue:\0"), initial);

            msg1_ptr(container, sel(b"addSubview:\0"), label);
            LABEL_PTR.store(label, Ordering::Release);
        }

        msg1_ptr(win, sel(b"setContentView:\0"), container);
        let f_show: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
        f_show(win, sel(b"orderFront:\0"), std::ptr::null_mut());

        VIEW_PTR.store(wf_view, Ordering::Release);
        WINDOW_PTR.store(win, Ordering::Release);
    }
}

pub fn hide_overlay() {
    unsafe {
        let q = main_queue();
        if !q.is_null() { dispatch_async_f(q, std::ptr::null_mut(), hide_main); }
    }
}

extern "C" fn hide_main(_: *mut c_void) {
    unsafe {
        let win = WINDOW_PTR.load(Ordering::Acquire);
        if !win.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(win, sel(b"orderOut:\0"), std::ptr::null_mut());
        }
        // Clear waveform data
        if let Ok(mut g) = WAVEFORM_DATA.lock() {
            g.levels.clear();
            g.vad_active = false;
        }
    }
}

pub fn push_audio_levels(levels: &[f32], vad_active: bool) {
    push_audio_levels_with_text(levels, vad_active, None);
}

pub fn push_audio_levels_with_text(levels: &[f32], vad_active: bool, subtitle: Option<&str>) {
    {
        let mut g = WAVEFORM_DATA.lock().unwrap();
        if !levels.is_empty() {
            g.levels.extend_from_slice(levels);
            if g.levels.len() > 100 {
                let excess = g.levels.len() - 100;
                g.levels.drain(..excess);
            }
            g.vad_active = vad_active;
        }
        if let Some(text) = subtitle {
            g.subtitle = text.to_string();
        }
    }
    unsafe {
        let q = main_queue();
        if !q.is_null() { dispatch_async_f(q, std::ptr::null_mut(), trigger_redraw); }
    }
}

extern "C" fn trigger_redraw(_: *mut c_void) {
    unsafe {
        let view = VIEW_PTR.load(Ordering::Acquire);
        if !view.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
            f(view, sel(b"setNeedsDisplay:\0"), true);
        }
        // Update subtitle label.
        let label = LABEL_PTR.load(Ordering::Acquire);
        if !label.is_null() {
            let text = {
                let g = WAVEFORM_DATA.lock().unwrap();
                if g.subtitle.is_empty() {
                    if g.vad_active { "🎤 Speaking...".to_string() } else { "🎤 Listening...".to_string() }
                } else {
                    // Show last ~150 chars (fits ~3 lines at 50 chars/line).
                    let s = &g.subtitle;
                    if s.chars().count() > 150 {
                        format!("...{}", s.chars().rev().take(147).collect::<Vec<_>>().into_iter().rev().collect::<String>())
                    } else {
                        s.clone()
                    }
                }
            };
            let ns = nsstring(&text);
            msg1_ptr(label, sel(b"setStringValue:\0"), ns);
            // Resize to fit text, then position: centered horizontally, anchored below waveform.
            msg0(label, sel(b"sizeToFit\0"));
            #[cfg(target_arch = "aarch64")]
            {
                let label_frame: NSRect = {
                    let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(label, sel(b"frame\0"))
                };
                let win = WINDOW_PTR.load(Ordering::Acquire);
                if !win.is_null() {
                    let cont = msg0(win, sel(b"contentView\0"));
                    let cont_frame: NSRect = {
                        let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                            std::mem::transmute(objc_msgSend as *const ());
                        f(cont, sel(b"frame\0"))
                    };
                    // Anchor to top (just below waveform, which is top 20px).
                    // AppKit: y=0 is bottom, so top = cont_h - waveform_h - label_h
                    let new_x = (cont_frame.w - label_frame.w) / 2.0;
                    let new_y = cont_frame.h - 20.0 - label_frame.h - 4.0; // 4px gap below waveform
                    let new_frame = NSRect { x: new_x.max(4.0), y: new_y, w: label_frame.w, h: label_frame.h };
                    let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(label, sel(b"setFrame:\0"), new_frame);
                }
            }
        }
    }
}
