//! clx-prefs (Slint) — CapsLockX preferences window.
//!
//! Native, fast-starting replacement for the WebView2/Tauri prefs window (which
//! pays a ~1-2s Chromium cold start on every open). Reads/writes the same
//! `%APPDATA%\CapsLockX\config.json` (macOS: ~/Library/... via dirs) as the rest
//! of CLX, preserving any fields it doesn't surface, and signals the running CLX
//! to hot-reload via the `CapsLockX_ConfigChanged` event (Windows).
//!
//! Argv: `clx-prefs-slint [version-footer-text]`

#![cfg_attr(windows, windows_subsystem = "windows")]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use serde_json::{json, Value};

slint::include_modules!();

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("CapsLockX")
        .join("config.json")
}

fn load_config() -> Value {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn cfg_bool(v: &Value, k: &str, d: bool) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(d)
}
fn cfg_f32(v: &Value, k: &str, d: f32) -> f32 {
    v.get(k)
        .and_then(Value::as_f64)
        .map(|x| x as f32)
        .unwrap_or(d)
}

fn save_config(cfg: &Value) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, s);
    }
    signal_config_changed();
}

/// Poke the running CLX to reload config.json live (matches
/// `SharedState::signal_config_changed` in the Windows adapter).
#[cfg(windows)]
fn signal_config_changed() {
    use windows::core::w;
    use windows::Win32::System::Threading::{OpenEventW, SetEvent, EVENT_MODIFY_STATE};
    unsafe {
        if let Ok(evt) = OpenEventW(EVENT_MODIFY_STATE, false, w!("CapsLockX_ConfigChanged")) {
            let _ = SetEvent(evt);
        }
    }
}
#[cfg(not(windows))]
fn signal_config_changed() {}

/// Exit this prefs process if the parent CLX (PID in `CLX_PARENT_PID`) exits, so
/// a prefs window can never orphan and linger when clx is killed / self-updates.
#[cfg(windows)]
fn exit_with_parent() {
    let Some(pid) = std::env::var("CLX_PARENT_PID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
    else {
        return;
    };
    std::thread::spawn(move || unsafe {
        use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::{
            OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
        };
        if let Ok(h) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            // Blocks until the parent process exits, then tears us down too.
            if WaitForSingleObject(h, u32::MAX) == WAIT_OBJECT_0 {
                std::process::exit(0);
            }
            let _ = CloseHandle(h);
        }
    });
}
#[cfg(not(windows))]
fn exit_with_parent() {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    exit_with_parent();

    let version = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "CapsLockX".into());

    let cfg = Rc::new(RefCell::new(load_config()));
    let win = PrefsWindow::new()?;

    {
        let c = cfg.borrow();
        win.set_use_capslock(cfg_bool(&c, "use_capslock", true));
        win.set_use_space(cfg_bool(&c, "use_space", true));
        win.set_use_insert(cfg_bool(&c, "use_insert", false));
        win.set_use_scroll_lock(cfg_bool(&c, "use_scroll_lock", false));
        win.set_use_ralt(cfg_bool(&c, "use_ralt", false));
        win.set_cursor_speed(cfg_f32(&c, "cursor_speed", 60.0));
        win.set_mouse_speed(cfg_f32(&c, "mouse_speed", 5900.0));
        win.set_scroll_speed(cfg_f32(&c, "scroll_speed", 1500.0));
        win.set_version_text(version.into());
    }

    // Launch at login is NOT part of config.json — the Task Scheduler entry is
    // the source of truth, so read it live and never persist a mirror flag.
    win.set_autostart_supported(cfg!(windows));
    win.set_autostart(clx_autostart::is_enabled());
    win.set_autostart_hint(autostart_hint().into());

    // Auto-apply on any change: fold the UI state back into the (preserved)
    // config object and write + signal.
    {
        let cfg = cfg.clone();
        let win_weak = win.as_weak();
        win.on_changed(move || {
            let Some(w) = win_weak.upgrade() else { return };
            let mut c = cfg.borrow_mut();
            c["use_capslock"] = json!(w.get_use_capslock());
            c["use_space"] = json!(w.get_use_space());
            c["use_insert"] = json!(w.get_use_insert());
            c["use_scroll_lock"] = json!(w.get_use_scroll_lock());
            c["use_ralt"] = json!(w.get_use_ralt());
            c["cursor_speed"] = json!(w.get_cursor_speed() as f64);
            c["mouse_speed"] = json!(w.get_mouse_speed() as f64);
            c["scroll_speed"] = json!(w.get_scroll_speed() as f64);
            save_config(&c);
        });
    }

    // Toggling autostart can raise a UAC prompt, so drive the checkbox from what
    // the system actually ends up as rather than from what was requested.
    {
        let win_weak = win.as_weak();
        win.on_autostart_toggled(move |want| {
            let actual = clx_autostart::set_enabled(want);
            if let Some(w) = win_weak.upgrade() {
                w.set_autostart(actual);
            }
        });
    }

    win.run()?;
    Ok(())
}

/// Caption under the checkbox: which binary the logon task starts, so it's
/// obvious when prefs is running next to a different clx.exe than you expect.
fn autostart_hint() -> String {
    match clx_autostart::target_exe() {
        Ok(p) => format!(
            "Runs {} at logon, elevated. Task Scheduler entry: {}",
            p.display(),
            clx_autostart::TASK_NAME
        ),
        Err(_) => String::new(),
    }
}
