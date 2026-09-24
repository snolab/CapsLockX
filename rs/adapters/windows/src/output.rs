/// WinPlatform – implements the `Platform` trait using Win32 APIs.
///
/// Keyboard / mouse output: SendInput (tagged with CLX_EXTRA_INFO so our
/// own hook callback can skip self-injected events).
/// Window management: Win32 window enumeration + manipulation APIs.
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{CloseHandle, BOOL, COLORREF, HANDLE, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, OpenProcessToken,
    QueryFullProcessImageNameW, TerminateProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
    KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    MAPVK_VK_TO_VSC, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    MOUSE_EVENT_FLAGS, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, FindWindowW, GetClassNameW, GetForegroundWindow, GetWindowLongW,
    GetWindowTextLengthW, GetWindowThreadProcessId, IsHungAppWindow, IsWindowVisible,
    SendMessageTimeoutW, SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongW,
    SetWindowPos, ShowWindow, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, HWND_NOTOPMOST, HWND_TOPMOST,
    LWA_ALPHA, SMTO_ABORTIFHUNG, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SWP_SHOWWINDOW, SW_MINIMIZE, SW_RESTORE, WS_EX_LAYERED, WS_EX_TOPMOST,
};

use windows::core::{w, PCWSTR};

use crate::vd_api;
use crate::vk::keycode_to_vk;
use capslockx_core::platform::{ArrangeMode, MouseButton};
use capslockx_core::{KeyCode, Platform};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Magic tag on SendInput events so our hook callback skips them.
pub const CLX_EXTRA_INFO: usize = 0x434C_5800;

const WS_CAPTION_RAW: u32 = 0x00C0_0000;
const WS_EX_TOOLWINDOW_RAW: u32 = 0x0000_0080;
const WM_CLOSE: u32 = 0x0010;

// ── WinPlatform ───────────────────────────────────────────────────────────────

pub struct WinPlatform {
    /// HWND stored during V-hold transparent (0 = none).
    v_hwnd: AtomicUsize,
    /// Monotonic counter for unique prompt-result temp file names.
    prompt_seq: AtomicUsize,
    /// The window `cycle_windows` last aimed at (0 = none). Cycling resumes
    /// from here when the foreground is something we do not track — a Ghost,
    /// an elevated window, a transient dialog. Without it, "foreground not in
    /// list" rewound to the head of the list and cycling could never get past
    /// the offending window.
    last_cycle_target: AtomicUsize,
}

impl WinPlatform {
    pub fn new() -> Self {
        Self {
            last_cycle_target: AtomicUsize::new(0),
            v_hwnd: AtomicUsize::new(0),
            prompt_seq: AtomicUsize::new(0),
        }
    }
}

// ── SendInput helpers ────────────────────────────────────────────────────────

/// Nav-cluster / arrow / right-modifier VKs that require KEYEVENTF_EXTENDEDKEY
/// (they share scancodes with the numpad and need the E0 prefix).
fn is_extended_vk(vk: u16) -> bool {
    matches!(
        vk,
        0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 // PgUp PgDn End Home ← ↑ → ↓
            | 0x2D | 0x2E                                     // Insert Delete
            | 0x5B | 0x5C                                     // L/R Win
            | 0xA3 | 0xA5 | 0x90 // RCtrl RAlt NumLock
    )
}

/// Build a keyboard INPUT using the SCANCODE (KEYEVENTF_SCANCODE) like AHK's
/// SendEvent — Windows treats scancode injection more like a real hardware key,
/// which (unlike bare virtual-key injection) lets a physically-held Shift modify
/// our injected arrow without the OS "isolating" (lifting) the Shift.
fn kbd(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let mut f = flags | KEYEVENTF_SCANCODE;
    if is_extended_vk(vk) {
        f |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: f,
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(inputs, size_of::<INPUT>() as i32);
    }
}

