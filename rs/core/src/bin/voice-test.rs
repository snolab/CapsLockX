/// Standalone voice transcription test binary with floating waveform overlay.
///
/// Behaves like Space+V toggle: starts listening immediately, runs VAD +
/// Whisper streaming, prints to stdout and /tmp/clx-voice.log.
/// Press Ctrl+C to stop.
///
/// Usage:
///   cargo run -p capslockx-core --release --bin voice-test
///
/// This does NOT touch CapsLockX or any keyboard hooks — safe to run alongside.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use capslockx_core::audio_capture::AudioCapture;
use capslockx_core::local_whisper::LocalWhisper;

// Re-use constants and helpers from voice module via copy (they're private).
// We inline the essentials here for a self-contained test binary.

const STREAMING_CHUNK_SAMPLES: usize = 1_600; // 100ms at 16kHz
const TEN_VAD_FRAME_SIZE: usize = 256;
const SPEECH_START_PROB: f32 = 0.5;
const SPEECH_END_PROB: f32 = 0.3;
const SPEECH_START_FRAMES: usize = 2;
const SILENCE_END_FRAMES: usize = 15;

// ── Overlay: Global shared waveform state ───────────────────────────────────

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

// ── ObjC runtime FFI ────────────────────────────────────────────────────────

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    fn objc_allocateClassPair(
        sup: *mut c_void,
        name: *const std::ffi::c_char,
        extra: usize,
    ) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(
        cls: *mut c_void,
        sel: *mut c_void,
        imp: *const c_void,
        types: *const std::ffi::c_char,
    ) -> bool;
    fn dispatch_async_f(queue: *mut c_void, ctx: *mut c_void, work: extern "C" fn(*mut c_void));
    fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
}

#[link(name = "CoreGraphics", kind = "framework")]
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
    fn CGContextAddArc(
        ctx: *mut c_void,
        x: f64,
        y: f64,
        r: f64,
        sa: f64,
        ea: f64,
        cw: i32,
    );
    fn CGContextFillPath(ctx: *mut c_void);
    fn CGContextBeginPath(ctx: *mut c_void);
    fn CGContextClosePath(ctx: *mut c_void);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

unsafe fn sel(name: &[u8]) -> *mut c_void {
    sel_registerName(name.as_ptr() as *const _)
}
unsafe fn cls(name: &[u8]) -> *mut c_void {
    objc_getClass(name.as_ptr() as *const _)
}
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
    dlsym(
        RTLD_DEFAULT,
        b"_dispatch_main_q\0".as_ptr() as *const _,
    )
}

// ── drawRect: callback (orange/yellow theme) ────────────────────────────────

