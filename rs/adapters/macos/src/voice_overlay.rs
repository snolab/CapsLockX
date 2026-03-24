//! Lightweight transparent floating overlay for voice input waveform.
//!
//! Uses a custom NSView subclass (`CLXWaveformView`) registered at runtime
//! with a `drawRect:` callback that reads waveform data from a global static
//! and draws via Core Graphics. No WKWebView — pure CG drawing.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

// ObjC exception catcher (compiled from objc_try.m).
extern "C" {
    fn objc_try_catch(fn_ptr: extern "C" fn(*mut c_void), context: *mut c_void) -> i32;
}

/// Wrap a closure for use inside dispatch callbacks.
/// Catches BOTH Rust panics AND ObjC exceptions (foreign exceptions).
fn catch_ffi_panic(_name: &str, f: impl FnOnce()) {
    extern "C" fn trampoline(ctx: *mut c_void) {
        let boxed: Box<Box<dyn FnOnce()>> = unsafe { Box::from_raw(ctx as *mut _) };
        boxed();
    }

    let boxed: Box<Box<dyn FnOnce()>> = Box::new(Box::new(f));
    let ctx = Box::into_raw(boxed) as *mut c_void;
    let result = unsafe { objc_try_catch(trampoline, ctx) };
    if result != 0 {
        eprintln!("[CLX] ObjC EXCEPTION in voice_overlay (caught, not crashing)");
    }
}

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
static HANDLE_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RESIZE_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

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

#[allow(dead_code)]
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

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSSize { w: f64, h: f64 }

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

// ── Position persistence ────────────────────────────────────────────────────

fn pos_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("voice_overlay_pos.json")
}

fn load_pos() -> Option<(f64, f64)> {
    let data = std::fs::read_to_string(pos_path()).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    let x = v.get("x")?.as_f64()?;
    let y = v.get("y")?.as_f64()?;
    Some((x, y))
}

fn save_pos(x: f64, y: f64) {
    let path = pos_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = format!("{{\"x\":{},\"y\":{}}}", x, y);
    let _ = std::fs::write(path, json);
}

// ── drawRect: callback ───────────────────────────────────────────────────────

static mut DRAW_RECT_THIS: *mut c_void = std::ptr::null_mut();
extern "C" fn draw_rect(this: *mut c_void, _cmd: *mut c_void, _dirty: NSRect) {
    unsafe { DRAW_RECT_THIS = this; }
    let r = unsafe { objc_try_catch(draw_rect_c, std::ptr::null_mut()) };
    if r != 0 { eprintln!("[CLX] ObjC exception in draw_rect (caught)"); }
}
extern "C" fn draw_rect_c(_: *mut c_void) {
    let this = unsafe { DRAW_RECT_THIS };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| draw_rect_inner(this)))
        .map_err(|_| eprintln!("[CLX] Rust panic in draw_rect (caught)"));
}
fn draw_rect_inner(this: *mut c_void) {
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
    let result = unsafe { objc_try_catch(show_main_inner_c, std::ptr::null_mut()) };
    if result != 0 {
        eprintln!("[CLX] ObjC exception in voice_overlay::show_main (caught, not crashing)");
    }
}

extern "C" fn show_main_inner_c(_: *mut c_void) {
    // catch_unwind for Rust panics (objc_try_catch handles ObjC exceptions).
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        show_main_inner();
    })).map_err(|_| eprintln!("[CLX] Rust panic in show_main_inner (caught)"));
}

