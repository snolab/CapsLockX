#![windows_subsystem = "windows"]

//! CapsLockX – Windows adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-windows
//! Build (release): cargo build -p capslockx-windows --release

mod commands;
mod config_store;
mod hook;
mod output;
mod shm;
mod vd_api;
mod vk;

use std::path::Path;
use std::process::{Child, Command};
use std::sync::OnceLock;

use tauri::image::Image;
use tauri::tray::TrayIconId;
use tauri::{AppHandle, Manager as _};

// ── Embedded tray icons ─────────────────────────────────────────────────────

static ICON_OFF: &[u8] = include_bytes!("../../../../Data/XIconWhite.ico");
static ICON_ON:  &[u8] = include_bytes!("../../../../Data/XIconBlue.ico");

// ── Global AppHandle so hook.rs can update the tray icon ────────────────────

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

const TRAY_ID: &str = "main";

/// Switch tray icon between on (blue) and off (white).
pub fn update_tray_icon(active: bool) {
    let Some(app) = APP_HANDLE.get() else { return };
    let id = TrayIconId::new(TRAY_ID);
    let Some(tray) = app.tray_by_id(&id) else { return };
    let bytes = if active { ICON_ON } else { ICON_OFF };
    if let Ok(icon) = Image::from_bytes(bytes) {
        let _ = tray.set_icon(Some(icon));
    }
}

fn main() {
    // ── CLI subcommands (delegate to external tools, no GUI) ───────────
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            "read-screen-text" => {
                let tool = Path::new(r".\rs\target\release\clx-screen-reader.exe");
                let tool_alt = Path::new(r".\clx-screen-reader.exe");
                let exe = if tool.exists() {
                    tool
                } else if tool_alt.exists() {
                    tool_alt
                } else {
                    eprintln!("error: clx-screen-reader.exe not found");
                    std::process::exit(1);
                };
                let status = Command::new(exe).status().unwrap_or_else(|e| {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                });
                std::process::exit(status.code().unwrap_or(1));
            }
            _ => {}
        }
    }

    // Ensure only one instance runs at a time.
    shm::SharedState::kill_previous();

    let cfg = config_store::load();
    hook::init_engine(cfg.clone().into_clx_config());

    // Create shared memory for IPC with AHK before installing the hook.
    if let Some(shm) = shm::SharedState::create() {
        eprintln!("[CLX] shared memory IPC created");
        hook::init_shared_state(shm);
    } else {
        eprintln!("[CLX] shared memory creation failed (standalone mode)");
    }

    vd_api::init();

    // Spawn AHK first so its hooks are installed before ours.
    // WH_KEYBOARD_LL hooks are called most-recent-first, so installing
    // our hook AFTER AHK ensures Rust gets first crack at every key.
    let mut ahk_child = spawn_ahk();
    if ahk_child.is_some() {
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    hook::install_hook();
    let engine = hook::engine();

    tauri::Builder::default()
        .manage(engine)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            // Store the handle so hook.rs can update the tray icon.
            let _ = APP_HANDLE.set(app.handle().clone());

            // Watch the quit event so a new instance can ask us to exit cleanly.
            if let Some(evt) = shm::SharedState::create_quit_event() {
                let app_handle = app.handle().clone();
                let raw = evt.0 as usize; // extract raw ptr for Send
                std::thread::spawn(move || {
                    use windows::Win32::Foundation::{CloseHandle, HANDLE};
                    use windows::Win32::System::Threading::WaitForSingleObject;
                    unsafe {
                        let h = HANDLE(raw as *mut _);
                        WaitForSingleObject(h, u32::MAX); // INFINITE
                        let _ = CloseHandle(h);
                    }
                    app_handle.exit(0);
                });
            }

            let prefs_item  = MenuItemBuilder::with_id("prefs",      "Preferences…").build(app)?;
            let config_item = MenuItemBuilder::with_id("config_dir", "Open Config Folder…").build(app)?;
            let quit_item   = MenuItemBuilder::with_id("quit",       "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&prefs_item, &config_item, &quit_item])
                .build()?;

            let icon = Image::from_bytes(ICON_OFF)
                .expect("embedded ICO must be valid");
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "prefs" => {
                        // Re-show if already open.
                        if let Some(w) = app.get_webview_window("prefs") {
                            let _ = w.show();
                            let _ = w.set_focus();
                            return;
                        }
                        // Create on demand (must spawn thread to avoid deadlock on Windows).
                        let app = app.clone();
                        std::thread::spawn(move || {
                            use tauri::webview::WebviewWindowBuilder;
                            use tauri::WebviewUrl;
                            let _ = WebviewWindowBuilder::new(
                                &app,
                                "prefs",
                                WebviewUrl::App("index.html".into()),
                            )
                            .title("CapsLockX Preferences")
                            .inner_size(560.0, 640.0)
                            .resizable(false)
                            .center()
                            .build();
                        });
                    }
                    "config_dir" => {
                        if let Some(dir) = config_store::config_path().parent() {
                            let _ = std::fs::create_dir_all(dir);
                            let _ = Command::new("explorer").arg(dir).spawn();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
        ])
        .build(tauri::generate_context!())
        .expect("tauri build error")
        .run(|_app, event| {
            // Keep running when the prefs window is closed; only exit via app.exit().
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });

    hook::uninstall_hook();

    // Terminate AHK child process on shutdown.
    if let Some(ref mut child) = ahk_child {
        eprintln!("[CLX] terminating AHK child");
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Spawn AHK module loader (lightweight, Rust-first path).
///
/// 1. Run `Core\ahk.exe Core\GenerateModuleRunner.ahk` and wait for it to finish
///    (generates `Core\CapsLockX-ModulesRunner.ahk` and `Core\CapsLockX-ModulesFunctions.ahk`).
/// 2. Spawn `Core\ahk.exe Core\ModuleLoader.ahk` (stays running for modules).
fn spawn_ahk() -> Option<Child> {
    let exe = Path::new(r".\Core\ahk.exe");
    let generator = Path::new(r".\Core\GenerateModuleRunner.ahk");
    let loader = Path::new(r".\Core\ModuleLoader.ahk");

    if !exe.exists() {
        eprintln!("[CLX] Core\\ahk.exe not found, AHK modules disabled");
        return None;
    }
    if !generator.exists() || !loader.exists() {
        eprintln!("[CLX] GenerateModuleRunner.ahk or ModuleLoader.ahk not found, AHK modules disabled");
        return None;
    }

    // Step 1: generate module runner/functions files (blocking).
    eprintln!("[CLX] running GenerateModuleRunner.ahk …");
    match Command::new(exe).arg(r"Core\GenerateModuleRunner.ahk").status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("[CLX] GenerateModuleRunner exited with {status}");
            }
        }
        Err(e) => {
            eprintln!("[CLX] failed to run GenerateModuleRunner: {e}");
            return None;
        }
    }

    // Step 2: spawn the lightweight module loader (non-blocking).
    match Command::new(exe).arg(r"Core\ModuleLoader.ahk").spawn() {
        Ok(child) => {
            eprintln!("[CLX] spawned ModuleLoader (pid={})", child.id());
            Some(child)
        }
        Err(e) => {
            eprintln!("[CLX] failed to spawn ModuleLoader: {e}");
            None
        }
    }
}
