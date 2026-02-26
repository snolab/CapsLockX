/// SendInput wrappers for keyboard and mouse events.
///
/// All injected events are tagged with CLX_EXTRA_INFO so our own hook
/// callback can skip re-processing them.
use std::mem::size_of;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
    MOUSEINPUT, VIRTUAL_KEY, SendInput,
};

/// Magic marker written to dwExtraInfo for every event we inject.
/// The hook callback checks this and skips our own events.
pub const CLX_EXTRA_INFO: usize = 0x434C_5800; // "CLX\0"

// ──────────────────────────────── keyboard ───────────────────────────────────

fn kbd_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

/// Press a key (key-down event)
pub fn key_down(vk: u32) {
    let inp = kbd_input(vk as u16, 0, KEYBD_EVENT_FLAGS(0));
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}

/// Release a key (key-up event)
pub fn key_up(vk: u32) {
    let inp = kbd_input(vk as u16, 0, KEYEVENTF_KEYUP);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}

/// Press and immediately release a key
pub fn tap_key(vk: u32) {
    let dn = kbd_input(vk as u16, 0, KEYBD_EVENT_FLAGS(0));
    let up = kbd_input(vk as u16, 0, KEYEVENTF_KEYUP);
    unsafe { SendInput(&[dn, up], size_of::<INPUT>() as i32) };
}

/// Press and release a key with Shift held (for Shift+Tab etc.)
pub fn tap_key_shifted(vk: u32) {
    use crate::vk::VK_SHIFT;
    let s_dn = kbd_input(VK_SHIFT as u16, 0, KEYBD_EVENT_FLAGS(0));
    let dn = kbd_input(vk as u16, 0, KEYBD_EVENT_FLAGS(0));
    let up = kbd_input(vk as u16, 0, KEYEVENTF_KEYUP);
    let s_up = kbd_input(VK_SHIFT as u16, 0, KEYEVENTF_KEYUP);
    unsafe { SendInput(&[s_dn, dn, up, s_up], size_of::<INPUT>() as i32) };
}

/// Press a key `n` times (capped at 128 to match AHK behaviour)
pub fn tap_key_n(vk: u32, n: i32) {
    let capped = n.clamp(0, 128);
    for _ in 0..capped {
        tap_key(vk);
    }
}

/// Press a key `n` times with Shift held
pub fn tap_key_shifted_n(vk: u32, n: i32) {
    let capped = n.clamp(0, 32);
    for _ in 0..capped {
        tap_key_shifted(vk);
    }
}

/// Press an extended key and release (e.g. media keys)
pub fn tap_extended_key(vk: u32) {
    let dn = kbd_input(vk as u16, 0, KEYEVENTF_EXTENDEDKEY);
    let up = kbd_input(vk as u16, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP);
    unsafe { SendInput(&[dn, up], size_of::<INPUT>() as i32) };
}

// ──────────────────────────────── mouse ──────────────────────────────────────

fn mouse_input(dx: i32, dy: i32, data: i32, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data as u32,
                dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(flags),
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

/// Relative mouse movement
pub fn send_mouse_move(dx: i32, dy: i32) {
    let inp = mouse_input(dx, dy, 0, MOUSEEVENTF_MOVE.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}

/// Vertical scroll (positive = up, negative = down, in WHEEL_DELTA units = 120)
pub fn send_scroll_v(delta: i32) {
    let clamped = delta.clamp(-16384, 16384);
    let inp = mouse_input(0, 0, clamped, MOUSEEVENTF_WHEEL.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}

/// Horizontal scroll (positive = right, negative = left)
pub fn send_scroll_h(delta: i32) {
    let clamped = delta.clamp(-16384, 16384);
    let inp = mouse_input(0, 0, clamped, MOUSEEVENTF_HWHEEL.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}

pub fn mouse_left_down() {
    let inp = mouse_input(0, 0, 0, MOUSEEVENTF_LEFTDOWN.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}
pub fn mouse_left_up() {
    let inp = mouse_input(0, 0, 0, MOUSEEVENTF_LEFTUP.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}
pub fn mouse_right_down() {
    let inp = mouse_input(0, 0, 0, MOUSEEVENTF_RIGHTDOWN.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}
pub fn mouse_right_up() {
    let inp = mouse_input(0, 0, 0, MOUSEEVENTF_RIGHTUP.0);
    unsafe { SendInput(&[inp], size_of::<INPUT>() as i32) };
}