extern "C" fn draw_rect(this: *mut c_void, _cmd: *mut c_void, _dirty: NSRect) {
    unsafe {
        let ns_gfx = cls(b"NSGraphicsContext\0");
        let ctx_obj = msg0(ns_gfx, sel(b"currentContext\0"));
        if ctx_obj.is_null() {
            return;
        }
        let cg = msg0(ctx_obj, sel(b"CGContext\0"));
        if cg.is_null() {
            return;
        }

        // Get bounds
        #[cfg(target_arch = "aarch64")]
        let bounds: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                std::mem::transmute(objc_msgSend as *const ());
            f(this, sel(b"bounds\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let bounds = NSRect {
            x: 0.0,
            y: 0.0,
            w: 400.0,
            h: 80.0,
        };

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
        CGContextAddArc(
            cg,
            rad,
            rad,
            rad,
            std::f64::consts::PI,
            std::f64::consts::PI * 1.5,
            0,
        );
        CGContextAddArc(cg, w - rad, rad, rad, std::f64::consts::PI * 1.5, 0.0, 0);
        CGContextAddArc(
            cg,
            w - rad,
            h - rad,
            rad,
            0.0,
            std::f64::consts::FRAC_PI_2,
            0,
        );
        CGContextAddArc(
            cg,
            rad,
            h - rad,
            rad,
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::PI,
            0,
        );
        CGContextClosePath(cg);
        CGContextFillPath(cg);

        // Waveform
        let mid = h / 2.0;
        let mx = 16.0;
        let uw = w - 2.0 * mx;
        let max_amp = mid - 8.0;

        if vad {
            // Orange: rgb(255, 160, 50) → (1.0, 0.63, 0.2)
            CGContextSetRGBStrokeColor(cg, 1.0, 0.63, 0.2, 0.9);
        } else {
            // Dark gray: rgb(80, 80, 80) → (0.31, 0.31, 0.31)
            CGContextSetRGBStrokeColor(cg, 0.31, 0.31, 0.31, 0.7);
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

// ── Overlay lifecycle ───────────────────────────────────────────────────────

/// Register the custom NSView subclass. Must be called once on main thread.
fn init_overlay() {
    unsafe {
        let sup = cls(b"NSView\0");
        if sup.is_null() {
            return;
        }
        let new_cls = objc_allocateClassPair(
            sup,
            b"VoiceTestWaveformView\0".as_ptr() as *const _,
            0,
        );
        if new_cls.is_null() {
            return;
        } // already registered
        let types = b"v@:{CGRect={CGPoint=dd}{CGSize=dd}}\0";
        class_addMethod(
            new_cls,
            sel(b"drawRect:\0"),
            draw_rect as *const c_void,
            types.as_ptr() as *const _,
        );
        objc_registerClassPair(new_cls);
    }
}

/// Create and show the overlay window at top-center of screen. Must be called on main thread.
fn show_overlay() {
    unsafe {
        let existing = WINDOW_PTR.load(Ordering::Acquire);
        if !existing.is_null() {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
                std::mem::transmute(objc_msgSend as *const ());
            f(existing, sel(b"orderFront:\0"), std::ptr::null_mut());
            return;
        }

        // Get screen size
        let scr = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
        if scr.is_null() {
            return;
        }
        #[cfg(target_arch = "aarch64")]
        let sf: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
                std::mem::transmute(objc_msgSend as *const ());
            f(scr, sel(b"frame\0"))
        };
        #[cfg(target_arch = "x86_64")]
        let sf = NSRect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        };

        let ow = 400.0_f64;
        let oh = 80.0_f64;
        // Position at TOP-center (macOS coords: y=0 is bottom, so top = screen_height - oh - margin)
        let rect = NSRect {
            x: (sf.w - ow) / 2.0,
            y: sf.h - oh - 40.0,
            w: ow,
            h: oh,
        };

        // Create window
        let alloc = msg0(cls(b"NSWindow\0"), sel(b"alloc\0"));
        let win: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(
                alloc,
                sel(b"initWithContentRect:styleMask:backing:defer:\0"),
                rect,
                0u64,
                2u64,
                false,
            )
        };
        if win.is_null() {
            return;
        }

        // Configure
        let f_bool: extern "C" fn(*mut c_void, *mut c_void, bool) =
            std::mem::transmute(objc_msgSend as *const ());
        f_bool(win, sel(b"setOpaque:\0"), false);
        msg1_ptr(
            win,
            sel(b"setBackgroundColor:\0"),
            msg0(cls(b"NSColor\0"), sel(b"clearColor\0")),
        );
        let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
            std::mem::transmute(objc_msgSend as *const ());
        f_i64(win, sel(b"setLevel:\0"), 3); // floating
        f_bool(win, sel(b"setIgnoresMouseEvents:\0"), true);
        f_bool(win, sel(b"setHasShadow:\0"), false);
        let f_u64: extern "C" fn(*mut c_void, *mut c_void, u64) =
            std::mem::transmute(objc_msgSend as *const ());
        f_u64(win, sel(b"setCollectionBehavior:\0"), 1 | 16); // allSpaces + stationary

        // Create view
        let view_cls = cls(b"VoiceTestWaveformView\0");
        if view_cls.is_null() {
            return;
        }
        let view_alloc = msg0(view_cls, sel(b"alloc\0"));
        let vr = NSRect {
            x: 0.0,
            y: 0.0,
            w: ow,
            h: oh,
        };
        let view: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(view_alloc, sel(b"initWithFrame:\0"), vr)
        };
        if view.is_null() {
            return;
        }

        msg1_ptr(win, sel(b"setContentView:\0"), view);
        let f_show: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        f_show(win, sel(b"orderFront:\0"), std::ptr::null_mut());

        VIEW_PTR.store(view, Ordering::Release);
        WINDOW_PTR.store(win, Ordering::Release);
    }
}

