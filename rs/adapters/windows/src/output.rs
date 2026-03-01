/// WinPlatform – implements the `Platform` trait using Win32 APIs.
///
/// Keyboard / mouse output: SendInput (tagged with CLX_EXTRA_INFO so our
/// own hook callback can skip self-injected events).
/// Window management: Win32 window enumeration + manipulation APIs.
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{BOOL, CloseHandle, COLORREF, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL, MOUSE_EVENT_FLAGS, MOUSEINPUT, VIRTUAL_KEY, SendInput,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowLongW, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsWindowVisible, SendMessageW, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, GWL_STYLE, HWND_NOTOPMOST, HWND_TOPMOST,
    LWA_ALPHA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SW_HIDE, SW_RESTORE, WS_EX_LAYERED, WS_EX_TOPMOST,
};

use capslockx_core::{KeyCode, Platform};
use capslockx_core::platform::{ArrangeMode, MouseButton};
use crate::vd_api;
use crate::vk::keycode_to_vk;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Magic tag on SendInput events so our hook callback skips them.
pub const CLX_EXTRA_INFO: usize = 0x434C_5800;

const WS_CAPTION_RAW: u32       = 0x00C0_0000;
const WS_EX_TOOLWINDOW_RAW: u32 = 0x0000_0080;
const WM_CLOSE: u32             = 0x0010;

// ── WinPlatform ───────────────────────────────────────────────────────────────

pub struct WinPlatform {
    /// HWND stored during V-hold transparent (0 = none).
    v_hwnd: AtomicUsize,
    /// Tracked current virtual desktop index (1-based).
    desktop_idx: AtomicUsize,
}

impl WinPlatform {
    pub fn new() -> Self {
        Self {
            v_hwnd: AtomicUsize::new(0),
            desktop_idx: AtomicUsize::new(1),
        }
    }
}

// ── SendInput helpers ────────────────────────────────────────────────────────

fn kbd(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    unsafe { SendInput(inputs, size_of::<INPUT>() as i32); }
}

