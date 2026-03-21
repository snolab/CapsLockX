/// Standalone voice transcription test binary.
/// Uses the exact same overlay as CLX Space+V: dual waveform (mic=green, sys=blue),
/// attributed subtitle, drag handle, screen-share-hidden window.
///
/// Usage:
///   cargo run -p capslockx-core --release --bin voice-test
///
/// Safe to run alongside CapsLockX — does NOT touch keyboard hooks.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use capslockx_core::audio_capture::AudioCapture;
use capslockx_core::local_whisper::LocalWhisper;

const STREAMING_CHUNK_SAMPLES: usize = 1_600; // 100ms at 16kHz
const TEN_VAD_FRAME_SIZE: usize = 256;
const SPEECH_START_PROB: f32 = 0.5;
const SPEECH_END_PROB: f32 = 0.3;
const SPEECH_START_FRAMES: usize = 2;
const SILENCE_END_FRAMES: usize = 15;

// ── Overlay: shared waveform state (identical to CLX voice_overlay) ───────────

struct WaveformData {
    mic_levels: Vec<f32>,
    sys_levels: Vec<f32>,
    mic_vad:    bool,
    sys_vad:    bool,
    subtitle:   String,
}

static WAVEFORM_DATA: Mutex<WaveformData> = Mutex::new(WaveformData {
    mic_levels: Vec::new(),
    sys_levels: Vec::new(),
    mic_vad:    false,
    sys_vad:    false,
    subtitle:   String::new(),
});

static VIEW_PTR:   AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static LABEL_PTR:  AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static HANDLE_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// ── ObjC / CoreGraphics FFI ───────────────────────────────────────────────────

#[link(name = "AppKit", kind = "framework")]
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

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextSaveGState(ctx: *mut c_void);
    fn CGContextRestoreGState(ctx: *mut c_void);
    fn CGContextSetRGBStrokeColor(ctx: *mut c_void, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSetLineWidth(ctx: *mut c_void, w: f64);
    fn CGContextMoveToPoint(ctx: *mut c_void, x: f64, y: f64);
    fn CGContextAddLineToPoint(ctx: *mut c_void, x: f64, y: f64);
    fn CGContextStrokePath(ctx: *mut c_void);
    fn CGContextSetLineCap(ctx: *mut c_void, cap: i32);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect { x: f64, y: f64, w: f64, h: f64 }

const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }
unsafe fn msg0(obj: *mut c_void, s: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(obj, s)
}
unsafe fn msg1_ptr(obj: *mut c_void, s: *mut c_void, a: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
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

// ── drawRect: (dual waveform — top=mic green, bottom=sys blue) ────────────────

extern "C" fn draw_rect(this: *mut c_void, _cmd: *mut c_void, _dirty: NSRect) {
    unsafe {
        let ctx_obj = msg0(cls(b"NSGraphicsContext\0"), sel(b"currentContext\0"));
        if ctx_obj.is_null() { return; }
        let cg = msg0(ctx_obj, sel(b"CGContext\0"));
        if cg.is_null() { return; }

        #[cfg(target_arch = "aarch64")]
        let bounds: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                std::mem::transmute(objc_msgSend as *const ());
            f(this, sel(b"bounds\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let bounds = NSRect { x: 0.0, y: 0.0, w: 600.0, h: 40.0 };

        let (w, h) = (bounds.w, bounds.h);
        let (mic_levels, sys_levels, mic_vad, sys_vad) = {
            let g = WAVEFORM_DATA.lock().unwrap();
            (g.mic_levels.clone(), g.sys_levels.clone(), g.mic_vad, g.sys_vad)
        };

        CGContextSaveGState(cg);

        let mx  = 8.0_f64;
        let uw  = w - 2.0 * mx;
        let amp = h / 4.0 - 2.0;

        fn draw_wave(cg: *mut c_void, levels: &[f32], mid_y: f64, max_amp: f64,
                     mx: f64, uw: f64, w: f64) {
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
                        CGContextAddLineToPoint(cg, mx + i as f64 * step,
                            mid_y + l.clamp(0.0, 1.0) as f64 * max_amp);
                    }
                    CGContextStrokePath(cg);
                    CGContextMoveToPoint(cg, mx, mid_y);
                    for (i, &l) in levels.iter().enumerate() {
                        CGContextAddLineToPoint(cg, mx + i as f64 * step,
                            mid_y - l.clamp(0.0, 1.0) as f64 * max_amp);
                    }
                    CGContextStrokePath(cg);
                }
            }
        }

        CGContextSetLineWidth(cg, 1.5);
        CGContextSetLineCap(cg, 1); // round

        // Mic — top half, green when active
        if mic_vad { CGContextSetRGBStrokeColor(cg, 0.29, 0.87, 0.5, 0.9); }
        else        { CGContextSetRGBStrokeColor(cg, 0.42, 0.44, 0.49, 0.5); }
        draw_wave(cg, &mic_levels, h * 0.75, amp, mx, uw, w);

        // Sys — bottom half, blue when active
        if sys_vad { CGContextSetRGBStrokeColor(cg, 0.3, 0.5, 0.95, 0.9); }
        else        { CGContextSetRGBStrokeColor(cg, 0.42, 0.44, 0.49, 0.3); }
        draw_wave(cg, &sys_levels, h * 0.25, amp, mx, uw, w);

        CGContextRestoreGState(cg);
    }
}