/// Push audio levels and request a redraw. Safe to call from any thread.
fn push_audio_levels(levels: &[f32], vad_active: bool) {
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
        if !q.is_null() {
            dispatch_async_f(q, std::ptr::null_mut(), trigger_redraw);
        }
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
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    eprintln!("[voice-test] Starting standalone voice transcription test...");
    eprintln!("[voice-test] Press Ctrl+C to stop.");
    eprintln!("[voice-test] Output: stdout + /tmp/clx-voice.log");
    eprintln!();

    // Ctrl+C handler.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    // Initialize NSApplication on main thread (Accessory policy — no dock icon).
    unsafe {
        let app = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        if !app.is_null() {
            let f_i64: extern "C" fn(*mut c_void, *mut c_void, i64) =
                std::mem::transmute(objc_msgSend as *const ());
            f_i64(app, sel(b"setActivationPolicy:\0"), 1); // NSApplicationActivationPolicyAccessory
        }
    }

    // Register custom NSView subclass and show overlay (main thread).
    init_overlay();
    show_overlay();
    eprintln!("[voice-test] Overlay window created (top-center, orange theme)");

    // Spawn audio loop on background thread.
    let running_bg = running.clone();
    std::thread::spawn(move || {
        audio_loop(running_bg);
    });

    // Run NSApp event loop on main thread.
    // This blocks until the app terminates.
    unsafe {
        let app = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        if !app.is_null() {
            // Run the run loop manually so we can check `running` periodically.
            // Use a CFRunLoop approach: run in default mode with a short timeout.
            let ns_date = cls(b"NSDate\0");
            let ns_runloop = cls(b"NSRunLoop\0");
            let current_rl = msg0(ns_runloop, sel(b"currentRunLoop\0"));
            let default_mode = msg0(cls(b"NSString\0"), sel(b"alloc\0"));
            let mode_str = b"kCFRunLoopDefaultMode\0";
            let default_mode: *mut c_void = {
                let f: extern "C" fn(
                    *mut c_void,
                    *mut c_void,
                    *const u8,
                    u64,
                ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(
                    default_mode,
                    sel(b"initWithUTF8String:\0"),
                    mode_str.as_ptr(),
                    0,
                )
            };

            while running.load(Ordering::Relaxed) {
                // Create a date 0.1 seconds from now.
                let date: *mut c_void = {
                    let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(
                        ns_date,
                        sel(b"dateWithTimeIntervalSinceNow:\0"),
                        0.1_f64,
                    )
                };
                // runMode:beforeDate:
                let _: *mut c_void = {
                    let f: extern "C" fn(
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                    ) -> *mut c_void =
                        std::mem::transmute(objc_msgSend as *const ());
                    f(
                        current_rl,
                        sel(b"runMode:beforeDate:\0"),
                        default_mode,
                        date,
                    )
                };
            }
        }
    }

    eprintln!();
    eprintln!("[voice-test] Exiting.");
}

/// The audio capture + VAD + Whisper loop. Runs on a background thread.
fn audio_loop(running: Arc<AtomicBool>) {
    // Load Whisper model.
    let mut local_whisper = match LocalWhisper::new() {
        Ok(w) => {
            eprintln!("[voice-test] Whisper model loaded");
            Some(w)
        }
        Err(e) => {
            eprintln!("[voice-test] Whisper unavailable: {e}");
            None
        }
    };

    // Create VAD.
    let model_bytes = include_bytes!(concat!(
        env!("CARGO_HOME"),
        "/registry/src/index.crates.io-1949cf8c6b5b557f/ten-vad-rs-0.1.6/onnx/ten-vad.onnx"
    ));
    let mut vad = ten_vad_rs::TenVad::new_from_bytes(model_bytes, 16000)
        .expect("failed to create TEN VAD");
    eprintln!("[voice-test] TEN VAD initialized");

    // Start audio capture.
    let ac = AudioCapture::new().expect("failed to create audio capture");
    ac.start().expect("failed to start audio capture");
    let sample_rate = ac.sample_rate();
    eprintln!("[voice-test] Audio capture started (sr={sample_rate})");
    eprintln!();

    // State.
    let mut in_speech = false;
    let mut speech_frames: usize = 0;
    let mut silence_frames: usize = 0;
    let mut committed_text = String::new();
    let mut pending_buf: Vec<f32> = Vec::new();
    let mut whisper_pending = String::new();
    let mut typed_pending = String::new();
    let mut pending_audio_since_last: usize = 0;
    let mut prev_whisper = String::new();
    let mut stable_count: usize = 0;
    let mut remainder: Vec<f32> = Vec::new();

    // Clear log.
    let _ = std::fs::write("/tmp/clx-voice.log", "");

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(50));

        let samples = ac.take_samples();
        if samples.is_empty() {
            continue;
        }

        // Compute RMS for visualization.
        let rms: f32 = {
            let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
            (sum_sq / samples.len() as f32).sqrt()
        };

        // Resample to 16kHz.
        let samples_16k = resample(&samples, sample_rate, 16000);

        // VAD processing.
        remainder.extend_from_slice(&samples_16k);
        let all = std::mem::take(&mut remainder);

        let was_in_speech = in_speech;
        let mut offset = 0;
        while offset + TEN_VAD_FRAME_SIZE <= all.len() {
            let frame = &all[offset..offset + TEN_VAD_FRAME_SIZE];
            offset += TEN_VAD_FRAME_SIZE;

            let frame_i16: Vec<i16> = frame
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                .collect();
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
        if offset < all.len() {
            remainder = all[offset..].to_vec();
        }

        pending_audio_since_last += samples_16k.len();

        // Push RMS level to overlay (scaled for visibility).
        let overlay_level = (rms * 10.0).clamp(0.0, 1.0);
        push_audio_levels(&[overlay_level], in_speech);

        // Visualization bar: [████░░░░░░] VAD:speech | "current text..."
        {
            let bar_len = 30;
            let filled = ((rms * 200.0).min(1.0) * bar_len as f32) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
            let vad_str = if in_speech {
                "\x1b[33mSPEECH\x1b[0m" // yellow/orange to match overlay
            } else {
                "\x1b[90msilent\x1b[0m"
            };
            let preview: String = format!("{committed_text}{typed_pending}");
            let preview_short: String = preview
                .chars()
                .rev()
                .take(40)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            eprint!("\r\x1b[K[{bar}] {vad_str} | {preview_short}");
        }

        if in_speech {
            // Streaming transcription.
            if pending_audio_since_last >= STREAMING_CHUNK_SAMPLES {
                if let Some(ref mut w) = local_whisper {
                    if let Ok(text) = w.transcribe(&pending_buf) {
                        if !text.is_empty() {
                            whisper_pending = text.clone();

                            // Track what would be typed (append-only).
                            if text.starts_with(&typed_pending) {
                                typed_pending = text;
                            }

                            // Log full text to file.
                            let full = format!("{committed_text}{typed_pending}");
                            let _ = std::fs::write("/tmp/clx-voice.log", &full);
                        }
                    }
                }
                pending_audio_since_last = 0;
            }

            // Stability commit.
            if whisper_pending == prev_whisper && !whisper_pending.is_empty() {
                stable_count += 1;
            } else {
                stable_count = 0;
            }
            prev_whisper = whisper_pending.clone();

            let should_commit = (stable_count >= 2 && pending_buf.len() > 32_000)
                || pending_buf.len() > 80_000;
            if should_commit {
                if typed_pending != whisper_pending {
                    typed_pending = whisper_pending.clone();
                }
                if !committed_text.is_empty() && !committed_text.ends_with(' ') && !committed_text.ends_with('\n') {
                    committed_text.push(' ');
                }
                committed_text.push_str(&whisper_pending);
                eprintln!("\r\x1b[K[COMMITTED] {:?}", whisper_pending);
                whisper_pending.clear();
                typed_pending.clear();
                prev_whisper.clear();
                pending_buf.clear();
                pending_audio_since_last = 0;
                stable_count = 0;
            }
        } else if was_in_speech {
            // Speech ended.
            if pending_buf.len() > 4800 {
                if let Some(ref mut w) = local_whisper {
                    if let Ok(final_text) = w.transcribe(&pending_buf) {
                        if !final_text.is_empty() {
                            whisper_pending = final_text;
                        }
                    }
                }
            }
            if !committed_text.is_empty() && !committed_text.ends_with(' ') && !committed_text.ends_with('\n') {
                    committed_text.push(' ');
                }
                committed_text.push_str(&whisper_pending);
            eprintln!("\r\x1b[K[END] {:?}", whisper_pending);
            let _ = std::fs::write("/tmp/clx-voice.log", &committed_text);

            whisper_pending.clear();
            typed_pending.clear();
            prev_whisper.clear();
            pending_buf.clear();
            pending_audio_since_last = 0;
            stable_count = 0;
            // Don't clear committed_text — keep accumulating across utterances.
        }
    }

    ac.stop();
    eprintln!();
    eprintln!("[voice-test] Done. Final text:");
    eprintln!("{committed_text}");
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
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
        // Simple Ctrl+C detection via signal.
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Check if we should stop (set externally or via signal).
        }
    });
    // Use a simple approach: set running=false on SIGINT.
    let r = running;
    unsafe {
        libc_signal(2, move || {
            r.store(false, Ordering::Relaxed);
        });
    }
}

// Minimal SIGINT handler without pulling in extra crates.
unsafe fn libc_signal<F: Fn() + Send + 'static>(sig: i32, handler: F) {
    use std::sync::Once;
    static INIT: Once = Once::new();
    static mut HANDLER: Option<Box<dyn Fn() + Send>> = None;

    extern "C" fn signal_handler(_: i32) {
        unsafe {
            if let Some(ref f) = HANDLER {
                f();
            }
        }
    }

    INIT.call_once(|| {
        HANDLER = Some(Box::new(handler));
        libc::signal(sig, signal_handler as *const () as libc::sighandler_t);
    });
}
