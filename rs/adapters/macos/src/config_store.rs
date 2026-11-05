/// Persistent configuration – reads/writes ~/.config/CapsLockX/config.json.
use capslockx_core::{ClxConfig, SpeedConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullConfig {
    pub use_capslock: bool,
    pub use_space: bool,
    pub use_insert: bool,
    pub use_scroll_lock: bool,
    pub use_ralt: bool,
    #[serde(default)]
    pub window_arrange_side_by_side: bool,
    pub cursor_speed: f64,
    #[serde(default = "default_edit_speed")]
    pub page_speed: f64,
    #[serde(default = "default_edit_speed")]
    pub tab_speed: f64,
    #[serde(default = "default_edit_speed")]
    pub action_speed: f64,
    pub mouse_speed: f64,
    pub scroll_speed: f64,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub anthropic_api_key: String,
    /// Backwards compat: old single key migrates to gemini/openai based on prefix.
    #[serde(default)]
    pub llm_api_key: String,
    #[serde(default)]
    pub llm_model: String,
    /// Window cycle order (Space+Z): "column", "row", "x,y", "y,x",
    /// "diagonal", "linear", "id".
    #[serde(default = "default_window_cycle_order")]
    pub window_cycle_order: String,
    /// Window arrange order (Space+C tile/cascade): same options as cycle order.
    #[serde(default = "default_window_arrange_order")]
    pub window_arrange_order: String,

    /// Settings this build doesn't know — notably the voice fields of older
    /// configs, which otoji imports on its first run. Kept so saving preferences
    /// here doesn't destroy them.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_window_cycle_order() -> String {
    "y,x".to_string()
}
fn default_window_arrange_order() -> String {
    "y,x".to_string()
}

fn default_edit_speed() -> f64 {
    30.0
}
impl Default for FullConfig {
    fn default() -> Self {
        Self {
            use_capslock: true,
            use_space: true,
            use_insert: false,
            use_scroll_lock: false,
            use_ralt: false,
            window_arrange_side_by_side: false,
            cursor_speed: 60.0,
            page_speed: 30.0,
            tab_speed: 30.0,
            action_speed: 30.0,
            mouse_speed: 1600.0,
            scroll_speed: 1600.0,
            gemini_api_key: String::new(),
            openai_api_key: String::new(),
            anthropic_api_key: String::new(),
            llm_api_key: String::new(),
            llm_model: String::new(),
            window_cycle_order: default_window_cycle_order(),
            window_arrange_order: default_window_arrange_order(),
            extra: serde_json::Map::new(),
        }
    }
}

impl FullConfig {
    pub fn from_clx_config(cfg: &ClxConfig) -> Self {
        Self {
            use_capslock: cfg.use_capslock,
            use_space: cfg.use_space,
            use_insert: cfg.use_insert,
            use_scroll_lock: cfg.use_scroll_lock,
            use_ralt: cfg.use_ralt,
            window_arrange_side_by_side: cfg.window_arrange_side_by_side,
            cursor_speed: cfg.speed.cursor_speed,
            page_speed: cfg.speed.page_speed,
            tab_speed: cfg.speed.tab_speed,
            action_speed: cfg.speed.action_speed,
            mouse_speed: cfg.speed.mouse_speed,
            scroll_speed: cfg.speed.scroll_speed,
            gemini_api_key: cfg.gemini_api_key.clone(),
            openai_api_key: cfg.openai_api_key.clone(),
            anthropic_api_key: cfg.anthropic_api_key.clone(),
            llm_api_key: String::new(),
            llm_model: String::new(),
            window_cycle_order: default_window_cycle_order(),
            window_arrange_order: default_window_arrange_order(),
            extra: serde_json::Map::new(),
        }
    }

    pub fn into_clx_config(self) -> ClxConfig {
        ClxConfig {
            use_capslock: self.use_capslock,
            use_space: self.use_space,
            use_insert: self.use_insert,
            use_scroll_lock: self.use_scroll_lock,
            use_ralt: self.use_ralt,
            window_arrange_side_by_side: self.window_arrange_side_by_side,
            speed: SpeedConfig {
                cursor_speed: self.cursor_speed,
                page_speed: self.page_speed,
                tab_speed: self.tab_speed,
                action_speed: self.action_speed,
                mouse_speed: self.mouse_speed,
                scroll_speed: self.scroll_speed,
            },
            // Migrate old single llm_api_key to per-provider keys.
            gemini_api_key: if !self.gemini_api_key.is_empty() {
                self.gemini_api_key
            } else if self.llm_api_key.starts_with("AIza") {
                self.llm_api_key.clone()
            } else {
                String::new()
            },
            openai_api_key: if !self.openai_api_key.is_empty() {
                self.openai_api_key
            } else if self.llm_api_key.starts_with("sk-")
                && !self.llm_api_key.starts_with("sk-ant-")
            {
                self.llm_api_key.clone()
            } else {
                String::new()
            },
            anthropic_api_key: if !self.anthropic_api_key.is_empty() {
                self.anthropic_api_key
            } else if self.llm_api_key.starts_with("sk-ant-") {
                self.llm_api_key.clone()
            } else {
                String::new()
            },
            // Brainstorm local-first fields (added on the Windows/rust-port side;
            // not surfaced in the macOS FullConfig yet) — use ClxConfig defaults
            // so this initializer stays complete after the beta↔main merge.
            prefer_local: true,
            local_model: String::new(),
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("config.json")
}

#[cfg(test)]
mod arrange_preference_tests {
    use super::*;

    #[test]
    fn arrange_preference_round_trips_and_old_configs_default_to_stacked() {
        let mut core = ClxConfig::default();
        core.window_arrange_side_by_side = true;
        let full = FullConfig::from_clx_config(&core);
        let mut json = serde_json::to_value(full).unwrap();
        let restored: FullConfig = serde_json::from_value(json.clone()).unwrap();
        assert!(restored.into_clx_config().window_arrange_side_by_side);
        json.as_object_mut()
            .unwrap()
            .remove("window_arrange_side_by_side");
        let legacy: FullConfig = serde_json::from_value(json).unwrap();
        assert!(!legacy.into_clx_config().window_arrange_side_by_side);
    }

    /// otoji imports these from CLX's config on first run, so a save here must
    /// not drop them even though this build no longer reads them.
    #[test]
    fn voice_fields_of_old_configs_survive_a_save() {
        let mut json = serde_json::to_value(FullConfig::default()).unwrap();
        let obj = json.as_object_mut().unwrap();
        obj.insert("stt_engine".into(), "whisper".into());
        obj.insert("elevenlabs_api_key".into(), "xi-example".into());
        let loaded: FullConfig = serde_json::from_value(json).unwrap();
        let saved = serde_json::to_value(&loaded).unwrap();
        assert_eq!(saved["stt_engine"], "whisper");
        assert_eq!(saved["elevenlabs_api_key"], "xi-example");
        assert!(saved["use_capslock"].as_bool().unwrap());
    }
}

pub fn load() -> FullConfig {
    let path = config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        match serde_json::from_str::<FullConfig>(&data) {
            Ok(cfg) => {
                eprintln!(
                    "[CLX] config: llm_key={}... llm_model={}",
                    &cfg.llm_api_key[..cfg.llm_api_key.len().min(10)],
                    cfg.llm_model
                );
                cfg
            }
            Err(e) => {
                eprintln!("[CLX] config parse error: {} — using defaults", e);
                FullConfig::default()
            }
        }
    } else {
        FullConfig::default()
    }
}

pub fn save(cfg: &FullConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(path, json);
    }
}