/// Tap a raw virtual-key as an EXTENDED scancode event (down+up), tagged as
/// self-injected so our own hook skips it.
///
/// Used by the Alt+Tab enhancement (`alt_tab.rs`) to emit arrow keys,
/// media/volume keys, and Delete while the Windows task-switcher is focused.
/// Scancode injection (like AHK's `SendEvent`) lets the physically-held Alt
/// modify the injected key — exactly what the switcher's arrow-navigation
/// needs — and multimedia VKs only inject reliably as extended scancodes.
pub fn tap_vk_extended(vk: u16) {
    send(&[
        kbd(vk, KEYEVENTF_EXTENDEDKEY),
        kbd(vk, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
    ]);
}

/// True if `key` is currently physically down. Checks the distinguished VK
/// (e.g. VK_LSHIFT 0xA0) and falls back to the combined VK (VK_SHIFT 0x10 /
/// VK_CONTROL 0x11 / VK_MENU 0x12) — the distinguished left/right modifier VKs
/// can read as up via GetAsyncKeyState even while physically held.
fn modifier_held(key: KeyCode) -> bool {
    let vk = keycode_to_vk(key) as i32;
    if unsafe { GetAsyncKeyState(vk) < 0 } {
        return true;
    }
    let combined = match key {
        KeyCode::LShift | KeyCode::RShift => Some(0x10),
        KeyCode::LCtrl | KeyCode::RCtrl => Some(0x11),
        KeyCode::LAlt | KeyCode::RAlt => Some(0x12),
        _ => None,
    };
    combined
        .map(|c| unsafe { GetAsyncKeyState(c) < 0 })
        .unwrap_or(false)
}

fn mouse_inp(dx: i32, dy: i32, data: i32, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
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
    fn open_preferences(&self) {
        // Space+, toggles: pressing it again closes the window instead of
        // stacking another one. The tray menu still uses open_prefs_window().
        crate::toggle_prefs_window();
    }

    // ── Brainstorm (CLX+B) support ──────────────────────────────────────────

    fn get_clipboard_text(&self) -> String {
        arboard::Clipboard::new()
            .and_then(|mut c| c.get_text())
            .unwrap_or_default()
    }

    fn set_clipboard_text(&self, text: &str) {
        if let Ok(mut c) = arboard::Clipboard::new() {
            let _ = c.set_text(text.to_owned());
        }
    }

    /// Doesn't open the clipboard, so it can't collide with the app that's
    /// servicing our synthetic Ctrl+C the way a `get_clipboard_text` poll would.
    fn clipboard_sequence(&self) -> Option<u64> {
        use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
        Some(u64::from(unsafe { GetClipboardSequenceNumber() }))
    }

    fn show_brainstorm_overlay(&self, text: &str) {
        crate::overlay::show(text);
    }

    fn hide_brainstorm_overlay(&self) {
        crate::overlay::hide();
    }

    /// Launch the local-AI setup wizard: the prefs window opened in setup mode.
    fn open_brainstorm_setup(&self) {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .arg("prefs-window")
                .arg("--setup=brainstorm")
                .spawn();
        }
    }

    /// Show the brainstorm prompt as a SEPARATE process.
    ///
    /// Same rationale as the prefs window: a WebView2 window in the hook process
    /// kills WH_KEYBOARD_LL while focused. Prefers the native Slint dialog
    /// (`clx-prompt-slint.exe` next to clx.exe, ~150ms to first paint) and falls
    /// back to the Tauri/WebView2 one (`clx prompt-window`, ~1s until Chromium
    /// has rendered) when the native binary isn't there. Both take the same
    /// `<title> <message> <prefill> <out_path>` argv and write the user's input
    /// to that temp file (NOT stdout — WebView2 spawns Chromium helper processes
    /// that may inherit a stdout pipe and hang `.output()`); we wait via
    /// `.status()` then read the file. Absent/empty file = cancelled.
    fn show_prompt_input(&self, title: &str, message: &str, prefill: &str) -> Option<String> {
        let n = self.prompt_seq.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("clx-prompt-{}-{}.txt", std::process::id(), n));
        // Stale-safe: ensure no leftover file from a crashed prior run.
        let _ = std::fs::remove_file(&path);

        let exe = std::env::current_exe().ok()?;
        let native = exe.parent().map(|d| d.join("clx-prompt-slint.exe"));
        let mut cmd = match native.filter(|p| p.exists()) {
            Some(native) => std::process::Command::new(native),
            None => {
                let mut c = std::process::Command::new(exe);
                c.arg("prompt-window");
                c
            }
        };
        let status = cmd
            .arg(title)
            .arg(message)
            .arg(prefill)
            .arg(&path)
            // Lets the dialog self-terminate if clx restarts while it's open.
            .env("CLX_PARENT_PID", std::process::id().to_string())
            .status()
            .ok()?;
        let _ = status; // exit code is advisory; the file is the channel.

        let result = std::fs::read_to_string(&path).ok();
        let _ = std::fs::remove_file(&path);
        match result {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
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

    fn key_tap_n_with_mod(&self, mod_key: KeyCode, key: KeyCode, n: i32) {
        let mod_vk = keycode_to_vk(mod_key);
        let vk = keycode_to_vk(key);
        let n = n.clamp(0, 128) as usize;
        let mod_already_down = unsafe { GetAsyncKeyState(mod_vk as i32) < 0 };
        if mod_already_down {
            let mut inputs = Vec::with_capacity(n * 2);
            for _ in 0..n {
                inputs.push(kbd(vk, KEYBD_EVENT_FLAGS(0)));
                inputs.push(kbd(vk, KEYEVENTF_KEYUP));
            }
            send(&inputs);
        } else {
            let mut inputs = Vec::with_capacity(2 + n * 2);
            inputs.push(kbd(mod_vk, KEYBD_EVENT_FLAGS(0)));
            for _ in 0..n {
                inputs.push(kbd(vk, KEYBD_EVENT_FLAGS(0)));
                inputs.push(kbd(vk, KEYEVENTF_KEYUP));
            }
            inputs.push(kbd(mod_vk, KEYEVENTF_KEYUP));
            send(&inputs);
        }
    }

    fn key_tap_with_mods(&self, key: KeyCode, mods: &[KeyCode], n: i32) {
        let vk = keycode_to_vk(key);
        let n = n.clamp(0, 128) as usize;
        let mod_states: Vec<(u16, bool)> = mods
            .iter()
            .map(|m| (keycode_to_vk(*m), modifier_held(*m)))
            .collect();

        let mut inputs = Vec::with_capacity(mod_states.len() * 2 + n * 2);
        for (mvk, held) in &mod_states {
            if !held {
                inputs.push(kbd(*mvk, KEYBD_EVENT_FLAGS(0)));
            }
        }
        for _ in 0..n {
            inputs.push(kbd(vk, KEYBD_EVENT_FLAGS(0)));
            inputs.push(kbd(vk, KEYEVENTF_KEYUP));
        }
        for (mvk, held) in mod_states.iter().rev() {
            if !held {
                inputs.push(kbd(*mvk, KEYEVENTF_KEYUP));
            }
        }
        send(&inputs);
    }

    /// Reads the hardware state, so it stays correct even when our own hook
    /// has stopped being serviced — the case the Space auto-repeat loop in
    /// `engine.rs` has to survive.
    fn is_key_physically_down(&self, key: KeyCode) -> bool {
        modifier_held(key)
    }

    fn mouse_move(&self, dx: i32, dy: i32) {
        send(&[mouse_inp(dx, dy, 0, MOUSEEVENTF_MOVE.0)]);
    }
    fn scroll_v(&self, delta: i32) {
        // Core sends delta in pixels. Windows MOUSEEVENTF_WHEEL uses
        // WHEEL_DELTA (120) units where 120 = one notch ≈ 3 lines ≈ ~48px.
        // Scale: 1px → 120/48 = 2.5 wheel units.
        let wheel = (delta as f64 * 2.5) as i32;
        send(&[mouse_inp(
            0,
            0,
            wheel.clamp(-16384, 16384),
            MOUSEEVENTF_WHEEL.0,
        )]);
    }
    fn scroll_h(&self, delta: i32) {
        let wheel = (delta as f64 * 2.5) as i32;
        send(&[mouse_inp(
            0,
            0,
            wheel.clamp(-16384, 16384),
            MOUSEEVENTF_HWHEEL.0,
        )]);
    }
    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let flag = match (button, pressed) {
            (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN.0,
            (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP.0,
            (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN.0,
            (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP.0,
            _ => return,
        };
        send(&[mouse_inp(0, 0, 0, flag)]);
    }

    // ── Window management ─────────────────────────────────────────────────────

    /// Cycle windows in order:
    ///   1. Windows on the current monitor (first → last)
    ///   2. Next monitor's windows on the same virtual desktop
    ///   3. Next virtual desktop's first monitor's first window
    fn cycle_windows(&self, dir: i32) {
        let windows = get_app_windows();
        if windows.is_empty() {
            // Empty desktop: nothing to cycle, so Z / Shift+Z step to the
            // next / previous virtual desktop (and land on its first / last
            // window) instead of doing nothing.
            vd_api::step_desktop_and_focus(dir);
            return;
        }
        let fg = unsafe { GetForegroundWindow() };

        // Where to resume from. The foreground is usually one of ours; when it
        // is not — a Ghost window, an elevated app, a transient dialog we
        // filter out — carry on from the window we last aimed at instead of
        // rewinding to the head of the list. The rewind was what let cycling
        // orbit the same few windows and never get past the offending one.
        let pos = windows.iter().position(|&h| h == fg).or_else(|| {
            let last = self.last_cycle_target.load(Ordering::Relaxed);
            if last == 0 {
                None
            } else {
                windows.iter().position(|&h| h.0 as usize == last)
            }
        });

        let next = match pos {
            Some(i) => i as i32 + dir,
            None => {
                // No idea where we are: enter from the end we are travelling
                // away from, so this step lands on the first / last window.
                if dir > 0 {
                    0
                } else {
                    windows.len() as i32 - 1
                }
            }
        };

        if next < 0 || next as usize >= windows.len() {
            // Ran off the end: continue into the next / previous desktop and
            // land on its first / last window. The wrap is deliberate — one
            // closed loop across every desktop.
            vd_api::step_desktop_and_focus(dir);
            return;
        }

        let target = windows[next as usize];
        // Deliberately NOT checking the BOOL. `SetForegroundWindow` returns
        // FALSE in plenty of cases where it did the right thing — Windows'
        // foreground-lock rules refuse a process that did not handle the last
        // input, which is exactly clx's position when a low-level hook
        // swallowed the keystroke. Treating FALSE as "refused, try the next
        // one" made a single Z skip the whole list.
        unsafe {
            let _ = SetForegroundWindow(target);
        }
        // Remember it either way: the next press resumes from here even if the
        // foreground ended up somewhere we do not recognise.
        self.last_cycle_target
            .store(target.0 as usize, Ordering::Relaxed);
    }

    fn foreground_captures_input(&self) -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return false;
            }
            let mut cls = [0u16; 48];
            let n = GetClassNameW(hwnd, &mut cls).max(0) as usize;
            let class = String::from_utf16_lossy(&cls[..n.min(cls.len())]);
            if CAPTURING_CLASSES.iter().any(|c| *c == class) {
                return true;
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            process_captures_input(pid)
        }
    }

    /// A latched key's key-up never arrived. Record why.
    ///
    /// The usual cause is UIPI: a non-elevated `WH_KEYBOARD_LL` hook receives
    /// nothing at all while the foreground window belongs to a
    /// higher-integrity process — Task Manager, or the DWM-owned `Ghost`
    /// window raised in front of an unresponsive app.
    ///
    /// This writes to the log and nothing else. It deliberately does **not**
    /// pop anything up: the user is mid-keystroke in another window, and
    /// stealing focus to explain that we lost focus is worse than the problem.
    fn on_input_lost(&self) {
        // Rate-limit: one line per 30 s, however many models trip at once.
        use std::sync::atomic::AtomicU64;
        static LAST_REPORT: AtomicU64 = AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = LAST_REPORT.load(Ordering::Relaxed);
        if now.saturating_sub(last) < 30 {
            return;
        }
        LAST_REPORT.store(now, Ordering::Relaxed);

        let (desc, blocked) = describe_foreground_block();
        crate::hook::crash_log_sync(&format!(
            "[on_input_lost] key-up never arrived; foreground={} blocked={}              (clx elevated: {})",
            desc,
            blocked,
            crate::is_elevated()
        ));
    }

    /// Escape hatch from a full-screen RDP / VM window (double-tap
    /// LCtrl+LAlt+LShift — see `capslockx_core::modules::rdp_escape`).
    ///
    /// Sends the focused window to the bottom of the Z-order *without*
    /// minimizing it — the remote session keeps running underneath — then
    /// activates the taskbar, which is what actually breaks `mstsc`'s
    /// keyboard grab. AHK did the same via `WinSet Bottom` +
    /// `WinActivate ahk_class Shell_TrayWnd`.
    fn send_active_window_to_back(&self) {
        // Off the hook thread. This attaches input queues and steals the
        // foreground, both of which are input-synchronous operations that must
        // never run inside the `WH_KEYBOARD_LL` callback — the same hazard that
        // forced virtual-desktop COM onto its own worker.
        std::thread::spawn(|| {
            let r = std::panic::catch_unwind(escape_foreground_window);
            if r.is_err() {
                crate::hook::crash_log_sync("[PANIC] recovered in escape_foreground_window");
            }
        });
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
        match mode {
            ArrangeMode::SideBySide => arrange_side_by_side(),
            ArrangeMode::Stacked => arrange_stacked(),
        }
    }

    fn close_tab(&self) {
        self.key_tap_cmd_or_ctrl(KeyCode::W);
    }

    fn close_window(&self) {
        let hwnd = unsafe { GetForegroundWindow() };
        self.cycle_windows(1);
        unsafe {
            // SendMessageW is synchronous: against a window whose thread has
            // stopped pumping (TeraCopy mid-copy, anything showing a Ghost) it
            // never returns, and since this runs on a thread spawned per press
            // every Shift+X would strand one forever. SMTO_ABORTIFHUNG gives
            // up instead.
            let _ = SendMessageTimeoutW(
                hwnd,
                WM_CLOSE,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                2000,
                None,
            );
        }
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
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    fn restore_window(&self) {
        let raw = self.v_hwnd.swap(0, Ordering::Relaxed);
        if raw != 0 {
            let hwnd = HWND(raw as *mut _);
            set_layered_alpha(hwnd, 255);
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    fn toggle_window_topmost(&self) {
        let hwnd = unsafe { GetForegroundWindow() };
        let exstyle = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 };
        unsafe {
            if exstyle & WS_EX_TOPMOST.0 != 0 {
                set_layered_alpha(hwnd, 255);
                let _ = SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            } else {
                set_layered_alpha(hwnd, 200);
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    // ── Virtual desktop ────────────────────────────────────────────────────────

    // Both desktop methods run inside the WH_KEYBOARD_LL callback, where COM
    // calls are refused (see vd_api module docs). They only post to the
    // clx-vd-worker thread, which does COM-first / hotkey-fallback itself.

    fn switch_to_desktop(&self, idx: u32) {
        vd_api::switch_desktop(idx.clamp(1, 10) as usize);
    }

    fn restart(&self) {
        // Spawn a new instance of ourselves, then exit.
        if let Ok(exe) = std::env::current_exe() {
            let wd =
                std::env::current_dir().unwrap_or_else(|_| exe.parent().unwrap().to_path_buf());
            let _ = std::process::Command::new(&exe).current_dir(wd).spawn();
        }
        // Not `process::exit` — that is the ExitProcess path that leaves an
        // unkillable zombie still holding the keyboard hook, which would then
        // starve the instance we just spawned. See `main::hard_exit`.
        crate::hard_exit(0);
    }

    fn move_window_to_desktop(&self, idx: u32) {
        // Capture the foreground window *now*, on the hook thread, so the
        // worker moves the window the user was actually looking at.
        let hwnd = unsafe { GetForegroundWindow() };
        vd_api::move_window_to_desktop(hwnd.0 as isize, idx.clamp(1, 10) as usize);
    }
}

// ── Window enumeration ────────────────────────────────────────────────────────

extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // `extern "system"` callback invoked by EnumWindows: a panic unwinding out of
    // here would abort the process. Catch it and stop enumeration (BOOL(0)) with a
    // partial list rather than crashing.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enum_callback_inner(hwnd, lparam)
    }))
    .unwrap_or(BOOL(0))
}

fn enum_callback_inner(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if style & WS_CAPTION_RAW == 0 {
            return BOOL(1);
        }
        if exstyle & WS_EX_TOOLWINDOW_RAW != 0 {
            return BOOL(1);
        }
        if GetWindowTextLengthW(hwnd) == 0 {
            return BOOL(1);
        }
        // Windows CLX cannot drive are dropped here rather than skipped by each
        // caller, so cycling, tiling and cross-desktop focus all agree that they
        // simply are not there.
        if window_is_unreachable(hwnd) {
            return BOOL(1);
        }

        // Skip cloaked windows (e.g. UWP apps on other virtual desktops).
        let mut cloaked: u32 = 0;
        let _ = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if cloaked != 0 {
            return BOOL(1);
        }
        (&mut *(lparam.0 as *mut Vec<HWND>)).push(hwnd);
        BOOL(1)
    }
}

/// Get out of a session that has swallowed the keyboard, and hand the host
/// machine back to the user.
///
/// Sending the window to the bottom is not enough on its own: a full-screen
/// RDP keeps the foreground, so every keystroke still travels to the guest and
/// the user is no better off. The gesture has to actually move focus somewhere
/// they can work.
///
/// Three steps, each verified rather than assumed:
///
/// 1. Drop the offender to the bottom of the Z-order, without activating
///    anything (that is the part that already behaved correctly).
/// 2. Give the foreground to the best remaining window — the topmost one from
///    `get_app_windows()`, which already excludes windows CLX cannot drive, so
///    this lands the user on something usable rather than on the taskbar.
///    `SetForegroundWindow` is refused unless the caller owns the foreground or
///    handled the last input, and CLX's hook *swallowed* the keystroke that got
///    us here, so it qualifies for neither: the input queues are briefly
///    attached to borrow that right, the way every focus-stealing tool has to.
/// 3. **Check whether it worked**, and if the offender still holds the
///    foreground, minimize it. Minimizing needs no activation rights at all, so
///    it always succeeds — the window vanishes rather than merely sinking, which
///    is a worse outcome than step 2 but strictly better than being stuck.
fn escape_foreground_window() {
    unsafe {
        let stuck = GetForegroundWindow();
        if stuck.0.is_null() {
            return;
        }

        let _ = SetWindowPos(
            stuck,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        // Prefer a real window to return to; fall back to the shell.
        let target = get_app_windows()
            .into_iter()
            .find(|&h| h != stuck)
            .or_else(|| match FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) {
                Ok(tray) if !tray.0.is_null() => Some(tray),
                _ => None,
            });

        if let Some(target) = target {
            let our_thread = GetCurrentThreadId();
            let their_thread = GetWindowThreadProcessId(stuck, None);
            let attached = their_thread != 0
                && their_thread != our_thread
                && AttachThreadInput(our_thread, their_thread, BOOL(1)).as_bool();

            let _ = SetForegroundWindow(target);
            let _ = BringWindowToTop(target);

            if attached {
                let _ = AttachThreadInput(our_thread, their_thread, BOOL(0));
            }
        }

        // Verify. The foreground change is synchronous once granted, but a
        // short settle avoids reading the state mid-switch.
        std::thread::sleep(std::time::Duration::from_millis(120));
        if GetForegroundWindow() == stuck {
            crate::hook::crash_log_sync(
                "[escape] foreground refused to move — minimizing the window instead",
            );
            let _ = ShowWindow(stuck, SW_MINIMIZE);
        }
    }
}

/// Window classes belonging to sessions that forward the whole keyboard to a
/// guest. `TscShellContainerClass` is mstsc — the reported case, and the one a
/// cross-desktop step lands on when a full-screen RDP owns that desktop.
const CAPTURING_CLASSES: &[&str] = &[
    "TscShellContainerClass", // mstsc — Remote Desktop Connection (verified)
    "RAIL_WINDOW",            // RemoteApp
    "VMPlayerFrame",          // VMware Player
    "VMUIFrame",              // VMware Workstation
    "VMConnectMainWindow",    // Hyper-V virtual machine connection
    "VBoxSDL",                // VirtualBox, SDL front end
];

/// VirtualBox is the awkward one: its main window is a Qt widget whose class is
/// a generic `Qt<version>QWindowIcon`-style name that changes between builds and
/// is shared with every other Qt application on the machine, so matching on it
/// would be both fragile and far too broad. It is caught by executable name in
/// `CAPTURING_EXES` instead. Hyper-V's `vmconnect` is really an RDP client under
/// the skin and is expected to behave exactly like mstsc — both are listed, and
/// both still want a live check.

/// Executables whose windows capture the keyboard but whose class names are too
/// generic to match on (Qt and Electron shells, mostly). Cached per PID like
/// the elevation check — this is consulted once per physics tick.
fn process_captures_input(pid: u32) -> bool {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: std::sync::OnceLock<Mutex<HashMap<u32, bool>>> = std::sync::OnceLock::new();
    const CAPTURING_EXES: &[&str] = &[
        "mstsc.exe",      // Remote Desktop Connection
        "msrdc.exe",      // Windows App / Remote Desktop client
        "vmconnect.exe",  // Hyper-V VM connection
        "vmware-vmx.exe", // VMware
        "vmware.exe",
        "vmplayer.exe",
        "virtualbox.exe",   // VirtualBox manager, when a VM is embedded
        "virtualboxvm.exe", // VirtualBox VM window — the usual one
        "vboxsdl.exe",
        "vmconnect.exe",
    ];
    if pid == 0 {
        return false;
    }
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(&known) = cache.lock().unwrap().get(&pid) {
        return known;
    }
    let captures = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, BOOL(0), pid)
            .map(|process| {
                let mut buf = [0u16; 260];
                let mut len = buf.len() as u32;
                let ok = QueryFullProcessImageNameW(
                    process,
                    PROCESS_NAME_WIN32,
                    windows::core::PWSTR(buf.as_mut_ptr()),
                    &mut len,
                )
                .is_ok();
                let _ = CloseHandle(process);
                if !ok {
                    return false;
                }
                let path = String::from_utf16_lossy(&buf[..len as usize]).to_ascii_lowercase();
                let exe = path.rsplit('\\').next().unwrap_or("").to_string();
                CAPTURING_EXES.iter().any(|e| *e == exe)
            })
            .unwrap_or(false)
    };
    let mut map = cache.lock().unwrap();
    if map.len() > 512 {
        map.clear();
    }
    map.insert(pid, captures);
    captures
}

