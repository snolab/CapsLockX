//! Disable the physical CapsLock's AlphaShift toggle by remapping it to F18
//! at the HID driver level via Apple's `hidutil`.
//!
//! CGEventTap can suppress CapsLock propagation, but the kernel IOHIDSystem
//! has already toggled AlphaShift and the LED by the time the tap fires.
//! The only way to fully prevent the toggle is to remap the key before it
//! reaches IOHIDSystem.
//!
//! After remap:
//! - Physical CapsLock arrives at CGEventTap as kVK_F18 (0x4F); `key_map.rs`
//!   aliases that to `KeyCode::CapsLock` so the engine is unchanged.
//! - Before applying the remap we **force AlphaShift to OFF via
//!   `IOHIDSetModifierLockState`** so a user whose CapsLock was already ON
//!   can't get stuck (the remap would otherwise prevent them from toggling
//!   it back off with the physical key).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

// ── hidutil JSON ─────────────────────────────────────────────────────────────
// HID Usage Page 0x07 (Keyboard/Keypad), Apple TN2450.
//   0x39 = CapsLock, 0x6D = F18
const REMAP_JSON: &str = r#"{"UserKeyMapping":[{"HIDKeyboardModifierMappingSrc":0x700000039,"HIDKeyboardModifierMappingDst":0x70000006D}]}"#;
const RESET_JSON: &str = r#"{"UserKeyMapping":[]}"#;

static APPLIED: AtomicBool = AtomicBool::new(false);

// ── IOKit AlphaShift state (delegates to ObjC for ABI safety) ──────────────
// See capslock_iokit.m for the implementation. Functions return 0 on success
// and a non-zero IOReturn on failure.

extern "C" {
    fn clx_caps_get(out_state: *mut bool) -> i32;
    fn clx_caps_set(state: bool) -> i32;
    fn clx_caps_toggle() -> i32;
}

/// Read the current OS CapsLock (AlphaShift) state.
pub fn get_caps_state() -> Option<bool> {
    let mut state: bool = false;
    let r = unsafe { clx_caps_get(&mut state) };
    if r == 0 {
        Some(state)
    } else {
        eprintln!("[CLX] clx_caps_get failed: r=0x{:08x}", r);
        None
    }
}

/// Force the OS CapsLock (AlphaShift) state to the given value.
pub fn set_caps_state(on: bool) -> bool {
    let r = unsafe { clx_caps_set(on) };
    if r != 0 {
        eprintln!("[CLX] clx_caps_set({}) failed: r=0x{:08x}", on, r);
    }
    r == 0
}

/// Toggle the OS CapsLock state. Used by single-tap passthrough so users
/// retain real CapsLock functionality even under the F18 remap.
pub fn toggle_caps_state() {
    let r = unsafe { clx_caps_toggle() };
    if r == 0 {
        eprintln!("[CLX] caps toggle OK");
    } else {
        eprintln!("[CLX] caps toggle failed: r=0x{:08x}", r);
    }
}

// ── hidutil apply / reset ───────────────────────────────────────────────────

pub fn apply() {
    if APPLIED.load(Ordering::SeqCst) {
        return;
    }

    // Force AlphaShift OFF first. If user's CapsLock was on, after remap they
    // wouldn't be able to toggle it off with the physical key.
    if let Some(true) = get_caps_state() {
        if set_caps_state(false) {
            eprintln!("[CLX] CapsLock was ON; forced OFF via IOHID before remap");
        } else {
            eprintln!("[CLX] WARNING: CapsLock is ON but IOHIDSetModifierLockState failed");
        }
    }

    match std::process::Command::new("hidutil")
        .args(["property", "--set", REMAP_JSON])
        .output()
    {
        Ok(out) if out.status.success() => {
            APPLIED.store(true, Ordering::SeqCst);
            eprintln!("[CLX] CapsLock → F18 (hidutil); AlphaShift toggle disabled");
            install_cleanup_handlers();
        }
        Ok(out) => {
            eprintln!(
                "[CLX] hidutil remap failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Err(e) => {
            eprintln!(
                "[CLX] hidutil not available ({}); CapsLock will toggle OS AlphaShift",
                e
            );
        }
    }
}

pub fn reset() {
    if !APPLIED.swap(false, Ordering::SeqCst) {
        return;
    }
    let _ = std::process::Command::new("hidutil")
        .args(["property", "--set", RESET_JSON])
        .output();
    eprintln!("[CLX] CapsLock remap reset");
}

pub fn is_applied() -> bool {
    APPLIED.load(Ordering::SeqCst)
}

fn install_cleanup_handlers() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        let handler = signal_handler as *const () as libc::sighandler_t;
        libc::signal(libc::SIGINT, handler);
        libc::signal(libc::SIGTERM, handler);
        libc::signal(libc::SIGHUP, handler);
        libc::atexit(atexit_handler);
    });
}

extern "C" fn signal_handler(sig: i32) {
    // Best-effort. fork+exec+wait are async-signal-safe per POSIX; the
    // alternative is a stale remap until reboot.
    if APPLIED.swap(false, Ordering::SeqCst) {
        let _ = std::process::Command::new("hidutil")
            .args(["property", "--set", RESET_JSON])
            .output();
    }
    unsafe {
        libc::signal(sig, libc::SIG_DFL);
        libc::raise(sig);
    }
}

extern "C" fn atexit_handler() {
    reset();
}
