//! AHK-parity Alt+Tab / Win+Tab switcher enhancement (Windows).
//!
//! Ports the `#if 多任务窗口切换界面内()` hotkey block from
//! `Modules/CLX-WindowManager.ahk`. While the Windows Alt+Tab or Win+Tab
//! switcher is the foreground window **and Alt is physically held**, these keys
//! are remapped (everything else passes through untouched):
//!
//! | Key   | Action              | AHK line          |
//! |-------|---------------------|-------------------|
//! | W A S D | ↑ ← ↓ →           | `!w !a !s !d`     |
//! | Q E     | Media Prev / Next | `!q !e`           |
//! | R F     | Volume Up / Down  | `!r !f`           |
//! | T G     | Media Stop / Play-Pause | `!t !g`     |
//! | H L     | Media Prev / Next | `!h !l`           |
//! | J K     | Volume Down / Up  | `!j !k`           |
//! | M       | Volume Mute       | `!m`              |
//! | X C     | Close highlighted window | `!x !c` (`AltTabViewCloseWindow`) |
//!
//! The remaps only fire while a switcher window is active, so `Alt+W` etc. in
//! ordinary apps are unaffected. Injected arrows keep Alt physically held —
//! `Alt+Arrow` is what moves the highlight in the switcher grid — matching the
//! AHK `{Blind}` remaps that preserve the Alt modifier.

use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId,
};

use crate::output::tap_vk_extended;

// ── Target virtual-key codes (what we inject) ──────────────────────────────
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_DELETE: u16 = 0x2E;
const VK_VOLUME_MUTE: u16 = 0xAD;
const VK_VOLUME_DOWN: u16 = 0xAE;
const VK_VOLUME_UP: u16 = 0xAF;
const VK_MEDIA_NEXT: u16 = 0xB0;
const VK_MEDIA_PREV: u16 = 0xB1;
const VK_MEDIA_STOP: u16 = 0xB2;
const VK_MEDIA_PLAY_PAUSE: u16 = 0xB3;

/// Which switcher variant is up — decides the close-window keystroke.
enum Switcher {
    /// Win11 modern Alt+Tab (`XamlExplorerHostIslandWindow`): Delete closes.
    Win11,
    /// Win10 Alt+Tab / Task View: AHK sends Delete then Right.
    Win10,
}

/// Called from the keyboard hook on a non-injected key-**down** while Alt is
/// held. Returns `true` if the key was consumed — the caller must then suppress
/// the original event so the letter never reaches the switcher.
pub fn try_handle(vk: u32) -> bool {
    // Fast reject: only a handful of keys are ever remapped, so we avoid the
    // GetForegroundWindow/class lookup on every Alt+<letter> combo.
    if !is_candidate(vk) {
        return false;
    }
    let Some(switcher) = active_switcher() else {
        return false;
    };
    match vk {
        0x57 => tap_vk_extended(VK_UP),               // W → ↑
        0x41 => tap_vk_extended(VK_LEFT),             // A → ←
        0x53 => tap_vk_extended(VK_DOWN),             // S → ↓
        0x44 => tap_vk_extended(VK_RIGHT),            // D → →
        0x52 => tap_vk_extended(VK_VOLUME_UP),        // R → Vol+
        0x46 => tap_vk_extended(VK_VOLUME_DOWN),      // F → Vol-
        0x51 => tap_vk_extended(VK_MEDIA_PREV),       // Q → Prev
        0x45 => tap_vk_extended(VK_MEDIA_NEXT),       // E → Next
        0x54 => tap_vk_extended(VK_MEDIA_STOP),       // T → Stop
        0x47 => tap_vk_extended(VK_MEDIA_PLAY_PAUSE), // G → Play/Pause
        0x4B => tap_vk_extended(VK_VOLUME_UP),        // K → Vol+
        0x4A => tap_vk_extended(VK_VOLUME_DOWN),      // J → Vol-
        0x4D => tap_vk_extended(VK_VOLUME_MUTE),      // M → Mute
        0x48 => tap_vk_extended(VK_MEDIA_PREV),       // H → Prev
        0x4C => tap_vk_extended(VK_MEDIA_NEXT),       // L → Next
        0x58 | 0x43 => close_highlighted(&switcher),  // X / C → close
        _ => return false,
    }
    true
}

