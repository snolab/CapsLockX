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
    mic_levels: Vec<f32>,
    sys_levels: Vec<f32>,
    mic_vad: bool,
    sys_vad: bool,
    subtitle: String,
}

static WAVEFORM_DATA: Mutex<WaveformData> = Mutex::new(WaveformData {
    mic_levels: Vec::new(),
    sys_levels: Vec::new(),
    mic_vad: false,
    sys_vad: false,
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
        let (mic_levels, sys_levels, mic_vad, sys_vad) = {
            let g = WAVEFORM_DATA.lock().unwrap();
            (g.mic_levels.clone(), g.sys_levels.clone(), g.mic_vad, g.sys_vad)
        };

        CGContextSaveGState(cg);

        let mx = 8.0;
        let uw = w - 2.0 * mx;

        // Draw a waveform helper
        fn draw_wave(cg: *mut c_void, levels: &[f32], mid_y: f64, max_amp: f64, mx: f64, uw: f64, w: f64) {
            unsafe {
                if levels.is_empty() {
                    CGContextMoveToPoint(cg, mx, mid_y);
                    CGContextAddLineToPoint(cg, w - mx, mid_y);
                    CGContextStrokePath(cg);
                } else {
                    let n = levels.len();
                    let step = if n > 1 { uw / (n - 1) as f64 } else { uw };
                    CGContextMoveToPoint(cg, mx, mid_y);
                    for (i, &l) in levels.iter().enumerate() {
                        let x = mx + i as f64 * step;
                        CGContextAddLineToPoint(cg, x, mid_y + l.clamp(0.0, 1.0) as f64 * max_amp);
                    }
                    CGContextStrokePath(cg);
                    CGContextMoveToPoint(cg, mx, mid_y);
                    for (i, &l) in levels.iter().enumerate() {
                        let x = mx + i as f64 * step;
                        CGContextAddLineToPoint(cg, x, mid_y - l.clamp(0.0, 1.0) as f64 * max_amp);
                    }
                    CGContextStrokePath(cg);
                }
            }
        }

        // Mic waveform — top half, green
        let mic_mid = h * 0.75;
        let amp = h / 4.0 - 2.0;
        if mic_vad {
            CGContextSetRGBStrokeColor(cg, 0.29, 0.87, 0.5, 0.9); // green
        } else {
            CGContextSetRGBStrokeColor(cg, 0.42, 0.44, 0.49, 0.5);
        }
        CGContextSetLineWidth(cg, 1.5);
        CGContextSetLineCap(cg, 1);
        draw_wave(cg, &mic_levels, mic_mid, amp, mx, uw, w);

        // System waveform — bottom half, blue
        let sys_mid = h * 0.25;
        if sys_vad {
            CGContextSetRGBStrokeColor(cg, 0.3, 0.5, 0.95, 0.9); // blue
        } else {
            CGContextSetRGBStrokeColor(cg, 0.42, 0.44, 0.49, 0.3);
        }
        draw_wave(cg, &sys_levels, sys_mid, amp, mx, uw, w);

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

        // Waveform view: dual waveform strip at top (40px)
        let wf_rect = NSRect { x: 0.0, y: oh - 40.0, w: ow, h: 40.0 };
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
            // Non-editable label, NO background (we use attributed string instead).
            f_bool(label, sel(b"setBezeled:\0"), false);
            f_bool(label, sel(b"setDrawsBackground:\0"), false);
            f_bool(label, sel(b"setEditable:\0"), false);
            f_bool(label, sel(b"setSelectable:\0"), false);
            // Align center.
            let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
            f_i64(label, sel(b"setAlignment:\0"), 1); // NSTextAlignmentCenter
            // Allow up to 3 lines with word wrapping.
            f_i64(label, sel(b"setMaximumNumberOfLines:\0"), 3);
            let cell = msg0(label, sel(b"cell\0"));
            if !cell.is_null() {
                f_i64(cell, sel(b"setLineBreakMode:\0"), 0); // word wrap
                f_bool(cell, sel(b"setWraps:\0"), true);
            }
            // Set initial attributed text with per-character background.
            set_attributed_subtitle(label, "🎤 Listening...");

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
            g.mic_levels.clear();
            g.sys_levels.clear();
            g.mic_vad = false;
            g.sys_vad = false;
        }
    }
}

pub fn push_audio_levels(levels: &[f32], vad_active: bool) {
    push_dual_audio_levels(levels, vad_active, &[], false, None);
}

pub fn push_audio_levels_with_text(levels: &[f32], vad_active: bool, subtitle: Option<&str>) {
    push_dual_audio_levels(levels, vad_active, &[], false, subtitle);
}