/// Is `pid` running at an integrity level we cannot drive?
///
/// Cached per PID: a process does not change elevation during its lifetime, and
/// this is consulted for every window on every cycle tick — up to a few hundred
/// times a second — so the token query must happen once, not per call.
///
/// A process we cannot even open a token on (dwm, anything protected) counts as
/// unreachable, which is the right answer for the case that matters: the DWM
/// `Ghost` window's owner.
fn process_is_unreachable(pid: u32) -> bool {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: std::sync::OnceLock<Mutex<HashMap<u32, bool>>> = std::sync::OnceLock::new();

    if pid == 0 {
        return true;
    }
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(&known) = cache.lock().unwrap().get(&pid) {
        return known;
    }

    let unreachable = unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, BOOL(0), pid) {
            Err(_) => true, // cannot even ask — protected or higher integrity
            Ok(process) => {
                let mut token = HANDLE::default();
                let verdict = if OpenProcessToken(process, TOKEN_QUERY, &mut token).is_ok() {
                    let mut elevation = TOKEN_ELEVATION::default();
                    let mut len = 0u32;
                    let ok = GetTokenInformation(
                        token,
                        TokenElevation,
                        Some(&mut elevation as *mut _ as *mut _),
                        std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                        &mut len,
                    )
                    .is_ok();
                    let _ = CloseHandle(token);
                    // `PROCESS_QUERY_LIMITED_INFORMATION` is deliberately
                    // permissive and succeeds against ordinary elevated apps, so
                    // the open alone proves nothing — the token is the answer.
                    ok && elevation.TokenIsElevated != 0
                } else {
                    true
                };
                let _ = CloseHandle(process);
                verdict
            }
        }
    };

    let mut map = cache.lock().unwrap();
    // PIDs churn; keep the table from growing without bound over a long session.
    if map.len() > 512 {
        map.clear();
    }
    map.insert(pid, unreachable);
    unreachable
}

