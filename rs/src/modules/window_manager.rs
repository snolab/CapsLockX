//! CLX-WindowManager: window cycling, tiling, close/kill, transparency, topmost.
//!
//! Key bindings (in CLX mode):
//!   z / Shift+z        → cycle windows forward / backward
//!   x                  → close tab (Ctrl+W)
//!   Shift+x            → close window and cycle to next
//!   Ctrl+Alt+x         → kill process and cycle to next
//!   c / Shift+c        → arrange stacked / side-by-side
//!   v (hold) / v Up    → temporary transparent + topmost / restore
//!   Shift+v            → toggle always-on-top + transparent

use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{BOOL, CloseHandle, COLORREF, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsWindowVisible, SendMessageW, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
    HWND_NOTOPMOST, HWND_TOPMOST, LWA_ALPHA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SW_RESTORE, WS_EX_LAYERED, WS_EX_TOPMOST,
};
use crate::input;
use crate::vk::*;

// Raw bit masks (avoid typed-wrapper gymnastics in comparisons)
const WS_CAPTION_RAW: u32      = 0x00C0_0000;
const WS_EX_TOOLWINDOW_RAW: u32 = 0x0000_0080;
const WM_CLOSE: u32            = 0x0010;

/// HWND of window made transparent by V-hold (0 = none)
static V_HWND: AtomicUsize = AtomicUsize::new(0);

// ── Public API ────────────────────────────────────────────────────────────────

pub fn on_key_down(vk: u32) -> bool {
    let shift = is_vk_down(VK_SHIFT);
    let ctrl  = is_vk_down(VK_LCONTROL) || is_vk_down(VK_RCONTROL);
    let alt   = is_vk_down(VK_LMENU)    || is_vk_down(VK_RMENU);

    match vk {
        VK_Z => {
            if shift { cycle_windows(-1) } else { cycle_windows(1) }
            true
        }
        VK_X => {
            if ctrl && alt { kill_window_and_cycle() }
            else if shift  { close_window_and_cycle() }
            else           { close_tab() }
            true
        }
        VK_C => {
            if shift { arrange_side_by_side() } else { arrange_stacked() }
            true
        }
        VK_V => {
            if shift { toggle_topmost() } else { set_transparent_hold() }
            true
        }
        _ => false,
    }
}

pub fn on_key_up(vk: u32) -> bool {
    if vk == VK_V {
        clear_transparent_hold();
        return true;
    }
    false
}

pub fn is_mapped_key(vk: u32) -> bool {
    matches!(vk, VK_Z | VK_X | VK_C | VK_V)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[inline]
fn is_vk_down(vk: u32) -> bool {
    unsafe { (GetKeyState(vk as i32) as u16) & 0x8000 != 0 }
}

// ── Window enumeration ────────────────────────────────────────────────────────

extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let style   = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

        // Must have a caption (title bar)
        if style & WS_CAPTION_RAW == 0 {
            return BOOL(1);
        }
        // Skip tool windows
        if exstyle & WS_EX_TOOLWINDOW_RAW != 0 {
            return BOOL(1);
        }
        // Must have a non-empty title
        if GetWindowTextLengthW(hwnd) == 0 {
            return BOOL(1);
        }

        // Skip ApplicationFrameWindow (invisible UWP host windows)
        let mut class = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class) as usize;
        let class_str = String::from_utf16_lossy(&class[..len.min(class.len())]);
        if class_str == "ApplicationFrameWindow" {
            return BOOL(1);
        }

        let vec = &mut *(lparam.0 as *mut Vec<HWND>);
        vec.push(hwnd);
        BOOL(1)
    }
}

fn get_app_windows() -> Vec<HWND> {
    let mut windows: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut windows as *mut Vec<HWND> as isize),
        );
    }
    windows
}

// ── Cycle windows ─────────────────────────────────────────────────────────────

fn cycle_windows(dir: i32) {
    let windows = get_app_windows();
    if windows.is_empty() {
        return;
    }

    let fg = unsafe { GetForegroundWindow() };
    let pos = windows.iter().position(|&h| h == fg);

    let target = match pos {
        None => windows[0],
        Some(idx) => {
            let n = windows.len() as i32;
            let next = ((idx as i32 + dir).rem_euclid(n)) as usize;
            windows[next]
        }
    };

    unsafe {
        let _ = SetForegroundWindow(target);
    }
}

// ── Arrange windows ───────────────────────────────────────────────────────────