fn show_main_inner() {
    unsafe {
        let existing = WINDOW_PTR.load(Ordering::Acquire);
        if !existing.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(existing, sel(b"orderFront:\0"), std::ptr::null_mut());
            let handle = HANDLE_PTR.load(Ordering::Acquire);
            if !handle.is_null() { f(handle, sel(b"orderFront:\0"), std::ptr::null_mut()); }
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
        // Hide from screen sharing / screenshots (NSWindowSharingNone = 0)
        f_u64(win, sel(b"setSharingType:\0"), 0);
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
            // Unlimited lines — overlay auto-resizes to content.
            f_i64(label, sel(b"setMaximumNumberOfLines:\0"), 0);
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

        // Create toolbar handle — horizontal bar above the overlay with ⠿ Move, ✕ Close.
        // Appears on hover, hidden by default. Dragging moves the overlay.
        let bar_h = 18.0_f64;
        let bar_rect = NSRect { x: rect.x, y: rect.y + oh, w: ow, h: bar_h };

        let handle_alloc = msg0(cls(b"NSPanel\0"), sel(b"alloc\0"));
        let handle_style: u64 = (1 << 4) | (1 << 7); // utility + nonActivating
        let handle: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(handle_alloc, sel(b"initWithContentRect:styleMask:backing:defer:\0"),
              bar_rect, handle_style, 2u64, false)
        };
        if !handle.is_null() {
            let f_bool: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
            f_bool(handle, sel(b"setOpaque:\0"), false);
            f_bool(handle, sel(b"setIgnoresMouseEvents:\0"), false);
            f_bool(handle, sel(b"setMovableByWindowBackground:\0"), true);
            f_bool(handle, sel(b"setHasShadow:\0"), false);
            f_bool(handle, sel(b"setHidesOnDeactivate:\0"), false);
            let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
            f_i64(handle, sel(b"setLevel:\0"), 3);
            let f_f64: extern "C" fn(*mut c_void, *mut c_void, f64) = std::mem::transmute(objc_msgSend as *const ());
            f_f64(handle, sel(b"setAlphaValue:\0"), 0.0); // hidden by default
            let f_u64: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
            f_u64(handle, sel(b"setSharingType:\0"), 0);
            f_u64(handle, sel(b"setCollectionBehavior:\0"), 1 | 16);

            let handle_bg = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.2, 0.2, 0.28, 0.85)
            };
            msg1_ptr(handle, sel(b"setBackgroundColor:\0"), handle_bg);

            let content_view = msg0(handle, sel(b"contentView\0"));
            let grip_color = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.65, 0.65, 0.75, 1.0)
            };
            let small_font = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 10.0)
            };
            let f_u64_2: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());

            // Helper to create a label in the bar.
            let make_label = |x: f64, w: f64, text: &str| -> *mut c_void {
                let lbl = msg0(cls(b"NSTextField\0"), sel(b"alloc\0"));
                let r = NSRect { x, y: 0.0, w, h: bar_h };
                let lbl: *mut c_void = {
                    let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(lbl, sel(b"initWithFrame:\0"), r)
                };
                msg1_ptr(lbl, sel(b"setStringValue:\0"), nsstring(text));
                f_bool(lbl, sel(b"setBezeled:\0"), false);
                f_bool(lbl, sel(b"setDrawsBackground:\0"), false);
                f_bool(lbl, sel(b"setEditable:\0"), false);
                f_bool(lbl, sel(b"setSelectable:\0"), false);
                // Let mouse events pass through to window background so dragging works.
                f_bool(lbl, sel(b"setIgnoresMouseEvents:\0"), true);
                msg1_ptr(lbl, sel(b"setTextColor:\0"), grip_color);
                msg1_ptr(lbl, sel(b"setFont:\0"), small_font);
                f_u64_2(lbl, sel(b"setAlignment:\0"), 1); // center
                msg1_ptr(content_view, sel(b"addSubview:\0"), lbl);
                lbl
            };

            // Layout: [⠿ Move] [model info] [✕]
            make_label(4.0, 50.0, "⠿ Move");

            // Show active models in center of toolbar.
            let stt_engine = std::env::var("CLX_STT_ENGINE").unwrap_or_else(|_| "SenseVoice".into());
            let has_gemini = std::env::var("GEMINI_API_KEY").is_ok() || {
                let p = dirs::config_dir().unwrap_or_default().join("CapsLockX").join("config.json");
                std::fs::read_to_string(&p).unwrap_or_default().contains("AIza")
            };
            let correction = if has_gemini { "+Gemini" } else { "" };
            let model_info = format!("{}{} | free local STT", stt_engine, correction);
            make_label(60.0, ow - 90.0, &model_info);
            // ✕ label is just visual — actual close button overlays it below.

            // Add a close button (actual NSButton for click handling).
            {
                // Register close action class once.
                static CLOSE_CLS_REGISTERED: std::sync::Once = std::sync::Once::new();
                CLOSE_CLS_REGISTERED.call_once(|| {
                    extern "C" {
                        fn objc_allocateClassPair(sup: *mut c_void, name: *const std::ffi::c_char, extra: usize) -> *mut c_void;
                        fn objc_registerClassPair(cls: *mut c_void);
                        fn class_addMethod(cls: *mut c_void, sel: *mut c_void, imp: *const c_void, types: *const std::ffi::c_char) -> bool;
                    }
                    unsafe extern "C" fn close_action(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
                        hide_overlay();
                    }
                    let sup = cls(b"NSObject\0");
                    let new_cls = objc_allocateClassPair(sup, b"CLXOverlayCloseAction\0".as_ptr() as *const _, 0);
                    if !new_cls.is_null() {
                        class_addMethod(new_cls, sel(b"closeOverlay:\0"), close_action as *const c_void, b"v@:@\0".as_ptr() as *const _);
                        objc_registerClassPair(new_cls);
                    }
                });

                let action_cls = cls(b"CLXOverlayCloseAction\0");
                if !action_cls.is_null() {
                    let target = msg0(msg0(action_cls, sel(b"alloc\0")), sel(b"init\0"));
                    msg0(target, sel(b"retain\0"));

                    let close_btn = msg0(cls(b"NSButton\0"), sel(b"alloc\0"));
                    let close_rect = NSRect { x: ow - 24.0, y: 0.0, w: 20.0, h: bar_h };
                    let close_btn: *mut c_void = {
                        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                            std::mem::transmute(objc_msgSend as *const ());
                        f(close_btn, sel(b"initWithFrame:\0"), close_rect)
                    };
                    msg1_ptr(close_btn, sel(b"setTitle:\0"), nsstring("✕"));
                    f_bool(close_btn, sel(b"setBordered:\0"), false);
                    msg1_ptr(close_btn, sel(b"setFont:\0"), small_font);
                    msg1_ptr(close_btn, sel(b"setTarget:\0"), target);
                    msg1_ptr(close_btn, sel(b"setAction:\0"), sel(b"closeOverlay:\0"));
                    // Remove the label ✕ we added, replace with clickable button.
                    msg1_ptr(content_view, sel(b"addSubview:\0"), close_btn);
                }
            }

            // Link toolbar as child window — moving the bar moves the overlay.
            let f_add_child: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, u64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_add_child(handle, sel(b"addChildWindow:ordered:\0"), win, 1); // NSWindowAbove

            // Create resize grip at bottom-right corner of overlay.
            // It's a separate tiny NSPanel that can be dragged. The hover_loop
            // tracks its position changes and resizes the overlay to match.
            let grip_size = 16.0_f64;
            let grip_rect = NSRect {
                x: rect.x + ow - grip_size,
                y: rect.y,
                w: grip_size,
                h: grip_size,
            };
            let resize_alloc = msg0(cls(b"NSPanel\0"), sel(b"alloc\0"));
            let resize_grip: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(resize_alloc, sel(b"initWithContentRect:styleMask:backing:defer:\0"),
                  grip_rect, handle_style, 2u64, false)
            };
            if !resize_grip.is_null() {
                f_bool(resize_grip, sel(b"setOpaque:\0"), false);
                f_bool(resize_grip, sel(b"setIgnoresMouseEvents:\0"), false);
                f_bool(resize_grip, sel(b"setMovableByWindowBackground:\0"), true);
                f_bool(resize_grip, sel(b"setHasShadow:\0"), false);
                f_bool(resize_grip, sel(b"setHidesOnDeactivate:\0"), false);
                f_i64(resize_grip, sel(b"setLevel:\0"), 3);
                f_f64(resize_grip, sel(b"setAlphaValue:\0"), 0.0);
                f_u64(resize_grip, sel(b"setSharingType:\0"), 0);
                f_u64(resize_grip, sel(b"setCollectionBehavior:\0"), 1 | 16);

                let grip_bg = {
                    let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
                        = std::mem::transmute(objc_msgSend as *const ());
                    f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.3, 0.3, 0.4, 0.8)
                };
                msg1_ptr(resize_grip, sel(b"setBackgroundColor:\0"), grip_bg);

                // ⇲ label
                let grip_lbl = msg0(cls(b"NSTextField\0"), sel(b"alloc\0"));
                let grip_lbl_rect = NSRect { x: 0.0, y: 0.0, w: grip_size, h: grip_size };
                let grip_lbl: *mut c_void = {
                    let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(grip_lbl, sel(b"initWithFrame:\0"), grip_lbl_rect)
                };
                msg1_ptr(grip_lbl, sel(b"setStringValue:\0"), nsstring("⇲"));
                f_bool(grip_lbl, sel(b"setBezeled:\0"), false);
                f_bool(grip_lbl, sel(b"setDrawsBackground:\0"), false);
                f_bool(grip_lbl, sel(b"setEditable:\0"), false);
                f_bool(grip_lbl, sel(b"setSelectable:\0"), false);
                msg1_ptr(grip_lbl, sel(b"setTextColor:\0"), grip_color);
                let grip_font = {
                    let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void
                        = std::mem::transmute(objc_msgSend as *const ());
                    f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 11.0)
                };
                msg1_ptr(grip_lbl, sel(b"setFont:\0"), grip_font);
                let rcv = msg0(resize_grip, sel(b"contentView\0"));
                msg1_ptr(rcv, sel(b"addSubview:\0"), grip_lbl);

                msg0(resize_grip, sel(b"retain\0"));
                f_show(resize_grip, sel(b"orderFront:\0"), std::ptr::null_mut());
                RESIZE_PTR.store(resize_grip, Ordering::Release);
            }

            msg0(handle, sel(b"retain\0"));
            f_show(handle, sel(b"orderFront:\0"), std::ptr::null_mut());
            HANDLE_PTR.store(handle, Ordering::Release);

            // Restore saved position if available.
            if let Some((sx, sy)) = load_pos() {
                #[repr(C)]
                #[derive(Clone, Copy)]
                struct NSPoint { x: f64, y: f64 }

                let set_origin: extern "C" fn(*mut c_void, *mut c_void, NSPoint) =
                    std::mem::transmute(objc_msgSend as *const ());
                // Move the handle (parent); child (win) follows automatically.
                let hf: NSRect = {
                    let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(handle, sel(b"frame\0"))
                };
                // Saved position is for the main overlay window.
                // Handle bar sits above the overlay.
                set_origin(handle, sel(b"setFrameOrigin:\0"),
                    NSPoint { x: sx, y: sy + hf.h });
            }

            // Spawn hover detection thread — shows/hides handle on mouse proximity.
            std::thread::Builder::new().name("clx-hover".into()).spawn(|| {
                hover_loop();
            }).ok();
        }
    }
}

