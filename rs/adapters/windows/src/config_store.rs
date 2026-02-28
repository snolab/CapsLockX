/// Persistent configuration – reads/writes %APPDATA%\CapsLockX\config.json.
use capslockx_core::{ClxConfig, SpeedConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullConfig {
    pub use_capslock:    bool,
    pub use_space:       bool,
    pub use_insert:      bool,
    pub use_scroll_lock: bool,
    pub use_ralt:        bool,
    pub cursor_speed:    f64,
    pub mouse_speed:     f64,
    pub scroll_speed:    f64,
}

impl Default for FullConfig {
    fn default() -> Self {
        Self {
            use_capslock:    true,
            use_space:       true,
            use_insert:      false,
            use_scroll_lock: false,
            use_ralt:        false,
            cursor_speed:    15.0,
            mouse_speed:     360.0,
            scroll_speed:    720.0,
        }
    }
}

impl FullConfig {
    pub fn from_clx_config(cfg: &ClxConfig) -> Self {
        Self {
            use_capslock:    cfg.use_capslock,
            use_space:       cfg.use_space,
            use_insert:      cfg.use_insert,
            use_scroll_lock: cfg.use_scroll_lock,
            use_ralt:        cfg.use_ralt,
            cursor_speed:    cfg.speed.cursor_speed,
            mouse_speed:     cfg.speed.mouse_speed,
            scroll_speed:    cfg.speed.scroll_speed,
        }
    }

    pub fn into_clx_config(self) -> ClxConfig {
        ClxConfig {
            use_capslock:    self.use_capslock,
            use_space:       self.use_space,
            use_insert:      self.use_insert,
            use_scroll_lock: self.use_scroll_lock,
            use_ralt:        self.use_ralt,
            speed: SpeedConfig {
                cursor_speed: self.cursor_speed,
                mouse_speed:  self.mouse_speed,
                scroll_speed: self.scroll_speed,
            },
        }
    }
}

fn config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("config.json")
}

pub fn load() -> FullConfig {
    let path = config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
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
