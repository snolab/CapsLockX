//! Check and prompt for macOS Voice Isolation mic mode.
//!
//! Voice Isolation is Apple's ML-based noise cancellation that dramatically
//! reduces speaker bleed into the mic. It can only be enabled by the user
//! in Control Center, but we can check if it's active and show the picker.

use std::ffi::c_void;

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
}

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }

const MODE_STANDARD: isize = 0;
const _MODE_WIDE_SPECTRUM: isize = 1;
const MODE_VOICE_ISOLATION: isize = 2;

/// Returns the currently active microphone mode.
pub fn active_microphone_mode() -> isize {
    unsafe {
        let device_cls = cls(b"AVCaptureDevice\0");
        if device_cls.is_null() { return MODE_STANDARD; }
        let f: extern "C" fn(*mut c_void, *mut c_void) -> isize =
            std::mem::transmute(objc_msgSend as *const ());
        f(device_cls, sel(b"activeMicrophoneMode\0"))
    }
}

/// Shows the system microphone mode picker (Control Center popover).
pub fn show_microphone_mode_picker() {
    unsafe {
        let device_cls = cls(b"AVCaptureDevice\0");
        if device_cls.is_null() { return; }
        let f: extern "C" fn(*mut c_void, *mut c_void, isize) =
            std::mem::transmute(objc_msgSend as *const ());
        f(device_cls, sel(b"showSystemUserInterface:\0"), 2); // microphoneModes = 2
    }
}

/// Check microphone permission via AVCaptureDevice and request access if
/// not yet determined. Uses a Swift subprocess to trigger the system dialog
/// because creating ObjC blocks from Rust is fragile.
pub fn check_and_request_mic_permission() -> bool {
    unsafe {
        let device_cls = cls(b"AVCaptureDevice\0");
        if device_cls.is_null() { return true; }

        // AVMediaTypeAudio = @"soun"
        let nsstring_cls = cls(b"NSString\0");
        let media_audio: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, *const u8) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(nsstring_cls, sel(b"stringWithUTF8String:\0"), b"soun\0".as_ptr())
        };

        // authorizationStatusForMediaType: -> NSInteger
        let status: isize = {
            let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> isize =
                std::mem::transmute(objc_msgSend as *const ());
            f(device_cls, sel(b"authorizationStatusForMediaType:\0"), media_audio)
        };

        match status {
            0 => {
                eprintln!("[CLX] mic permission: not determined — requesting via subprocess...");
                // Spawn a swift process that requests mic access and waits for user response.
                let result = std::process::Command::new("swift")
                    .args(["-e", r#"
import AVFoundation
import Foundation
let sem = DispatchSemaphore(value: 0)
AVCaptureDevice.requestAccess(for: .audio) { granted in
    if granted { print("granted") } else { print("denied") }
    sem.signal()
}
sem.wait()
"#])
                    .output();
                match result {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if stdout.trim() == "granted" {
                            eprintln!("[CLX] mic permission: authorized (user granted)");
                            return true;
                        } else {
                            eprintln!("[CLX] mic permission: denied by user");
                            return false;
                        }
                    }
                    Err(e) => {
                        eprintln!("[CLX] mic permission: swift subprocess failed: {e}");
                        return false;
                    }
                }
            }
            1 => { eprintln!("[CLX] mic permission: RESTRICTED"); false }
            2 => { eprintln!("[CLX] mic permission: DENIED — enable in System Settings → Privacy & Security → Microphone"); false }
            3 => { eprintln!("[CLX] mic permission: authorized"); true }
            _ => { eprintln!("[CLX] mic permission: unknown status {status}"); true }
        }
    }
}

/// Check mic mode and prompt user if Voice Isolation is not active.
/// Called at voice startup.
pub fn ensure_voice_isolation() {
    let mode = active_microphone_mode();
    let mode_name = match mode {
        0 => "Standard",
        1 => "Wide Spectrum",
        2 => "Voice Isolation",
        _ => "Unknown",
    };
    eprintln!("[CLX] mic mode: {} ({})", mode_name, mode);

    if mode != MODE_VOICE_ISOLATION {
        eprintln!("[CLX] Voice Isolation is NOT active");
        eprintln!("[CLX] Tip: Enable Voice Isolation in Control Center for best STT when speaking directly");
        eprintln!("[CLX] Note: Voice Isolation blocks external audio sources — keep Standard mode to capture nearby speakers");
    } else {
        eprintln!("[CLX] Voice Isolation is active — external speaker audio will be filtered out");
    }
}