// ── Overlay lifecycle ─────────────────────────────────────────────────────────

fn init_overlay() {
    unsafe {
        let sup = cls(b"NSView\0");
        if sup.is_null() { return; }
        let new_cls = objc_allocateClassPair(sup, b"CLXWaveformView\0".as_ptr() as *const _, 0);
        if new_cls.is_null() { return; } // already registered
        let types = b"v@:{CGRect={CGPoint=dd}{CGSize=dd}}\0";
        class_addMethod(new_cls, sel(b"drawRect:\0"), draw_rect as *const c_void,
                        types.as_ptr() as *const _);
        objc_registerClassPair(new_cls);
    }
}

fn show_overlay() {
    unsafe {
        let q = main_queue();
        if !q.is_null() { dispatch_async_f(q, std::ptr::null_mut(), show_main); }
    }
}

extern "C" fn show_main(_: *mut c_void) {
    unsafe {
        let existing = WINDOW_PTR.load(Ordering::Acquire);
        if !existing.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
                std::mem::transmute(objc_msgSend as *const ());
            f(existing, sel(b"orderFront:\0"), std::ptr::null_mut());
            let handle = HANDLE_PTR.load(Ordering::Acquire);
            if !handle.is_null() { f(handle, sel(b"orderFront:\0"), std::ptr::null_mut()); }
            return;
        }

        let scr = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
        if scr.is_null() { return; }
        #[cfg(target_arch = "aarch64")]
        let sf: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                std::mem::transmute(objc_msgSend as *const ());
            f(scr, sel(b"frame\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let sf = NSRect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };

        let ow = 600.0_f64;
        let oh = 100.0_f64;
        let rect = NSRect { x: (sf.w - ow) / 2.0, y: sf.h - oh - 40.0, w: ow, h: oh };

        // Main window — fully transparent, floating, hidden from screen share.
        let win: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(cls(b"NSWindow\0"), sel(b"alloc\0")),
              sel(b"initWithContentRect:styleMask:backing:defer:\0"),
              rect, 0u64, 2u64, false)
        };
        if win.is_null() { return; }

        let f_bool: extern "C" fn(*mut c_void, *mut c_void, bool) =
            std::mem::transmute(objc_msgSend as *const ());
        let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
            std::mem::transmute(objc_msgSend as *const ());
        let f_u64: extern "C" fn(*mut c_void, *mut c_void, u64) =
            std::mem::transmute(objc_msgSend as *const ());
        let f_show: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());

        f_bool(win, sel(b"setOpaque:\0"), false);
        msg1_ptr(win, sel(b"setBackgroundColor:\0"),
                 msg0(cls(b"NSColor\0"), sel(b"clearColor\0")));
        f_i64(win, sel(b"setLevel:\0"), 3); // floating
        f_bool(win, sel(b"setIgnoresMouseEvents:\0"), true);
        f_bool(win, sel(b"setHasShadow:\0"), false);
        f_u64(win, sel(b"setSharingType:\0"), 0); // NSWindowSharingNone
        f_u64(win, sel(b"setCollectionBehavior:\0"), 1 | 16); // allSpaces + stationary

        // Container view
        let container: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(cls(b"NSView\0"), sel(b"alloc\0")),
              sel(b"initWithFrame:\0"),
              NSRect { x: 0.0, y: 0.0, w: ow, h: oh })
        };

        // Waveform view — top 40px
        let view_cls = cls(b"CLXWaveformView\0");
        if view_cls.is_null() { return; }
        let wf_view: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(view_cls, sel(b"alloc\0")),
              sel(b"initWithFrame:\0"),
              NSRect { x: 0.0, y: oh - 40.0, w: ow, h: 40.0 })
        };
        msg1_ptr(container, sel(b"addSubview:\0"), wf_view);

        // Subtitle label — bottom portion
        let label: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(cls(b"NSTextField\0"), sel(b"alloc\0")),
              sel(b"initWithFrame:\0"),
              NSRect { x: 10.0, y: 4.0, w: ow - 20.0, h: oh - 28.0 })
        };
        if !label.is_null() {
            f_bool(label, sel(b"setBezeled:\0"), false);
            f_bool(label, sel(b"setDrawsBackground:\0"), false);
            f_bool(label, sel(b"setEditable:\0"), false);
            f_bool(label, sel(b"setSelectable:\0"), false);
            f_i64(label, sel(b"setAlignment:\0"), 1); // center
            f_i64(label, sel(b"setMaximumNumberOfLines:\0"), 0);
            let cell = msg0(label, sel(b"cell\0"));
            if !cell.is_null() {
                f_i64(cell, sel(b"setLineBreakMode:\0"), 0); // word wrap
                f_bool(cell, sel(b"setWraps:\0"), true);
            }
            set_attributed_subtitle(label, "🎤 Listening...");
            msg1_ptr(container, sel(b"addSubview:\0"), label);
            LABEL_PTR.store(label, Ordering::Release);
        }

        msg1_ptr(win, sel(b"setContentView:\0"), container);
        f_show(win, sel(b"orderFront:\0"), std::ptr::null_mut());
        VIEW_PTR.store(wf_view, Ordering::Release);
        WINDOW_PTR.store(win, Ordering::Release);

        // Drag handle — ⠿ panel at left edge, always visible at alpha 0.4
        // (no hover detection in test binary — CLX uses core-graphics for that)
        let handle_w = 20.0_f64;
        let handle_rect = NSRect { x: rect.x - handle_w, y: rect.y, w: handle_w, h: oh };
        let handle: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(cls(b"NSPanel\0"), sel(b"alloc\0")),
              sel(b"initWithContentRect:styleMask:backing:defer:\0"),
              handle_rect, (1 << 4) | (1 << 7), 2u64, false)
        };
        if !handle.is_null() {
            let f_f64: extern "C" fn(*mut c_void, *mut c_void, f64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_bool(handle, sel(b"setOpaque:\0"), false);
            f_bool(handle, sel(b"setIgnoresMouseEvents:\0"), false);
            f_bool(handle, sel(b"setMovableByWindowBackground:\0"), true);
            f_bool(handle, sel(b"setHasShadow:\0"), false);
            f_bool(handle, sel(b"setHidesOnDeactivate:\0"), false);
            f_i64(handle, sel(b"setLevel:\0"), 3);
            f_f64(handle, sel(b"setAlphaValue:\0"), 0.4);
            f_u64(handle, sel(b"setSharingType:\0"), 0);
            f_u64(handle, sel(b"setCollectionBehavior:\0"), 1 | 16);

            let bg_color: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.3, 0.3, 0.4, 0.7)
            };
            msg1_ptr(handle, sel(b"setBackgroundColor:\0"), bg_color);

            // ⠿ grip label
            let grip: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(msg0(cls(b"NSTextField\0"), sel(b"alloc\0")),
                  sel(b"initWithFrame:\0"),
                  NSRect { x: 0.0, y: 0.0, w: handle_w, h: oh })
            };
            msg1_ptr(grip, sel(b"setStringValue:\0"), nsstring("⠿"));
            f_bool(grip, sel(b"setBezeled:\0"), false);
            f_bool(grip, sel(b"setDrawsBackground:\0"), false);
            f_bool(grip, sel(b"setEditable:\0"), false);
            f_bool(grip, sel(b"setSelectable:\0"), false);
            let grip_color: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.7, 0.7, 0.8, 1.0)
            };
            msg1_ptr(grip, sel(b"setTextColor:\0"), grip_color);
            f_u64(grip, sel(b"setAlignment:\0"), 1);
            msg1_ptr(msg0(handle, sel(b"contentView\0")), sel(b"addSubview:\0"), grip);

            // Link as child — moving handle moves overlay.
            let f_child: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, u64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_child(handle, sel(b"addChildWindow:ordered:\0"), win, 1);
            msg0(handle, sel(b"retain\0"));
            f_show(handle, sel(b"orderFront:\0"), std::ptr::null_mut());
            HANDLE_PTR.store(handle, Ordering::Release);
        }
    }
}