/// Polls mouse position and shows/hides the drag handle when cursor is near overlay.
/// Also detects window movement and persists the new position.
fn hover_loop() {
    use core_graphics::event::{CGEvent, CGEventType};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let mut visible = false;
    let mut last_saved_x: f64 = f64::NAN;
    let mut last_saved_y: f64 = f64::NAN;
    let mut save_cooldown: u32 = 0; // ticks since position changed

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let win = WINDOW_PTR.load(Ordering::Acquire);
        let handle = HANDLE_PTR.load(Ordering::Acquire);
        if win.is_null() || handle.is_null() { continue; }

        // Get mouse position (screen coords, y=0 at bottom on macOS).
        let mouse = unsafe {
            if let Ok(event) = CGEvent::new(
                CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap()
            ) {
                let loc = event.location();
                (loc.x, loc.y)
            } else {
                continue;
            }
        };

        // Get overlay window frame.
        let win_frame: NSRect = unsafe {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                = std::mem::transmute(objc_msgSend as *const ());
            f(win, sel(b"frame\0"))
        };

        // Track position changes — save after position stabilizes for ~1s.
        if (win_frame.x - last_saved_x).abs() > 1.0 || (win_frame.y - last_saved_y).abs() > 1.0 {
            last_saved_x = win_frame.x;
            last_saved_y = win_frame.y;
            save_cooldown = 10; // wait 10 ticks (1 second) before saving
        }
        if save_cooldown > 0 {
            save_cooldown -= 1;
            if save_cooldown == 0 {
                save_pos(last_saved_x, last_saved_y);
            }
        }

        // Resize grip tracking disabled — calling setFrame from background thread
        // causes silent crashes. Resize is handled by the drag handle's child window
        // relationship (moving the toolbar bar repositions the overlay).
        // TODO: dispatch resize to main thread via dispatch_async_f.

        // Convert Quartz coords (y=0 at top) to AppKit (y=0 at bottom).
        // CGEvent uses Quartz; NSWindow uses AppKit. Screen height needed.
        let screen_h = unsafe {
            let scr = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                = std::mem::transmute(objc_msgSend as *const ());
            f(scr, sel(b"frame\0")).h
        };
        let mouse_appkit_y = screen_h - mouse.1;

        // Check if mouse is within overlay bounds (with 30px margin).
        let margin = 30.0;
        let near = mouse.0 >= win_frame.x - margin
            && mouse.0 <= win_frame.x + win_frame.w + margin
            && mouse_appkit_y >= win_frame.y - margin
            && mouse_appkit_y <= win_frame.y + win_frame.h + margin;

        if near && !visible {
            visible = true;
            unsafe {
                dispatch_async_f(main_queue(), std::ptr::null_mut(), show_handle);
            }
        } else if !near && visible {
            visible = false;
            unsafe {
                dispatch_async_f(main_queue(), std::ptr::null_mut(), hide_handle);
            }
        }
    }
}