fn get_work_rect(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
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
        let _ = SetWindowPos(
            hwnd,
            HWND(std::ptr::null_mut()),
            x, y, w, h,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
        );
    }
}

fn arrange_side_by_side() {
    let windows = get_app_windows();
    let n = windows.len();
    if n == 0 {
        return;
    }

    let fg = unsafe { GetForegroundWindow() };
    let (ax, ay, aw, ah) = get_work_rect(fg);

    // Shorter edge gets fewer splits
    let (rows, cols) = if aw <= ah {
        let c = (n as f64).sqrt() as usize;
        let c = c.max(1);
        let r = (n + c - 1) / c;
        (r, c)
    } else {
        let r = (n as f64).sqrt() as usize;
        let r = r.max(1);
        let c = (n + r - 1) / r;
        (r, c)
    };

    for (k, &hwnd) in windows.iter().enumerate() {
        let nx = (k % cols) as i32;
        let ny = (k / cols) as i32;
        let sw = aw / cols as i32;
        let sh = ah / rows as i32;

        // Overlap slightly to hide window borders
        let mut x = ax + nx * sw - 8;
        let mut y = ay + ny * sh;
        let mut w = sw + 16;
        let mut h = sh + 8;

        // Clamp to work area
        let dx = (ax - x).max(0); x += dx; w -= dx;
        let dy = (ay - y).max(0); y += dy; h -= dy;
        w = w.min(ax + aw - x);
        h = h.min(ay + ah - 1 - y); // leave 1px at bottom

        fast_resize(hwnd, x, y, w, h);
    }
}

fn arrange_stacked() {
    let windows = get_app_windows();
    let n = windows.len();
    if n == 0 {
        return;
    }

    let fg = unsafe { GetForegroundWindow() };
    let (ax, ay, aw, ah) = get_work_rect(fg);

    let dx = 48_i32.min(aw / n as i32);
    let dy = 48_i32.min(ah / n as i32);

    // Cascade: left-bottom to right-top
    for (k, &hwnd) in windows.iter().enumerate() {
        let x = ax + dx * k as i32;
        let y = ay + dy * (n - k - 1) as i32;
        let w = (aw / 2).max(aw - 2 * dx - (n as i32 - 2) * dx + dx);
        let h = (ah / 2).max(ah - 2 * dy - (n as i32 - 2) * dy + dy);
        fast_resize(hwnd, x, y, w, h);
    }
}

// ── Close / kill ──────────────────────────────────────────────────────────────

fn close_tab() {
    // Send Ctrl+W (close active tab in browsers, editors, etc.)
    input::key_down(VK_LCONTROL);
    input::tap_key(VK_W);
    input::key_up(VK_LCONTROL);
}

fn close_window_and_cycle() {
    let hwnd = unsafe { GetForegroundWindow() };
    cycle_windows(1);
    unsafe {
        SendMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}

fn kill_window_and_cycle() {
    let hwnd = unsafe { GetForegroundWindow() };
    cycle_windows(1);
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid != 0 {
            if let Ok(proc_handle) = OpenProcess(PROCESS_TERMINATE, BOOL(0), pid) {
                let _ = TerminateProcess(proc_handle, 1);
                let _ = CloseHandle(proc_handle);
            }
        }
    }
}

// ── Transparency ──────────────────────────────────────────────────────────────

fn set_layered_alpha(hwnd: HWND, alpha: u8) {
    unsafe {
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if exstyle & WS_EX_LAYERED.0 == 0 {
            SetWindowLongW(hwnd, GWL_EXSTYLE, (exstyle | WS_EX_LAYERED.0) as i32);
        }
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

fn set_transparent_hold() {
    let hwnd = unsafe { GetForegroundWindow() };
    V_HWND.store(hwnd.0 as usize, Ordering::Relaxed);
    set_layered_alpha(hwnd, 100);
    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

fn clear_transparent_hold() {
    let raw = V_HWND.swap(0, Ordering::Relaxed);
    if raw != 0 {
        let hwnd = HWND(raw as *mut _);
        set_layered_alpha(hwnd, 255);
        unsafe {
            let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }
}

// ── Always-on-top toggle ──────────────────────────────────────────────────────

fn toggle_topmost() {
    let hwnd = unsafe { GetForegroundWindow() };
    let exstyle = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 };
    unsafe {
        if exstyle & WS_EX_TOPMOST.0 != 0 {
            // Remove topmost and restore full opacity
            let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            set_layered_alpha(hwnd, 255);
        } else {
            // Set topmost with slight transparency
            set_layered_alpha(hwnd, 200);
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }
}