// ── Attributed subtitle helpers (identical to CLX voice_overlay) ──────────────

unsafe fn nscolor(r: f64, g: f64, b: f64, a: f64) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), r, g, b, a)
}

unsafe fn make_attrs(bg: *mut c_void) -> *mut c_void {
    let f2: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let dict = msg0(msg0(cls(b"NSMutableDictionary\0"), sel(b"alloc\0")), sel(b"init\0"));
    f2(dict, sel(b"setObject:forKey:\0"),
       msg0(cls(b"NSColor\0"), sel(b"whiteColor\0")), nsstring("NSColor"));
    f2(dict, sel(b"setObject:forKey:\0"), bg, nsstring("NSBackgroundColor"));
    let font: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 14.0_f64)
    };
    f2(dict, sel(b"setObject:forKey:\0"), font, nsstring("NSFont"));
    let para = msg0(msg0(cls(b"NSMutableParagraphStyle\0"), sel(b"alloc\0")), sel(b"init\0"));
    let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
        std::mem::transmute(objc_msgSend as *const ());
    f_i64(para, sel(b"setAlignment:\0"), 1);
    f2(dict, sel(b"setObject:forKey:\0"), para, nsstring("NSParagraphStyle"));
    dict
}

unsafe fn set_attributed_subtitle(label: *mut c_void, text: &str) {
    let pool = msg0(msg0(cls(b"NSAutoreleasePool\0"), sel(b"alloc\0")), sel(b"init\0"));

    let bg_me      = nscolor(0.1, 0.35, 0.15, 0.8); // dark green  — 🎤 lines
    let bg_other   = nscolor(0.1, 0.2,  0.4,  0.8); // dark blue   — 🔊 lines
    let bg_default = nscolor(0.0, 0.0,  0.0,  0.7); // dark        — other

    let result = msg0(msg0(cls(b"NSMutableAttributedString\0"), sel(b"alloc\0")),
                      sel(b"init\0"));
    let lines: Vec<&str> = text.split('\n').collect();
    for (li, line) in lines.iter().enumerate() {
        let (content, bg) = if line.starts_with("🎤 ")      { (*line, bg_me) }
                            else if line.starts_with("🔊 ") { (*line, bg_other) }
                            else                              { (*line, bg_default) };
        if !content.is_empty() {
            let attrs = make_attrs(bg);
            let attr_seg: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void)
                    -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(msg0(cls(b"NSAttributedString\0"), sel(b"alloc\0")),
                  sel(b"initWithString:attributes:\0"), nsstring(content), attrs)
            };
            msg1_ptr(result, sel(b"appendAttributedString:\0"), attr_seg);
        }
        if li < lines.len() - 1 {
            let nl_attrs = make_attrs(nscolor(0.0, 0.0, 0.0, 0.0));
            let nl_seg: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void)
                    -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(msg0(cls(b"NSAttributedString\0"), sel(b"alloc\0")),
                  sel(b"initWithString:attributes:\0"), nsstring("\n"), nl_attrs)
            };
            msg1_ptr(result, sel(b"appendAttributedString:\0"), nl_seg);
        }
    }
    msg1_ptr(label, sel(b"setAttributedStringValue:\0"), result);
    msg0(pool, sel(b"drain\0"));
}