extern "C" fn show_handle(_: *mut std::ffi::c_void) {
    unsafe {
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, f64)
            = std::mem::transmute(objc_msgSend as *const ());
        let handle = HANDLE_PTR.load(Ordering::Acquire);
        if !handle.is_null() { f(handle, sel(b"setAlphaValue:\0"), 0.7); }
        let resize = RESIZE_PTR.load(Ordering::Acquire);
        if !resize.is_null() { f(resize, sel(b"setAlphaValue:\0"), 0.7); }
    }
}

extern "C" fn hide_handle(_: *mut std::ffi::c_void) {
    unsafe {
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, f64)
            = std::mem::transmute(objc_msgSend as *const ());
        let handle = HANDLE_PTR.load(Ordering::Acquire);
        if !handle.is_null() { f(handle, sel(b"setAlphaValue:\0"), 0.0); }
        let resize = RESIZE_PTR.load(Ordering::Acquire);
        if !resize.is_null() { f(resize, sel(b"setAlphaValue:\0"), 0.0); }
    }
}

pub fn hide_overlay() {
    unsafe {
        let q = main_queue();
        if !q.is_null() { dispatch_async_f(q, std::ptr::null_mut(), hide_main); }
    }
}

extern "C" fn hide_main(_: *mut c_void) {
    let r = unsafe { objc_try_catch(hide_main_c, std::ptr::null_mut()) };
    if r != 0 { eprintln!("[CLX] ObjC exception in hide_main (caught)"); }
}
extern "C" fn hide_main_c(_: *mut c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hide_main_inner()))
        .map_err(|_| eprintln!("[CLX] Rust panic in hide_main (caught)"));
}
fn hide_main_inner() {
    unsafe {
        let win = WINDOW_PTR.load(Ordering::Acquire);
        if !win.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(win, sel(b"orderOut:\0"), std::ptr::null_mut());
        }
        let handle = HANDLE_PTR.load(Ordering::Acquire);
        if !handle.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(handle, sel(b"orderOut:\0"), std::ptr::null_mut());
        }
        let resize = RESIZE_PTR.load(Ordering::Acquire);
        if !resize.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
            f(resize, sel(b"orderOut:\0"), std::ptr::null_mut());
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

