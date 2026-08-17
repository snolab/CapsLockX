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

/// Launch-at-login state. Deliberately NOT part of `FullConfig`: the Task
/// Scheduler entry is the source of truth, so there is no mirror flag in
/// config.json that could drift out of sync with reality.
#[tauri::command]
fn get_autostart() -> bool {
    clx_autostart::is_enabled()
}

/// Apply the requested state and return what the system actually ended up as,
/// so a declined UAC prompt snaps the checkbox back instead of lying.
#[tauri::command]
fn set_autostart(on: bool) -> bool {
    clx_autostart::set_enabled(on)
}

#[tauri::command]
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

/// Version + build time + install source shown in the prefs footer so you can
/// confirm WHICH build is running and where it came from. The build time is the
/// running exe's mtime (changes on every rebuild — handy for verifying a fix got
/// deployed); the source is inferred from the exe path (git dev build vs.
/// chocolatey vs. npm vs. a plain install).
#[derive(serde::Serialize)]
struct VersionInfo {
    /// Crate version, e.g. "2.0.0".
    version: String,
    /// Build timestamp (running exe mtime), "YYYY-MM-DD HH:MM UTC".
    built: String,
    /// Install source: "git (dev build)" | "chocolatey" | "npm" | "installed".
    source: String,
    /// Full path of the running executable.
    exe_path: String,
    /// Official repo "owner/name" for the UI's latest-release lookup.
    repo: String,
}

#[tauri::command]
fn get_version() -> VersionInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let exe = std::env::current_exe().unwrap_or_default();
    let built = std::fs::metadata(&exe)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| fmt_utc(d.as_secs()))
        .unwrap_or_default();
    VersionInfo {
        version,
        built,
        source: detect_source(&exe),
        exe_path: exe.to_string_lossy().to_string(),
        repo: "snolab/CapsLockX".to_string(),
    }
}

/// Infer how this binary was installed from its path + neighbouring markers.
fn detect_source(exe: &std::path::Path) -> String {
    let p = exe.to_string_lossy().to_lowercase();
    // Cargo dev build: lives under `…/target/{release,debug}/` inside a checkout.
    if p.contains("\\target\\release\\") || p.contains("\\target\\debug\\") {
        return "git (dev build)".to_string();
    }
    // Chocolatey shims/libs.
    if p.contains("\\chocolatey\\") {
        return "chocolatey".to_string();
    }
    // npm global install (…\npm\node_modules\… or …\AppData\Roaming\npm\…).
    if p.contains("\\npm\\") || p.contains("\\node_modules\\") {
        return "npm".to_string();
    }
    // Fallback: a checkout anywhere with a .git dir in an ancestor → git.
    if exe
        .ancestors()
        .any(|a| a.join(".git").exists() || a.file_name().is_some_and(|n| n == "CapsLockX"))
    {
        return "git (source tree)".to_string();
    }
    "installed".to_string()
}

/// Format Unix epoch seconds as "YYYY-MM-DD HH:MM UTC" without external crates
/// (Howard Hinnant's civil-from-days algorithm).
fn fmt_utc(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let tod = secs % 86400;
    let (h, mi) = (tod / 3600, (tod % 3600) / 60);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", y, m, d, h, mi)
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

/// Exit this prefs process if the parent CLX (PID in `CLX_PARENT_PID`) exits, so
/// a prefs window can never orphan and linger when clx is killed / self-updates.
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
            if WaitForSingleObject(h, u32::MAX) == WAIT_OBJECT_0 {
                std::process::exit(0);
            }
            let _ = CloseHandle(h);
        }
    });
}

/// Entry point for the `prefs-window` subcommand. Blocks until the window closes.
pub fn run() {
    exit_with_parent();
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
            get_version,
            get_autostart,
            set_autostart,
            autostart_hint,
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