/// Should this window be invisible to every window-management feature?
///
/// Three kinds, and the rule is the same for all of them: CLX cannot usefully
/// drive them, and focusing one costs us the keyboard hook, so they are treated
/// as though they do not exist — not skipped in cycling only, but absent from
/// the single list that cycling, tiling and cross-desktop focus all read.
///
/// 1. the DWM `Ghost` stand-in raised in front of an app that stopped responding
/// 2. the unresponsive app itself — focusing it just summons the ghost
/// 3. higher-integrity windows, *when clx is not elevated*. An elevated clx can
///    drive them perfectly well, so it must not hide them from itself.
unsafe fn window_is_unreachable(hwnd: HWND) -> bool {
    let mut cls = [0u16; 16];
    let n = GetClassNameW(hwnd, &mut cls);
    if n == 5 && String::from_utf16_lossy(&cls[..5]) == "Ghost" {
        return true;
    }
    if IsHungAppWindow(hwnd).as_bool() {
        return true;
    }
    static SELF_ELEVATED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *SELF_ELEVATED.get_or_init(crate::is_elevated) {
        return false; // we outrank everything an ordinary user can run
    }
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    process_is_unreachable(pid)
}

/// Enumerate visible app windows, ordered by monitor index then HWND value.
///
/// Cycling order: current monitor's windows → next monitor → … → desktop switch.
pub(crate) fn get_app_windows() -> Vec<HWND> {
    let mut v: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut v as *mut Vec<HWND> as isize),
        );
    }
    // Sort by (monitor_index, hwnd) so cycling goes through all windows on one
    // monitor before moving to the next.  Within a monitor, HWND order is stable
    // (doesn't shift on focus change, unlike Z-order from EnumWindows).
    v.sort_by_key(|&h| {
        let mon = unsafe { MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST) };
        (mon.0 as usize, h.0 as usize)
    });
    v
}