fn mouse_inp(dx: i32, dy: i32, data: i32, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx, dy,
                mouseData: data as u32,
                dwFlags: MOUSE_EVENT_FLAGS(flags),
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

// ── Platform impl ─────────────────────────────────────────────────────────────

impl Platform for WinPlatform {
    fn key_down(&self, key: KeyCode) {
        send(&[kbd(keycode_to_vk(key), KEYBD_EVENT_FLAGS(0))]);
    }
    fn key_up(&self, key: KeyCode) {
        send(&[kbd(keycode_to_vk(key), KEYEVENTF_KEYUP)]);
    }
    fn key_tap(&self, key: KeyCode) {
        let vk = keycode_to_vk(key);
        send(&[kbd(vk, KEYBD_EVENT_FLAGS(0)), kbd(vk, KEYEVENTF_KEYUP)]);
    }
    fn key_tap_extended(&self, key: KeyCode) {
        let vk = keycode_to_vk(key);
        send(&[
            kbd(vk, KEYEVENTF_EXTENDEDKEY),
            kbd(vk, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
        ]);
    }

    fn mouse_move(&self, dx: i32, dy: i32) {
        send(&[mouse_inp(dx, dy, 0, MOUSEEVENTF_MOVE.0)]);
    }
    fn scroll_v(&self, delta: i32) {
        send(&[mouse_inp(0, 0, delta.clamp(-16384, 16384), MOUSEEVENTF_WHEEL.0)]);
    }
    fn scroll_h(&self, delta: i32) {
        send(&[mouse_inp(0, 0, delta.clamp(-16384, 16384), MOUSEEVENTF_HWHEEL.0)]);
    }
    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let flag = match (button, pressed) {
            (MouseButton::Left,  true)  => MOUSEEVENTF_LEFTDOWN.0,
            (MouseButton::Left,  false) => MOUSEEVENTF_LEFTUP.0,
            (MouseButton::Right, true)  => MOUSEEVENTF_RIGHTDOWN.0,
            (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP.0,
            _ => return,
        };
        send(&[mouse_inp(0, 0, 0, flag)]);
    }

    // ── Window management ─────────────────────────────────────────────────────

    fn cycle_windows(&self, dir: i32) {
        let windows = get_app_windows();
        let fg = unsafe { GetForegroundWindow() };
        let pos = windows.iter().position(|&h| h == fg);
        match pos {
            None => {
                // No focused window in list — focus first/last on current desktop.
                if let Some(&w) = if dir > 0 { windows.first() } else { windows.last() } {
                    unsafe { let _ = SetForegroundWindow(w); }
                }
            }
            Some(idx) => {
                let new_idx = idx as i32 + dir;
                if new_idx >= 0 && (new_idx as usize) < windows.len() {
                    // Normal: activate adjacent window on the same desktop.
                    unsafe { let _ = SetForegroundWindow(windows[new_idx as usize]); }
                } else {
                    // At the boundary: switch to the next/prev virtual desktop.
                    // Run synchronously on the hook thread (which has COM/message-pump)
                    // rather than a spawned thread (which does not).
                    switch_desktop_step(dir);
                }
            }
        }
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
        match mode {
            ArrangeMode::SideBySide => arrange_side_by_side(),
            ArrangeMode::Stacked    => arrange_stacked(),
        }
    }

    fn close_tab(&self) {
        self.key_tap_ctrl(KeyCode::W);
    }

    fn close_window(&self) {
        let hwnd = unsafe { GetForegroundWindow() };
        self.cycle_windows(1);
        unsafe { SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)); }
    }

    fn kill_window(&self) {
        let hwnd = unsafe { GetForegroundWindow() };
        self.cycle_windows(1);
        unsafe {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != 0 {
                if let Ok(h) = OpenProcess(PROCESS_TERMINATE, BOOL(0), pid) {
                    let _ = TerminateProcess(h, 1);
                    let _ = CloseHandle(h);
                }
            }
        }
    }

    fn set_window_transparent(&self, alpha: u8) {
        let hwnd = unsafe { GetForegroundWindow() };
        self.v_hwnd.store(hwnd.0 as usize, Ordering::Relaxed);
        set_layered_alpha(hwnd, alpha);
        unsafe {
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }

    fn restore_window(&self) {
        let raw = self.v_hwnd.swap(0, Ordering::Relaxed);
        if raw != 0 {
            let hwnd = HWND(raw as *mut _);
            set_layered_alpha(hwnd, 255);
            unsafe {
                let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    fn toggle_window_topmost(&self) {
        let hwnd = unsafe { GetForegroundWindow() };
        let exstyle = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 };
        unsafe {
            if exstyle & WS_EX_TOPMOST.0 != 0 {
                set_layered_alpha(hwnd, 255);
                let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            } else {
                set_layered_alpha(hwnd, 200);
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    // ── Virtual desktop ────────────────────────────────────────────────────────

    fn switch_to_desktop(&self, idx: u32) {
        let idx = idx.clamp(1, 10) as usize;
        // Try instant COM API first (Win10+, position-independent).
        if vd_api::switch_desktop(idx) { return; }
        // Hotkey fallback: query real current position, then send Win+Ctrl+Arrow.
        let cur = vd_api::current_desktop_idx()
            .unwrap_or_else(|| self.desktop_idx.load(Ordering::Relaxed));
        if cur == idx { return; }
        navigate_desktops(cur, idx);
        self.desktop_idx.store(idx, Ordering::Relaxed);
    }

    fn restart(&self) {
        // Spawn a new instance of ourselves, then exit.
        if let Ok(exe) = std::env::current_exe() {
            let wd = std::env::current_dir().unwrap_or_else(|_| exe.parent().unwrap().to_path_buf());
            let _ = std::process::Command::new(&exe).current_dir(wd).spawn();
        }
        std::process::exit(0);
    }

    fn move_window_to_desktop(&self, idx: u32) {
        let idx = idx.clamp(1, 10) as usize;
        let cur = vd_api::current_desktop_idx()
            .unwrap_or_else(|| self.desktop_idx.load(Ordering::Relaxed));
        if cur == idx { return; }
        // Hide window, switch desktop, show it on the new desktop.
        let hwnd = unsafe { GetForegroundWindow() };
        unsafe { let _ = ShowWindow(hwnd, SW_HIDE); }
        if !vd_api::switch_desktop(idx) {
            navigate_desktops(cur, idx);
            self.desktop_idx.store(idx, Ordering::Relaxed);
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

// ── Window enumeration ────────────────────────────────────────────────────────

extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() { return BOOL(1); }
        let style   = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if style & WS_CAPTION_RAW == 0 { return BOOL(1); }
        if exstyle & WS_EX_TOOLWINDOW_RAW != 0 { return BOOL(1); }
        if GetWindowTextLengthW(hwnd) == 0 { return BOOL(1); }
        // Skip cloaked windows (e.g. UWP apps on other virtual desktops).
        let mut cloaked: u32 = 0;
        let _ = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if cloaked != 0 { return BOOL(1); }
        (&mut *(lparam.0 as *mut Vec<HWND>)).push(hwnd);
        BOOL(1)
    }
}

fn get_app_windows() -> Vec<HWND> {
    let mut v: Vec<HWND> = Vec::new();
    unsafe { let _ = EnumWindows(Some(enum_callback), LPARAM(&mut v as *mut Vec<HWND> as isize)); }
    // Sort by HWND value for a stable ordering that doesn't shift when focus changes.
    // EnumWindows returns Z-order (top window first), which rearranges on every focus
    // change and causes cycling to bounce between two windows.
    v.sort_by_key(|h| h.0 as usize);
    v
}

// ── Virtual desktop helpers ───────────────────────────────────────────────────

/// Step one virtual desktop in `dir` direction (+1 or -1).
/// Uses the instant COM API when available; falls back to Win+Ctrl+Arrow.
fn switch_desktop_step(dir: i32) {
    // Query real current index so we don't drift if user switched manually.
    if let Some(cur) = vd_api::current_desktop_idx() {
        let next = (cur as i32 + dir).max(1) as usize;
        if vd_api::switch_desktop(next) { return; }
    }
    // Hotkey fallback.
    const VK_LWIN: u16  = 0x5B;
    const VK_LCTRL: u16 = 0xA2;
    const VK_LEFT: u16  = 0x25;
    const VK_RIGHT: u16 = 0x27;
    let vk_dir = if dir > 0 { VK_RIGHT } else { VK_LEFT };
    send(&[
        kbd(VK_LWIN,  KEYBD_EVENT_FLAGS(0)),
        kbd(VK_LCTRL, KEYBD_EVENT_FLAGS(0)),
        kbd(vk_dir,   KEYBD_EVENT_FLAGS(0)),
        kbd(vk_dir,   KEYEVENTF_KEYUP),
        kbd(VK_LCTRL, KEYEVENTF_KEYUP),
        kbd(VK_LWIN,  KEYEVENTF_KEYUP),
    ]);
}

/// Navigate from desktop `from` to desktop `to` by sending Win+Ctrl+Left/Right.
fn navigate_desktops(from: usize, to: usize) {
    // VK codes
    const VK_LWIN: u16  = 0x5B;
    const VK_LCTRL: u16 = 0xA2;
    const VK_LEFT: u16  = 0x25;
    const VK_RIGHT: u16 = 0x27;

    let (count, vk_dir) = if to > from {
        (to - from, VK_RIGHT)
    } else {
        (from - to, VK_LEFT)
    };

    // Hold Win+Ctrl
    send(&[
        kbd(VK_LWIN, KEYBD_EVENT_FLAGS(0)),
        kbd(VK_LCTRL, KEYBD_EVENT_FLAGS(0)),
    ]);
    // Tap direction key `count` times
    for _ in 0..count {
        send(&[
            kbd(vk_dir, KEYBD_EVENT_FLAGS(0)),
            kbd(vk_dir, KEYEVENTF_KEYUP),
        ]);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    // Release Win+Ctrl
    send(&[
        kbd(VK_LCTRL, KEYEVENTF_KEYUP),
        kbd(VK_LWIN, KEYEVENTF_KEYUP),
    ]);
}

// ── Transparency helpers ──────────────────────────────────────────────────────

fn set_layered_alpha(hwnd: HWND, alpha: u8) {
    unsafe {
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if ex & WS_EX_LAYERED.0 == 0 {
            SetWindowLongW(hwnd, GWL_EXSTYLE, (ex | WS_EX_LAYERED.0) as i32);
        }
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

// ── Window arrangement ────────────────────────────────────────────────────────

fn get_work_rect(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut mi).as_bool() {
            let r = mi.rcWork;
            (r.left, r.top, r.right - r.left, r.bottom - r.top)
        } else {
            (0, 0, 1920, 1080)
        }
    }
}

fn fast_resize(hwnd: HWND, x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetWindowPos(hwnd, HWND(std::ptr::null_mut()), x, y, w, h,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS);
    }
}

fn arrange_side_by_side() {
    let windows = get_app_windows();
    let n = windows.len();
    if n == 0 { return; }
    let fg = unsafe { GetForegroundWindow() };
    let (ax, ay, aw, ah) = get_work_rect(fg);
    let (rows, cols) = if aw <= ah {
        let c = (n as f64).sqrt() as usize;
        let c = c.max(1);
        (n.div_ceil(c), c)
    } else {
        let r = (n as f64).sqrt() as usize;
        let r = r.max(1);
        (r, n.div_ceil(r))
    };
    for (k, &hwnd) in windows.iter().enumerate() {
        let nx = (k % cols) as i32;
        let ny = (k / cols) as i32;
        let sw = aw / cols as i32;
        let sh = ah / rows as i32;
        let mut x = ax + nx * sw - 8;
        let mut y = ay + ny * sh;
        let mut w = sw + 16;
        let mut h = sh + 8;
        let dx = (ax - x).max(0); x += dx; w -= dx;
        let dy = (ay - y).max(0); y += dy; h -= dy;
        w = w.min(ax + aw - x);
        h = h.min(ay + ah - 1 - y);
        fast_resize(hwnd, x, y, w, h);
    }
}

fn arrange_stacked() {
    let windows = get_app_windows();
    let n = windows.len();
    if n == 0 { return; }
    let fg = unsafe { GetForegroundWindow() };
    let (ax, ay, aw, ah) = get_work_rect(fg);
    let dx = 48_i32.min(aw / n as i32);
    let dy = 48_i32.min(ah / n as i32);
    for (k, &hwnd) in windows.iter().enumerate() {
        let x = ax + dx * k as i32;
        let y = ay + dy * (n - k - 1) as i32;
        let w = (aw / 2).max(aw - 2 * dx - (n as i32 - 2) * dx + dx);
        let h = (ah / 2).max(ah - 2 * dy - (n as i32 - 2) * dy + dy);
        fast_resize(hwnd, x, y, w, h);
    }
}
