#![windows_subsystem = "windows"]

//! CapsLockX – Windows adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-windows
//! Build (release): cargo build -p capslockx-windows --release

mod commands;
mod config_store;
mod hook;
mod output;
mod vk;

use tauri::Manager as _;

fn main() {
    let cfg = config_store::load();
    hook::init_engine(cfg.clone().into_clx_config());
    hook::install_hook();
    let engine = hook::engine();

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
            TrayIconBuilder::new()
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("tauri error");

    hook::uninstall_hook();
}
