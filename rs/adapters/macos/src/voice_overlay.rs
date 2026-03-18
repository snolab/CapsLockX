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
}

static WAVEFORM_DATA: Mutex<WaveformData> = Mutex::new(WaveformData {
    levels: Vec::new(),
    vad_active: false,
});

static VIEW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

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

        // Background: rounded rect
        let rad = 12.0_f64;
        CGContextSetRGBFillColor(cg, 0.0, 0.0, 0.0, 0.6);
        CGContextBeginPath(cg);
        CGContextAddArc(cg, rad, rad, rad, std::f64::consts::PI, std::f64::consts::PI * 1.5, 0);
        CGContextAddArc(cg, w - rad, rad, rad, std::f64::consts::PI * 1.5, 0.0, 0);
        CGContextAddArc(cg, w - rad, h - rad, rad, 0.0, std::f64::consts::FRAC_PI_2, 0);
        CGContextAddArc(cg, rad, h - rad, rad, std::f64::consts::FRAC_PI_2, std::f64::consts::PI, 0);
        CGContextClosePath(cg);
        CGContextFillPath(cg);

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

        let ow = 400.0_f64;
        let oh = 80.0_f64;
        let rect = NSRect { x: (sf.w - ow) / 2.0, y: 60.0, w: ow, h: oh };

        // Create window
        let alloc = msg0(cls(b"NSWindow\0"), sel(b"alloc\0"));
        let win: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(alloc, sel(b"initWithContentRect:styleMask:backing:defer:\0"), rect, 0u64, 2u64, false)
        };
        if win.is_null() { return; }

        // Configure
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

        msg1_ptr(win, sel(b"setContentView:\0"), view);
        let f_show: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(objc_msgSend as *const ());
        f_show(win, sel(b"orderFront:\0"), std::ptr::null_mut());

        VIEW_PTR.store(view, Ordering::Release);
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
    {
        let mut g = WAVEFORM_DATA.lock().unwrap();
        g.levels.extend_from_slice(levels);
        if g.levels.len() > 100 {
            let excess = g.levels.len() - 100;
            g.levels.drain(..excess);
        }
        g.vad_active = vad_active;
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
    }
}
