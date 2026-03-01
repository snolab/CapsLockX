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

use tauri::Manager as _;

fn main() {
    let cfg = config_store::load();
    hook::init_engine(cfg.clone().into_clx_config());

    // Create shared memory for IPC with AHK before installing the hook.
    if let Some(shm) = shm::SharedState::create() {
        eprintln!("[CLX] shared memory IPC created");
        hook::init_shared_state(shm);
    } else {
        eprintln!("[CLX] shared memory creation failed (standalone mode)");
    }

    hook::install_hook();
    let engine = hook::engine();

    // Spawn AHK with --no-core so it reads mode from shared memory.
    let mut ahk_child = spawn_ahk();

    tauri::Builder::default()
        .manage(engine)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;
            use tauri::image::Image;

            let prefs_item = MenuItemBuilder::with_id("prefs", "Preferences…").build(app)?;
            let quit_item  = MenuItemBuilder::with_id("quit",  "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&prefs_item, &quit_item]).build()?;

            let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))
                .expect("tray.png must be a valid PNG");
            TrayIconBuilder::with_id("clx")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "prefs" => {
                        if let Some(w) = app.get_webview_window("prefs") {
                            let _ = w.show();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            hook::set_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("tauri error");

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
    let exe = Path::new("CapsLockX.exe");
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