unsafe fn auto_resize_overlay(label: *mut c_void) {
    let win = WINDOW_PTR.load(Ordering::Acquire);
    if win.is_null() { return; }
    let f_frame: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
        std::mem::transmute(objc_msgSend as *const ());
    let win_frame = f_frame(win, sel(b"frame\0"));

    let fit_size: NSRect = {
        let cell = msg0(label, sel(b"cell\0"));
        if cell.is_null() { return; }
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> NSRect =
            std::mem::transmute(objc_msgSend as *const ());
        f(cell, sel(b"cellSizeForBounds:\0"),
          NSRect { x: win_frame.w - 20.0, y: 10000.0, w: 0.0, h: 0.0 })
    };
    let new_h = (fit_size.x + 40.0 + 12.0).clamp(80.0, 500.0);
    if (new_h - win_frame.h).abs() < 2.0 { return; }

    let new_frame = NSRect {
        x: win_frame.x, y: win_frame.y + win_frame.h - new_h,
        w: win_frame.w, h: new_h,
    };
    let f_set: extern "C" fn(*mut c_void, *mut c_void, NSRect, bool) =
        std::mem::transmute(objc_msgSend as *const ());
    f_set(win, sel(b"setFrame:display:\0"), new_frame, true);

    let content = msg0(win, sel(b"contentView\0"));
    if !content.is_null() {
        let f_sf: extern "C" fn(*mut c_void, *mut c_void, NSRect) =
            std::mem::transmute(objc_msgSend as *const ());
        f_sf(content, sel(b"setFrame:\0"),
             NSRect { x: 0.0, y: 0.0, w: win_frame.w, h: new_h });
        let view = VIEW_PTR.load(Ordering::Acquire);
        if !view.is_null() {
            f_sf(view, sel(b"setFrame:\0"),
                 NSRect { x: 0.0, y: new_h - 40.0, w: win_frame.w, h: 40.0 });
        }
        f_sf(label, sel(b"setFrame:\0"),
             NSRect { x: 10.0, y: 4.0, w: win_frame.w - 20.0, h: new_h - 48.0 });
    }
}