#[allow(dead_code)]
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
    let r = unsafe { objc_try_catch(trigger_redraw_c, std::ptr::null_mut()) };
    if r != 0 { eprintln!("[CLX] ObjC exception in trigger_redraw (caught)"); }
}
extern "C" fn trigger_redraw_c(_: *mut c_void) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| trigger_redraw_inner()))
        .map_err(|_| eprintln!("[CLX] Rust panic in trigger_redraw (caught)"));
}
fn trigger_redraw_inner() {
    unsafe {
        let view = VIEW_PTR.load(Ordering::Acquire);
        if !view.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
            f(view, sel(b"setNeedsDisplay:\0"), true);
        }
        // Update subtitle label.
        let label = LABEL_PTR.load(Ordering::Acquire);
        static DBG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dbg_n = DBG.fetch_add(1, Ordering::Relaxed);
        if dbg_n < 5 || dbg_n % 200 == 0 {
            eprintln!("[overlay] trigger_redraw #{}: label={:?}", dbg_n, !label.is_null());
        }
        if !label.is_null() {
            let text = {
                let g = WAVEFORM_DATA.lock().unwrap();
                if dbg_n < 5 || dbg_n % 200 == 0 {
                    let preview: String = g.subtitle.chars().take(60).collect();
                    eprintln!("[overlay] subtitle={:?}", preview);
                }
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

            // Auto-resize disabled — was causing silent crashes on some content.
            // The overlay uses a fixed height with scrollable text instead.
            // auto_resize_overlay(label);
        }
    }
}