/// The set of source keys the switcher enhancement remaps. Keeping this cheap
/// and separate lets the hook bail before touching any Win32 window APIs.
fn is_candidate(vk: u32) -> bool {
    matches!(
        vk,
        0x41 | 0x44 | 0x53 | 0x57 // W A S D
            | 0x51 | 0x45 | 0x52 | 0x46 | 0x54 | 0x47 // Q E R F T G
            | 0x48 | 0x4A | 0x4B | 0x4C | 0x4D // H J K L M
            | 0x58 | 0x43 // X C
    )
}

/// Close the highlighted window in the switcher (AHK `AltTabViewCloseWindow`).
/// Alt stays physically held (the AHK `{Blind}` behaviour), so this is
/// effectively Alt+Delete — which the switcher interprets as "close this one".
fn close_highlighted(switcher: &Switcher) {
    tap_vk_extended(VK_DELETE);
    if matches!(switcher, Switcher::Win10) {
        // AHK: `SendEvent {Blind}{Delete}{Right}` on Win10 — after the close,
        // nudge the highlight right so focus lands on a live entry.
        tap_vk_extended(VK_RIGHT);
    }
}

/// Detect whether a task-switcher window is currently in the foreground.
///
/// Mirrors AHK's `多任务窗口切换界面内()`:
///   * `XamlExplorerHostIslandWindow` — Win11 Alt+Tab / Task View
///   * `MultitaskingViewFrame`        — Win10 Alt+Tab / Task View
///   * `Windows.UI.Core.CoreWindow` owned by explorer.exe — Win10 Win+Tab
fn active_switcher() -> Option<Switcher> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 128];
        let n = GetClassNameW(hwnd, &mut buf);
        if n <= 0 {
            return None;
        }
        let class = String::from_utf16_lossy(&buf[..n as usize]);
        match class.as_str() {
            "XamlExplorerHostIslandWindow" => Some(Switcher::Win11),
            "MultitaskingViewFrame" => Some(Switcher::Win10),
            "Windows.UI.Core.CoreWindow" if process_is_explorer(hwnd) => Some(Switcher::Win10),
            _ => None,
        }
    }
}

/// True if `hwnd`'s owning process is explorer.exe. Guards the generic
/// `Windows.UI.Core.CoreWindow` class (used by many UWP apps) so we only treat
/// the shell's Win+Tab view as a switcher.
fn process_is_explorer(hwnd: HWND) -> bool {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return false;
        }
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, BOOL(0), pid) else {
            return false;
        };
        let mut buf = [0u16; MAX_PATH as usize];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if !ok {
            return false;
        }
        String::from_utf16_lossy(&buf[..len as usize])
            .to_ascii_lowercase()
            .ends_with("explorer.exe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_switcher_keys_are_candidates() {
        // Every remapped source key is a candidate…
        for vk in [
            0x41, 0x44, 0x53, 0x57, // WASD
            0x51, 0x45, 0x52, 0x46, 0x54, 0x47, // QERF TG
            0x48, 0x4A, 0x4B, 0x4C, 0x4D, // HJKL M
            0x58, 0x43, // XC
        ] {
            assert!(is_candidate(vk), "vk 0x{vk:02X} should be a candidate");
        }
        // …and unrelated keys are not (so the hook stays off the window APIs).
        for vk in [
            0x42u32, /* B */
            0x5A,    /* Z */
            0x20,    /* Space */
            0x09,    /* Tab */
        ] {
            assert!(!is_candidate(vk), "vk 0x{vk:02X} must not be a candidate");
        }
    }

    #[test]
    fn try_handle_ignores_non_candidate_without_touching_window_apis() {
        // A non-candidate must short-circuit before any foreground lookup.
        assert!(!try_handle(0x5A)); // Z
        assert!(!try_handle(0x42)); // B
    }
}