// ── Push audio levels (matches CLX voice_overlay public API) ──────────────────

fn push_dual_audio_levels(mic_levels: &[f32], mic_vad: bool,
                          sys_levels: &[f32], sys_vad: bool,
                          subtitle: Option<&str>) {
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
        if let Some(text) = subtitle { g.subtitle = text.to_string(); }
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
            let f: extern "C" fn(*mut c_void, *mut c_void, bool) =
                std::mem::transmute(objc_msgSend as *const ());
            f(view, sel(b"setNeedsDisplay:\0"), true);
        }
        let label = LABEL_PTR.load(Ordering::Acquire);
        if !label.is_null() {
            let text = {
                let g = WAVEFORM_DATA.lock().unwrap();
                if g.subtitle.is_empty() {
                    match (g.mic_vad, g.sys_vad) {
                        (true,  true)  => "🎤 Speaking... 🔊 Playing...".to_string(),
                        (true,  false) => "🎤 Speaking...".to_string(),
                        (false, true)  => "🔊 Playing...".to_string(),
                        (false, false) => "🎤 Listening...".to_string(),
                    }
                } else {
                    g.subtitle.lines().map(|line| {
                        let chars: Vec<char> = line.chars().collect();
                        if chars.len() > 80 {
                            let prefix: String = chars[..2.min(chars.len())].iter().collect();
                            let tail:   String = chars[chars.len()-74..].iter().collect();
                            format!("{}...{}", prefix, tail)
                        } else { line.to_string() }
                    }).collect::<Vec<_>>().join("\n")
                }
            };
            set_attributed_subtitle(label, &text);
            auto_resize_overlay(label);
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    eprintln!("[voice-test] Starting standalone voice transcription test...");
    eprintln!("[voice-test] Press Ctrl+C to stop.");
    eprintln!("[voice-test] Output: stdout + /tmp/clx-voice.log");
    eprintln!();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    unsafe {
        let app = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        if !app.is_null() {
            let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_i64(app, sel(b"setActivationPolicy:\0"), 1); // Accessory — no dock icon
        }
    }

    init_overlay();
    show_overlay();
    eprintln!("[voice-test] Overlay window created (CLX-style: dual waveform + subtitle + drag handle)");

    let running_bg = running.clone();
    std::thread::spawn(move || { audio_loop(running_bg); });

    // Run NSRunLoop on main thread (pumps ObjC events for the overlay).
    unsafe {
        let ns_date    = cls(b"NSDate\0");
        let current_rl = msg0(cls(b"NSRunLoop\0"), sel(b"currentRunLoop\0"));
        let default_mode: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, *const u8, u64) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(msg0(cls(b"NSString\0"), sel(b"alloc\0")),
              sel(b"initWithUTF8String:\0"),
              b"kCFRunLoopDefaultMode\0".as_ptr(), 0)
        };
        while running.load(Ordering::Relaxed) {
            let date: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
                    std::mem::transmute(objc_msgSend as *const ());
                f(ns_date, sel(b"dateWithTimeIntervalSinceNow:\0"), 0.1_f64)
            };
            let _: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void)
                    -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(current_rl, sel(b"runMode:beforeDate:\0"), default_mode, date)
            };
        }
    }

    eprintln!();
    eprintln!("[voice-test] Exiting.");
}