/// Resize the overlay window to fit the label's text content.
unsafe fn auto_resize_overlay(label: *mut c_void) {
    let win = WINDOW_PTR.load(Ordering::Acquire);
    if win.is_null() { return; }

    // Get current window frame.
    let f_frame: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
        std::mem::transmute(objc_msgSend as *const ());
    let win_frame = f_frame(win, sel(b"frame\0"));

    // Measure label's preferred height for current width.
    let label_w = win_frame.w - 20.0; // 10px padding each side
    let fit_size: NSSize = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> NSSize =
            std::mem::transmute(objc_msgSend as *const ());
        // cellSizeForBounds: returns NSSize {width, height}.
        let constraint = NSRect { x: 0.0, y: 0.0, w: label_w, h: 10000.0 };
        let cell = msg0(label, sel(b"cell\0"));
        if cell.is_null() { return; }
        f(cell, sel(b"cellSizeForBounds:\0"), constraint)
    };

    let text_h = fit_size.h; // NSSize.h = height the text needs
    let waveform_h = 40.0_f64;
    let padding = 12.0_f64;
    let min_h = 80.0_f64;
    let max_h = 500.0_f64;

    let new_h = (text_h + waveform_h + padding).clamp(min_h, max_h);

    // Only resize if height changed significantly (avoid jitter).
    if (new_h - win_frame.h).abs() < 2.0 { return; }

    // Resize from top (keep top-left corner fixed in AppKit coords: adjust y).
    let new_y = win_frame.y + win_frame.h - new_h;
    let new_frame = NSRect { x: win_frame.x, y: new_y, w: win_frame.w, h: new_h };

    let f_set: extern "C" fn(*mut c_void, *mut c_void, NSRect, bool) =
        std::mem::transmute(objc_msgSend as *const ());
    f_set(win, sel(b"setFrame:display:\0"), new_frame, true);

    // Resize the container view and reposition label/waveform.
    let content = msg0(win, sel(b"contentView\0"));
    if !content.is_null() {
        let cv_rect = NSRect { x: 0.0, y: 0.0, w: win_frame.w, h: new_h };
        let f_sf: extern "C" fn(*mut c_void, *mut c_void, NSRect) =
            std::mem::transmute(objc_msgSend as *const ());
        f_sf(content, sel(b"setFrame:\0"), cv_rect);

        // Reposition waveform at top.
        let view = VIEW_PTR.load(Ordering::Acquire);
        if !view.is_null() {
            let wf_rect = NSRect { x: 0.0, y: new_h - waveform_h, w: win_frame.w, h: waveform_h };
            f_sf(view, sel(b"setFrame:\0"), wf_rect);
        }

        // Reposition label below waveform.
        let label_rect = NSRect { x: 10.0, y: 4.0, w: win_frame.w - 20.0, h: new_h - waveform_h - 8.0 };
        f_sf(label, sel(b"setFrame:\0"), label_rect);
    }
}