/// Describe the foreground window, and say whether it is one our hook cannot
/// see past.
///
/// "Cannot see past" is `process_is_unreachable`: the process token's
/// `TokenElevation`, plus "we could not open it at all" for protected
/// processes. It is emphatically *not* "OpenProcess failed" —
/// `PROCESS_QUERY_LIMITED_INFORMATION` is deliberately permissive and succeeds
/// against ordinary elevated apps (measured: `fg_query=True fg_elevated=1`),
/// which is why the earlier version of this function reported `blocked=false`
/// for every window that was genuinely blocking us. The DWM `Ghost` window is
/// called out by name because it is the common case and the useful thing to
/// say.
fn describe_foreground_block() -> (String, bool) {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return ("(none)".into(), false);
        }
        let mut cls = [0u16; 64];
        let n = GetClassNameW(hwnd, &mut cls).max(0) as usize;
        let class = String::from_utf16_lossy(&cls[..n.min(cls.len())]);
        let hung = IsHungAppWindow(hwnd).as_bool();

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        // Was: "OpenProcess failed ⇒ higher integrity". That inference is wrong —
        // PROCESS_QUERY_LIMITED_INFORMATION succeeds against ordinary elevated
        // apps (measured: fg_query=True while fg_elevated=1), so it reported
        // blocked=false for every genuinely blocked window. Ask the token.
        let opaque = process_is_unreachable(pid);

        let mut label = if class == "Ghost" {
            format!("{} (无响应程序的替身窗口 / DWM ghost)", class)
        } else {
            class
        };
        if hung {
            label.push_str(" [无响应 / not responding]");
        }
        (label, opaque)
    }
}

