/// CLX Voice Standalone — full Space+V pipeline without the rest of CLX.
///
/// Captures mic (VoiceProcessingIO AEC) + system audio (ScreenCaptureKit),
/// runs VAD + STT (SenseVoice/Whisper), shows the exact CLX dual-waveform overlay.
///
/// Build:   cargo build -p capslockx-macos --bin voice-standalone --release
/// Run:     DYLD_LIBRARY_PATH=rs/target/release ./target/release/voice-standalone

#[path = "../voice_overlay.rs"]  mod voice_overlay;
#[path = "../voice_capture.rs"]  mod voice_capture;
#[path = "../system_audio.rs"]   mod system_audio;
#[path = "../mic_mode.rs"]       mod mic_mode;
#[path = "../audio_tap.rs"]      mod audio_tap;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use capslockx_core::key_code::KeyCode;
use capslockx_core::platform::{Platform, SystemAudioStream};
use capslockx_core::modules::voice::VoiceModule;

// Signal flags — set by signal handlers, consumed by timer poll.
static SIG_KEY_DOWN: AtomicBool = AtomicBool::new(false);
static SIG_KEY_UP: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigusr1(_sig: i32) {
    SIG_KEY_DOWN.store(true, Ordering::Relaxed);
}
extern "C" fn handle_sigusr2(_sig: i32) {
    SIG_KEY_UP.store(true, Ordering::Relaxed);
}

// ── Minimal Platform impl — only the voice-related hooks ─────────────────────

struct VoicePlatform;

impl Platform for VoicePlatform {
    // Required stubs (no keyboard/mouse in a standalone voice tool).
    fn key_down(&self, _: KeyCode) {}
    fn key_up(&self, _: KeyCode) {}
    fn mouse_move(&self, _: i32, _: i32) {}
    fn scroll_v(&self, _: i32) {}
    fn scroll_h(&self, _: i32) {}
    fn mouse_button(&self, _: capslockx_core::platform::MouseButton, _: bool) {}

    // Voice hooks — wired to the macos adapter implementations.
    fn start_aec_mic(&self) -> Option<Box<dyn SystemAudioStream>> {
        match voice_capture::VoiceCapture::new() {
            Ok(cap) => {
                if let Err(e) = cap.start() {
                    eprintln!("[voice-standalone] VoiceProcessingIO start failed: {e}");
                    return None;
                }
                eprintln!("[voice-standalone] VoiceProcessingIO AEC mic started");
                Some(Box::new(cap))
            }
            Err(e) => {
                eprintln!("[voice-standalone] VoiceProcessingIO unavailable: {e}");
                None
            }
        }
    }

    fn start_system_audio(&self) -> Option<Box<dyn SystemAudioStream>> {
        // Try Core Audio Taps first (macOS 14.2+).
        match audio_tap::AudioTapCapture::new() {
            Ok(cap) => {
                eprintln!("[voice-standalone] Core Audio Tap system audio started");
                return Some(Box::new(cap));
            }
            Err(e) => {
                eprintln!("[voice-standalone] Core Audio Tap unavailable: {e}");
            }
        }
        // Fallback: ScreenCaptureKit.
        match system_audio::SystemAudioCapture::new() {
            Ok(cap) => {
                eprintln!("[voice-standalone] ScreenCaptureKit system audio started");
                Some(Box::new(cap))
            }
            Err(e) => {
                eprintln!("[voice-standalone] System audio unavailable: {e}");
                None
            }
        }
    }

    fn show_voice_overlay(&self) { voice_overlay::show_overlay(); }
    fn hide_voice_overlay(&self) { voice_overlay::hide_overlay(); }

    fn update_voice_overlay(&self, mic: &[f32], mic_vad: bool, sys: &[f32], sys_vad: bool) {
        voice_overlay::push_dual_audio_levels(mic, mic_vad, sys, sys_vad, None);
    }

    fn update_voice_subtitle(&self, text: &str) {
        voice_overlay::push_audio_levels_with_text(&[], false, Some(text));
    }

