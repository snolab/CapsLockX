//! Out-of-process preferences window (`clx prefs-window`).
//!
//! Runs as a SEPARATE process from the main hook/tray process. This is
//! required: a WebView2 (Chromium) window hosted in the SAME process as the
//! `WH_KEYBOARD_LL` hook makes Windows stop delivering the hook while that
//! window is focused — so clx's hotkeys went dead whenever the in-process prefs
//! window had focus (confirmed: a different-process Chromium window like Chrome
//! does NOT have this effect). Hosting the prefs UI in its own process — the
//! same approach as the macOS `clx-prompt` subprocess — keeps the hook process
//! free of any WebView2 window, so hotkeys keep working while prefs is open, and
//! closing it can't stall the hook thread either.
//!
//! Config is persisted to disk by `set_config`; the main process is notified via
//! the `CapsLockX_ConfigChanged` event so it can reload and apply live.

use tauri::webview::WebviewWindowBuilder;
use tauri::WebviewUrl;

use crate::config_store::{self, FullConfig};

#[tauri::command]
fn get_config() -> FullConfig {
    config_store::load()
}

#[tauri::command]
fn set_config(cfg: FullConfig) {
    config_store::save(&cfg);
    crate::shm::SharedState::signal_config_changed();
}

/// Entry point for the `prefs-window` subcommand. Blocks until the window closes.
pub fn run() {
    // Single-instance: if a prefs window is already open, focus it and exit so
    // a second "Preferences…" click doesn't spawn a duplicate window.
    if !acquire_single_instance() {
        focus_existing();
        return;
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_config, set_config])
        .setup(|app| {
            WebviewWindowBuilder::new(app.handle(), "prefs", WebviewUrl::App("index.html".into()))
                .title("CapsLockX Preferences")
                .inner_size(560.0, 640.0)
                .resizable(false)
                .center()
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("tauri build error (prefs window)")
        // Default behavior: closing the window ends this subprocess. We do NOT
        // prevent_exit here — unlike the main process, this one SHOULD exit when
        // its only window closes.
        .run(|_app, _event| {});
}

/// Acquire a process-wide named mutex. Returns `true` if we are the only prefs
/// instance, `false` if one is already running. The handle is intentionally
/// leaked so the mutex stays owned for the whole process lifetime.
fn acquire_single_instance() -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;
    unsafe {
        match CreateMutexW(None, true, w!("CapsLockX_PrefsWindow")) {
            Ok(handle) => {
                let already = GetLastError() == ERROR_ALREADY_EXISTS;
                std::mem::forget(handle);
                !already
            }
            Err(_) => true, // if the mutex can't be created, just proceed.
        }
    }
}

/// Bring an already-open "CapsLockX Preferences" window to the foreground.
fn focus_existing() {
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };
    unsafe {
        if let Ok(hwnd) = FindWindowW(PCWSTR::null(), w!("CapsLockX Preferences")) {
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}