// ── Audio capture + VAD + Whisper loop ───────────────────────────────────────

fn audio_loop(running: Arc<AtomicBool>) {
    let mut local_whisper = match LocalWhisper::new() {
        Ok(w)  => { eprintln!("[voice-test] Whisper model loaded"); Some(w) }
        Err(e) => { eprintln!("[voice-test] Whisper unavailable: {e}"); None }
    };

    let model_bytes = include_bytes!(concat!(
        env!("CARGO_HOME"),
        "/registry/src/index.crates.io-1949cf8c6b5b557f/ten-vad-rs-0.1.6/onnx/ten-vad.onnx"
    ));
    let mut vad = ten_vad_rs::TenVad::new_from_bytes(model_bytes, 16000)
        .expect("failed to create TEN VAD");
    eprintln!("[voice-test] TEN VAD initialized");

    let ac = AudioCapture::new().expect("failed to create audio capture");
    ac.start().expect("failed to start audio capture");
    let sample_rate = ac.sample_rate();
    eprintln!("[voice-test] Audio capture started (sr={sample_rate})");
    eprintln!();

    let mut in_speech            = false;
    let mut speech_frames: usize = 0;
    let mut silence_frames: usize = 0;
    let mut committed_text       = String::new();
    let mut pending_buf: Vec<f32> = Vec::new();
    let mut whisper_pending      = String::new();
    let mut typed_pending        = String::new();
    let mut pending_audio_since_last: usize = 0;
    let mut prev_whisper         = String::new();
    let mut stable_count: usize  = 0;
    let mut remainder: Vec<f32>  = Vec::new();

    let _ = std::fs::write("/tmp/clx-voice.log", "");

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(50));

        let samples = ac.take_samples();
        if samples.is_empty() { continue; }

        let rms: f32 = {
            let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
            (sum_sq / samples.len() as f32).sqrt()
        };

        let samples_16k = resample(&samples, sample_rate, 16000);

        remainder.extend_from_slice(&samples_16k);
        let all = std::mem::take(&mut remainder);

        let was_in_speech = in_speech;
        let mut offset = 0;
        while offset + TEN_VAD_FRAME_SIZE <= all.len() {
            let frame = &all[offset..offset + TEN_VAD_FRAME_SIZE];
            offset += TEN_VAD_FRAME_SIZE;

            let frame_i16: Vec<i16> = frame.iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16).collect();
            let prob = vad.process_frame(&frame_i16).unwrap_or(0.0);

            if !in_speech {
                if prob > SPEECH_START_PROB {
                    speech_frames += 1;
                    pending_buf.extend_from_slice(frame);
                    if speech_frames >= SPEECH_START_FRAMES {
                        in_speech = true;
                        silence_frames = 0;
                        eprint!("[VAD:speech] ");
                    }
                } else {
                    speech_frames = 0;
                    pending_buf.clear();
                }
            } else {
                pending_buf.extend_from_slice(frame);
                if prob > SPEECH_END_PROB {
                    silence_frames = 0;
                } else {
                    silence_frames += 1;
                    if silence_frames >= SILENCE_END_FRAMES {
                        eprint!("[VAD:silence] ");
                        in_speech = false;
                        speech_frames = 0;
                        silence_frames = 0;
                    }
                }
            }
        }
        if offset < all.len() { remainder = all[offset..].to_vec(); }

        pending_audio_since_last += samples_16k.len();

        // Push mic RMS to overlay; subtitle = current streaming transcription.
        let overlay_level = (rms * 10.0).clamp(0.0, 1.0);
        let subtitle: Option<String> = if !typed_pending.is_empty() {
            Some(format!("🎤 {}", typed_pending))
        } else if !committed_text.is_empty() {
            // Show tail of committed text when idle.
            let chars: Vec<char> = format!("🎤 {}", committed_text).chars().collect();
            let s: String = if chars.len() > 120 {
                chars[chars.len()-120..].iter().collect()
            } else { chars.iter().collect() };
            Some(s)
        } else { None };
        push_dual_audio_levels(&[overlay_level], in_speech, &[], false,
                               subtitle.as_deref());

        // Terminal visualization bar.
        {
            let bar_len = 30;
            let filled = ((rms * 200.0).min(1.0) * bar_len as f32) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
            let vad_str = if in_speech { "\x1b[33mSPEECH\x1b[0m" }
                          else         { "\x1b[90msilent\x1b[0m" };
            let preview: String = format!("{committed_text}{typed_pending}")
                .chars().rev().take(40).collect::<Vec<_>>()
                .into_iter().rev().collect();
            eprint!("\r\x1b[K[{bar}] {vad_str} | {preview}");
        }

        if in_speech {
            if pending_audio_since_last >= STREAMING_CHUNK_SAMPLES {
                if let Some(ref mut w) = local_whisper {
                    if let Ok(text) = w.transcribe(&pending_buf) {
                        if !text.is_empty() {
                            whisper_pending = text.clone();
                            if text.starts_with(&typed_pending) {
                                typed_pending = text;
                            }
                            let full = format!("{committed_text}{typed_pending}");
                            let _ = std::fs::write("/tmp/clx-voice.log", &full);
                        }
                    }
                }
                pending_audio_since_last = 0;
            }

            if whisper_pending == prev_whisper && !whisper_pending.is_empty() {
                stable_count += 1;
            } else { stable_count = 0; }
            prev_whisper = whisper_pending.clone();

            let should_commit = (stable_count >= 2 && pending_buf.len() > 32_000)
                || pending_buf.len() > 80_000;
            if should_commit {
                if typed_pending != whisper_pending { typed_pending = whisper_pending.clone(); }
                if !committed_text.is_empty()
                    && !committed_text.ends_with(' ')
                    && !committed_text.ends_with('\n') { committed_text.push(' '); }
                committed_text.push_str(&whisper_pending);
                eprintln!("\r\x1b[K[COMMITTED] {:?}", whisper_pending);
                whisper_pending.clear(); typed_pending.clear();
                prev_whisper.clear(); pending_buf.clear();
                pending_audio_since_last = 0; stable_count = 0;
            }
        } else if was_in_speech {
            if pending_buf.len() > 4800 {
                if let Some(ref mut w) = local_whisper {
                    if let Ok(final_text) = w.transcribe(&pending_buf) {
                        if !final_text.is_empty() { whisper_pending = final_text; }
                    }
                }
            }
            if !committed_text.is_empty()
                && !committed_text.ends_with(' ')
                && !committed_text.ends_with('\n') { committed_text.push(' '); }
            committed_text.push_str(&whisper_pending);
            eprintln!("\r\x1b[K[END] {:?}", whisper_pending);
            let _ = std::fs::write("/tmp/clx-voice.log", &committed_text);

            whisper_pending.clear(); typed_pending.clear();
            prev_whisper.clear(); pending_buf.clear();
            pending_audio_since_last = 0; stable_count = 0;
        }
    }

    ac.stop();
    eprintln!();
    eprintln!("[voice-test] Done. Final text:");
    eprintln!("{committed_text}");
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate { return samples.to_vec(); }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 * ratio;
        let idx0 = src_idx as usize;
        let frac = (src_idx - idx0 as f64) as f32;
        let s0 = samples.get(idx0).copied().unwrap_or(0.0);
        let s1 = samples.get(idx0 + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        loop { std::thread::sleep(std::time::Duration::from_millis(100)); }
    });
    let r = running;
    unsafe { libc_signal(2, move || { r.store(false, Ordering::Relaxed); }); }
}

unsafe fn libc_signal<F: Fn() + Send + 'static>(sig: i32, handler: F) {
    use std::sync::Once;
    static INIT: Once = Once::new();
    static mut HANDLER: Option<Box<dyn Fn() + Send>> = None;

    extern "C" fn signal_handler(_: i32) {
        unsafe { if let Some(ref f) = HANDLER { f(); } }
    }

    INIT.call_once(|| {
        HANDLER = Some(Box::new(handler));
        libc::signal(sig, signal_handler as *const () as libc::sighandler_t);
    });
}
