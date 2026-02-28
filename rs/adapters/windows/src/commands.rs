/// Tauri IPC commands exposed to the preferences webview.
use std::sync::Arc;
use tauri::State;
use capslockx_core::ClxEngine;
use crate::config_store::{self, FullConfig};

#[tauri::command]
pub fn get_config(engine: State<'_, Arc<ClxEngine>>) -> FullConfig {
    FullConfig::from_clx_config(&engine.get_config())
}

#[tauri::command]
pub fn set_config(cfg: FullConfig, engine: State<'_, Arc<ClxEngine>>) {
    config_store::save(&cfg);
    engine.update_config(cfg.into_clx_config());
}
