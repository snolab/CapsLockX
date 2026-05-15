//! Force the system cursor to remain visible while CLX mouse mode is active.
//!
//! On laptops / tablets with no physical mouse plugged in, Windows suppresses
//! the cursor (`CURSOR_SUPPRESSED`). CLX-Mouse moves the pointer via
//! `SendInput`, but the user can't see where it is.
//!
//! Strategy: when CLX mode engages, enable Windows' Mouse Keys accessibility
//! feature *silently* (no numpad hijack, no UI sound). This makes Windows
//! treat the system as having an active pointing device and unsuppresses
//! the cursor. On disengage, restore the prior Mouse Keys state.

use std::sync::Mutex;

use once_cell::sync::Lazy;
use windows::Win32::UI::Accessibility::MOUSEKEYS;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETMOUSEKEYS, SPI_SETMOUSEKEYS,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

// Mouse Keys flag bits (from WinUser.h — not exported by windows-rs 0.58).
const MKF_MOUSEKEYSON: u32 = 0x00000001;
const MKF_AVAILABLE: u32 = 0x00000002;

static SAVED: Lazy<Mutex<Option<MOUSEKEYS>>> = Lazy::new(|| Mutex::new(None));

/// Enable a silent Mouse Keys configuration so Windows shows the cursor.
/// Idempotent — calling twice has no extra effect.
pub fn enable() {
    let mut saved = SAVED.lock().unwrap();
    if saved.is_some() {
        return;
    }

    unsafe {
        // Snapshot current settings so we can restore on disable().
        let mut current = MOUSEKEYS {
            cbSize: std::mem::size_of::<MOUSEKEYS>() as u32,
            ..Default::default()
        };
        let ok = SystemParametersInfoW(
            SPI_GETMOUSEKEYS,
            current.cbSize,
            Some(&mut current as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        if ok.is_err() {
            return;
        }
        *saved = Some(current);

        // Enable Mouse Keys without hijacking the numpad: turn ON + AVAILABLE,
        // but do NOT set MKF_REPLACENUMBERS — numpad keeps typing digits.
        let mut want = current;
        want.dwFlags = MKF_MOUSEKEYSON | MKF_AVAILABLE;
        let _ = SystemParametersInfoW(
            SPI_SETMOUSEKEYS,
            want.cbSize,
            Some(&mut want as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }

    // Nudge cursor suppression: send a 0-delta mouse move so Windows
    // registers "mouse activity" and reveals a hidden cursor.
    nudge();
}

/// Restore the Mouse Keys settings captured by `enable()`.
pub fn disable() {
    let mut saved = SAVED.lock().unwrap();
    let Some(mut prior) = saved.take() else { return };
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_SETMOUSEKEYS,
            prior.cbSize,
            Some(&mut prior as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
}

/// Send a zero-delta relative mouse move. Wakes the cursor from
/// `CURSOR_SUPPRESSED` state on touch-only systems.
pub fn nudge() {
    unsafe {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(
            std::slice::from_mut(&mut input),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}
