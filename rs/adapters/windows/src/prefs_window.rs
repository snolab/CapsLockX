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

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::webview::WebviewWindowBuilder;
use tauri::{Emitter, WebviewUrl};

use crate::config_store::{self, FullConfig};

/// True when launched as `prefs-window --setup=brainstorm` so the UI opens
/// straight into the local-AI wizard.
static SETUP_MODE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn get_config() -> FullConfig {
    config_store::load()
}

#[tauri::command]
fn set_config(cfg: FullConfig) {
    config_store::save(&cfg);
    crate::shm::SharedState::signal_config_changed();
}

#[tauri::command]
fn is_setup_mode() -> bool {
    SETUP_MODE.load(Ordering::Relaxed)
}

/// Detected local-AI status + hardware + recommended model for the wizard.
#[derive(serde::Serialize)]
struct BsStatus {
    /// "ready" | "running_no_model" | "down" | "not_installed"
    status: String,
    ram_gb: u32,
    vram_gb: u32,
    cpu_cores: u32,
    recommended: String,
    installed: bool,
    current_model: String,
}

#[tauri::command]
fn bs_detect() -> BsStatus {
    use capslockx_core::local_llm::{self, LocalLlmStatus};
    let hw = local_llm::probe_hardware();
    let st = local_llm::local_status();
    let status = match st {
        LocalLlmStatus::Ready => "ready",
        LocalLlmStatus::RunningNoModel => "running_no_model",
        LocalLlmStatus::OllamaDownNotRunning => "down",
        LocalLlmStatus::NotInstalled => "not_installed",
    }
    .to_string();
    let cfg = config_store::load();
    let current_model = if cfg.local_model.is_empty() {
        local_llm::recommend_model(&hw).to_string()
    } else {
        cfg.local_model
    };
    BsStatus {
        status,
        ram_gb: hw.ram_gb,
        vram_gb: hw.vram_gb,
        cpu_cores: hw.cpu_cores,
        recommended: local_llm::recommend_model(&hw).to_string(),
        installed: !matches!(st, LocalLlmStatus::NotInstalled),
        current_model,
    }
}

/// Install Ollama (if needed) → start the server → pull `model` → persist it as
/// the brainstorm local model. Emits "bs-progress" string events for the UI.
/// Long-running (model pull is minutes); runs off the UI thread.
#[tauri::command]
async fn bs_run_setup(app: tauri::AppHandle, model: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use capslockx_core::local_llm as ll;
        let emit = |s: &str| {
            let _ = app.emit("bs-progress", s.to_string());
        };
        if !ll::ollama_installed() {
            emit("Installing Ollama (you may see a permission prompt)…");
            ll::install_ollama()?;
        }
        emit("Starting the Ollama server…");
        ll::ensure_running()?;
        emit(&format!(
            "Downloading model {model} — this can take several minutes…"
        ));
        ll::ensure_model(&model)?;
        emit("Saving settings…");
        let mut cfg = config_store::load();
        cfg.prefer_local = true;
        cfg.local_model = model.clone();
        config_store::save(&cfg);
        crate::shm::SharedState::signal_config_changed();
        Ok(format!(
            "Ready! Local model {model} is set up. Press Space+B to chat."
        ))
    })
    .await
    .map_err(|e| format!("setup task failed: {e}"))?
}

/// Entry point for the `prefs-window` subcommand. Blocks until the window closes.
pub fn run() {
    SETUP_MODE.store(
        std::env::args().any(|a| a == "--setup=brainstorm"),
        Ordering::Relaxed,
    );

    // Single-instance: if a prefs window is already open, focus it and exit so
    // a second "Preferences…" click doesn't spawn a duplicate window.
    if !acquire_single_instance() {
        focus_existing();
        return;
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            is_setup_mode,
            bs_detect,
            bs_run_setup
        ])
        .setup(|app| {
            WebviewWindowBuilder::new(app.handle(), "prefs", WebviewUrl::App("index.html".into()))
                .title("CapsLockX Preferences")
                .inner_size(560.0, 760.0)
                .resizable(true)
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