pub fn push_dual_audio_levels(mic_levels: &[f32], mic_vad: bool, sys_levels: &[f32], sys_vad: bool, subtitle: Option<&str>) {
    {
        let mut g = WAVEFORM_DATA.lock().unwrap();
        if !mic_levels.is_empty() {
            g.mic_levels.extend_from_slice(mic_levels);
            if g.mic_levels.len() > 100 { let e = g.mic_levels.len() - 100; g.mic_levels.drain(..e); }
            g.mic_vad = mic_vad;
        }
        if !sys_levels.is_empty() {
            g.sys_levels.extend_from_slice(sys_levels);
            if g.sys_levels.len() > 100 { let e = g.sys_levels.len() - 100; g.sys_levels.drain(..e); }
            g.sys_vad = sys_vad;
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

/// Set an NSAttributedString on the label with per-character dark background.
/// This gives true subtitle-style rendering — background only behind text, not the whole frame.
/// Set an NSAttributedString on the label with per-character dark background.
/// Create an NSColor from RGBA.
unsafe fn nscolor(r: f64, g: f64, b: f64, a: f64) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), r, g, b, a)
}

/// Build an attributes dictionary with given bg color.
unsafe fn make_attrs(bg: *mut c_void) -> *mut c_void {
    let f2: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let dict = msg0(msg0(cls(b"NSMutableDictionary\0"), sel(b"alloc\0")), sel(b"init\0"));
    f2(dict, sel(b"setObject:forKey:\0"), msg0(cls(b"NSColor\0"), sel(b"whiteColor\0")), nsstring("NSColor"));
    f2(dict, sel(b"setObject:forKey:\0"), bg, nsstring("NSBackgroundColor"));
    let font: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 14.0_f64)
    };
    f2(dict, sel(b"setObject:forKey:\0"), font, nsstring("NSFont"));
    let para = msg0(msg0(cls(b"NSMutableParagraphStyle\0"), sel(b"alloc\0")), sel(b"init\0"));
    let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
    f_i64(para, sel(b"setAlignment:\0"), 1);
    f2(dict, sel(b"setObject:forKey:\0"), para, nsstring("NSParagraphStyle"));
    dict
}

unsafe fn set_attributed_subtitle(label: *mut c_void, text: &str) {
    let pool = msg0(msg0(cls(b"NSAutoreleasePool\0"), sel(b"alloc\0")), sel(b"init\0"));

    // Colors: green bg for [Me], blue bg for [Other], dark bg for default.
    let bg_me = nscolor(0.1, 0.35, 0.15, 0.8);     // dark green
    let bg_other = nscolor(0.1, 0.2, 0.4, 0.8);     // dark blue
    let bg_default = nscolor(0.0, 0.0, 0.0, 0.7);   // dark

    // Build NSMutableAttributedString by parsing [Me]/🔊 prefixes.
    let mut_attr_cls = cls(b"NSMutableAttributedString\0");
    let result = msg0(msg0(mut_attr_cls, sel(b"alloc\0")), sel(b"init\0"));

    // Split text into segments by newlines or [Me]/🔊 tags.
    // Process each line separately, then join with newline.
    let lines: Vec<&str> = text.split('\n').collect();
    for (li, line) in lines.iter().enumerate() {
        // Determine background color from line prefix.
        let (content, bg) = if line.starts_with("🎤 ") {
            (*line, bg_me)
        } else if line.starts_with("🔊 ") {
            (*line, bg_other)
        } else {
            (*line, bg_default)
        };

        if !content.is_empty() {
            let attrs = make_attrs(bg);
            let ns_seg = nsstring(content);
            let attr_seg: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(msg0(cls(b"NSAttributedString\0"), sel(b"alloc\0")),
                  sel(b"initWithString:attributes:\0"), ns_seg, attrs)
            };
            msg1_ptr(result, sel(b"appendAttributedString:\0"), attr_seg);
        }

        // Add newline between lines (not after the last one).
        if li < lines.len() - 1 {
            let nl = nsstring("\n");
            let nl_attrs = make_attrs(nscolor(0.0, 0.0, 0.0, 0.0)); // transparent bg for newline
            let nl_seg: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(msg0(cls(b"NSAttributedString\0"), sel(b"alloc\0")),
                  sel(b"initWithString:attributes:\0"), nl, nl_attrs)
            };
            msg1_ptr(result, sel(b"appendAttributedString:\0"), nl_seg);
        }
    }

    msg1_ptr(label, sel(b"setAttributedStringValue:\0"), result);
    msg0(pool, sel(b"drain\0"));
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
                    match (g.mic_vad, g.sys_vad) {
                        (true, true) => "🎤 Speaking... 🔊 Playing...".to_string(),
                        (true, false) => "🎤 Speaking...".to_string(),
                        (false, true) => "🔊 Playing...".to_string(),
                        (false, false) => "🎤 Listening...".to_string(),
                    }
                } else {
                    // Truncate each line, preserving emoji prefix (🎤/🔊).
                    g.subtitle.lines().map(|line| {
                        let chars: Vec<char> = line.chars().collect();
                        if chars.len() > 80 {
                            // Keep first 2 chars (emoji + space), then "..." + last N chars.
                            let prefix: String = chars[..2.min(chars.len())].iter().collect();
                            let tail: String = chars[chars.len()-74..].iter().collect();
                            format!("{}...{}", prefix, tail)
                        } else {
                            line.to_string()
                        }
                    }).collect::<Vec<_>>().join("\n")
                }
            };
            set_attributed_subtitle(label, &text);
        }
    }
}
