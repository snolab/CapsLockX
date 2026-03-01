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
    hook::install_hook();
    let engine = hook::engine();

    // Spawn AHK with --no-core so it reads mode from shared memory.
    let mut ahk_child = spawn_ahk();

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

            let prefs_item = MenuItemBuilder::with_id("prefs", "Preferences…").build(app)?;
            let quit_item  = MenuItemBuilder::with_id("quit",  "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&prefs_item, &quit_item]).build()?;

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

/// Spawn `CapsLockX.exe CapsLockX.ahk --no-core` if the AHK launcher exists.
fn spawn_ahk() -> Option<Child> {
    // Use `.\` prefix so Windows resolves from CWD, not the app's own directory.
    // Without it, CreateProcess finds our own binary (target/release/CapsLockX.exe)
    // first, causing an infinite fork bomb.
    let exe = Path::new(r".\CapsLockX.exe");
    if !exe.exists() {
        eprintln!("[CLX] CapsLockX.exe not found, AHK modules disabled");
        return None;
    }
    match Command::new(exe).args(["CapsLockX.ahk", "--no-core"]).spawn() {
        Ok(child) => {
            eprintln!("[CLX] spawned AHK (pid={})", child.id());
            Some(child)
        }
        Err(e) => {
            eprintln!("[CLX] failed to spawn AHK: {e}");
            None
        }
    }
}