    fn type_text(&self, text: &str) {
        // In standalone mode: just print to stdout (no cursor typing).
        println!("{text}");
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    // Capture the FIRST panic message before panic_cannot_unwind swallows it.
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() { s.to_string() }
                  else if let Some(s) = info.payload().downcast_ref::<String>() { s.clone() }
                  else { "unknown".to_string() };
        let loc = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
        eprintln!("\n[PANIC] >>> {} at {} <<<\n", msg, loc);
    }));

    // Kill any previous instance of voice-standalone (dedup).
    {
        let my_pid = std::process::id();
        if let Ok(output) = std::process::Command::new("pgrep").arg("-f").arg("voice-standalone").output() {
            let pids = String::from_utf8_lossy(&output.stdout);
            for line in pids.lines() {
                if let Ok(pid) = line.trim().parse::<u32>() {
                    if pid != my_pid {
                        eprintln!("[voice-standalone] killing old instance (pid {})", pid);
                        let _ = std::process::Command::new("kill").arg(pid.to_string()).output();
                    }
                }
            }
        }
    }

    eprintln!("[voice-standalone] CLX Voice Standalone");
    eprintln!("[voice-standalone] Dual capture: mic (AEC) + system audio");
    eprintln!("[voice-standalone] Ctrl+C to quit\n");

    // Register the NSApplication + overlay class (needs to happen on main thread).
    voice_overlay::init_overlay();

    // Prompt user to enable Voice Isolation if not active.
    mic_mode::ensure_voice_isolation();

    let platform = Arc::new(VoicePlatform);
    let voice = VoiceModule::new(Arc::clone(&platform) as Arc<dyn Platform>);

    // Start immediately with both mic and system audio.
    voice.start_always_on(true);

    // Register SIGUSR1/SIGUSR2 handlers for CLX core to send key down/up signals.
    unsafe {
        libc::signal(libc::SIGUSR1, handle_sigusr1 as libc::sighandler_t);
        libc::signal(libc::SIGUSR2, handle_sigusr2 as libc::sighandler_t);
    }
    eprintln!("[voice-standalone] Signal handlers: SIGUSR1=key_down SIGUSR2=key_up (pid={})", std::process::id());

    // Poll signal flags from a background thread (signal handlers can't call VoiceModule directly).
    {
        let voice_ref = Arc::new(voice);
        let voice_poll = Arc::clone(&voice_ref);
        std::thread::Builder::new().name("sig-poll".into()).spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(20));
                if SIG_KEY_DOWN.swap(false, Ordering::Relaxed) {
                    eprintln!("[voice-standalone] SIGUSR1 → key_down(V)");
                    voice_poll.on_key_down(KeyCode::V);
                }
                if SIG_KEY_UP.swap(false, Ordering::Relaxed) {
                    eprintln!("[voice-standalone] SIGUSR2 → key_up(V)");
                    voice_poll.on_key_up(KeyCode::V);
                }
            }
        }).expect("failed to spawn signal poll thread");
        // voice_ref keeps VoiceModule alive; it's leaked intentionally (process-lifetime).
        std::mem::forget(voice_ref);
    }

    // Run the AppKit/CoreFoundation event loop — required for:
    //   • NSWindow/NSView updates (overlay redraws)
    //   • ScreenCaptureKit stream callbacks
    eprintln!("[voice-standalone] Entering AppKit run loop...");
    unsafe { run_nsapp() };
}

/// Spin NSApplication's run loop. Never returns (until the process is killed).
unsafe fn run_nsapp() {
    use std::ffi::{c_void, c_char};
    extern "C" {
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_msgSend(recv: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }
    let f0: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let f1: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());

    let cls         = objc_getClass(b"NSApplication\0".as_ptr() as *const c_char);
    let sel_shared  = sel_registerName(b"sharedApplication\0".as_ptr() as *const c_char);
    let sel_run     = sel_registerName(b"run\0".as_ptr() as *const c_char);
    let sel_policy  = sel_registerName(b"setActivationPolicy:\0".as_ptr() as *const c_char);

    let app = f0(cls, sel_shared);
    // LSUIElement-style: no Dock icon, no menu bar.
    f1(app, sel_policy, 1 as *mut c_void); // NSApplicationActivationPolicyAccessory = 1
    f0(app, sel_run);
}