// ── Virtual desktop hotkey fallback (Win+Ctrl+Arrow) ─────────────────────────
//
// Both helpers are called from the clx-vd-worker thread when the COM API is
// unavailable; the hook itself never sends these.

/// Blind single step in `dir` direction (+1 or -1) via Win+Ctrl+Arrow.
pub(crate) fn navigate_desktops_step(dir: i32) {
    const VK_LWIN: u16 = 0x5B;
    const VK_LCTRL: u16 = 0xA2;
    const VK_LEFT: u16 = 0x25;
    const VK_RIGHT: u16 = 0x27;
    let vk_dir = if dir > 0 { VK_RIGHT } else { VK_LEFT };
    send(&[
        kbd(VK_LWIN, KEYBD_EVENT_FLAGS(0)),
        kbd(VK_LCTRL, KEYBD_EVENT_FLAGS(0)),
        kbd(vk_dir, KEYBD_EVENT_FLAGS(0)),
        kbd(vk_dir, KEYEVENTF_KEYUP),
        kbd(VK_LCTRL, KEYEVENTF_KEYUP),
        kbd(VK_LWIN, KEYEVENTF_KEYUP),
    ]);
}

/// Navigate from desktop `from` to desktop `to` by sending Win+Ctrl+Left/Right.
pub(crate) fn navigate_desktops(from: usize, to: usize) {
    // VK codes
    const VK_LWIN: u16 = 0x5B;
    const VK_LCTRL: u16 = 0xA2;
    const VK_LEFT: u16 = 0x25;
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
        let _ = SetWindowPos(
            hwnd,
            HWND(std::ptr::null_mut()),
            x,
            y,
            w,
            h,
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
        let dx = (ax - x).max(0);
        x += dx;
        w -= dx;
        let dy = (ay - y).max(0);
        y += dy;
        h -= dy;
        w = w.min(ax + aw - x);
        h = h.min(ay + ah - 1 - y);
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
    let dx = 72_i32.min(aw / n as i32);
    let dy = 32_i32.min(ah / n as i32);
    let w = (aw / 2).max(aw - 2 * dx - (n as i32 - 2) * dx + dx);
    let h = (ah / 2).max(ah - 2 * dy - (n as i32 - 2) * dy + dy);

    // Resize all windows.
    for (k, &hwnd) in windows.iter().enumerate() {
        let x = ax + dx * k as i32;
        let y = ay + dy * (n - k - 1) as i32;
        fast_resize(hwnd, x, y, w, h);
    }

    // Z-order: card deck fan-out from current window.
    // Current = topmost, neighbors next, farthest = bottom.
    // Example: [0,1,2,3,4] current=2 → raise order: 0,4,1,3,2
    let current_idx = windows.iter().position(|&h| h == fg).unwrap_or(0);
    let mut z_order: Vec<usize> = (0..n).collect();
    z_order.sort_by(|&a, &b| {
        let da = (a as isize - current_idx as isize).unsigned_abs();
        let db = (b as isize - current_idx as isize).unsigned_abs();
        db.cmp(&da)
    });
    unsafe {
        for &idx in &z_order {
            let hwnd = windows[idx];
            let _ = SetWindowPos(
                hwnd,
                HWND(std::ptr::null_mut()),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
        // Restore focus to the current window.
        let _ = SetForegroundWindow(fg);
    }
}
