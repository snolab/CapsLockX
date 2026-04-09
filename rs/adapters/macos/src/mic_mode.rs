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
