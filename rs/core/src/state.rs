use crate::key_code::KeyCode;
/// Global CapsLockX mode state (atomics → no locking in hot path).
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;

// ── Mode bitmask constants ─────────────────────────────────────────────────────
pub const CM_NORMAL: u32 = 0;
pub const CM_FN: u32 = 1; // trigger key held
pub const CM_CLX: u32 = 2; // CapsLock locked mode

// ── Speed configuration ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SpeedConfig {
    pub cursor_speed: f64, // HJKL arrow keys — units/second, default 60
    pub page_speed: f64,   // YUIO page nav   — units/second, default 30
    pub tab_speed: f64,    // NP tab switch    — units/second, default 30
    pub action_speed: f64, // TG enter/delete  — units/second, default 30
    pub mouse_speed: f64,  // WASD mouse       — px/first-second, default 1600
    pub scroll_speed: f64, // RF scroll        — px/first-second, default 1600
}

impl Default for SpeedConfig {
    fn default() -> Self {
        Self {
            cursor_speed: 60.0,
            page_speed: 30.0,
            tab_speed: 30.0,
            action_speed: 30.0,
            mouse_speed: 1600.0,
            scroll_speed: 1600.0,
        }
    }
}

// ── Trigger-key configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClxConfig {
    pub use_capslock: bool,
    pub use_space: bool,
    pub use_insert: bool,
    pub use_scroll_lock: bool,
    pub use_ralt: bool,
    /// CLX+C uses side-by-side; Shift+C always uses the opposite layout.
    pub window_arrange_side_by_side: bool,
    pub speed: SpeedConfig,
    /// Per-provider API keys.
    pub gemini_api_key: String,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    // ── Brainstorm: local-first LLM ───────────────────────────────────────
    /// Prefer a local LLM (Ollama) for CLX+B over any cloud API. Default true.
    /// Cloud keys remain a runtime fallback when the local server is down.
    pub prefer_local: bool,
    /// Local model tag for brainstorm (e.g. "qwen2.5:7b").
    /// Empty = auto-discover/recommend based on hardware.
    pub local_model: String,
}

impl Default for ClxConfig {
    fn default() -> Self {
        Self {
            use_capslock: true,
            use_space: true,
            use_insert: false,
            use_scroll_lock: false,
            use_ralt: false,
            window_arrange_side_by_side: false,
            speed: SpeedConfig::default(),
            gemini_api_key: String::new(),
            openai_api_key: String::new(),
            anthropic_api_key: String::new(),
            prefer_local: true,
            local_model: String::new(),
        }
    }
}

impl ClxConfig {
    /// Return the best available LLM API key and model for brainstorm/correction.
    /// Priority: Gemini > OpenAI > Anthropic > Ollama (local, no key).
    pub fn best_llm_key_and_model(&self) -> (String, String) {
        if !self.gemini_api_key.is_empty() {
            return (self.gemini_api_key.clone(), String::new());
        }
        if !self.openai_api_key.is_empty() {
            return (self.openai_api_key.clone(), String::new());
        }
        if !self.anthropic_api_key.is_empty() {
            return (self.anthropic_api_key.clone(), String::new());
        }
        // Ollama (local, no key needed).
        ("ollama".to_string(), String::new())
    }

    /// LLM key+model for the brainstorm module. Local-first: when `prefer_local`
    /// is set, route to a local backend (cloud keys remain a runtime fallback).
    /// Otherwise defers to the cloud-priority `best_llm_key_and_model`.
    ///
    /// With the in-process llama.cpp backend compiled in (`local-llm`), this
    /// returns `("local", <gguf path>)` — the hardware-sized weight file under
    /// the models dir, which the brainstorm module downloads on first use. If a
    /// `.gguf` path is explicitly configured in `local_model`, that wins.
    /// Without the feature it falls back to Ollama, as before.
    pub fn brainstorm_llm_key_and_model(&self) -> (String, String) {
        if self.prefer_local {
            #[cfg(all(feature = "local-llm", not(target_arch = "wasm32")))]
            {
                let path = if self.local_model.ends_with(".gguf") {
                    self.local_model.clone()
                } else {
                    let hw = crate::local_llm::probe_hardware();
                    let spec = crate::local_llm::recommend_gguf(&hw);
                    crate::local_llm::gguf_model_path(&spec)
                        .to_string_lossy()
                        .into_owned()
                };
                return ("local".to_string(), path);
            }
            #[cfg(not(all(feature = "local-llm", not(target_arch = "wasm32"))))]
            return ("ollama".to_string(), self.local_model.clone());
        }
        self.best_llm_key_and_model()
    }
}

// ── State struct ──────────────────────────────────────────────────────────────

pub struct ClxState {
    pub config: RwLock<ClxConfig>,
    mode: AtomicU32,
    paused: AtomicBool,
    shift_held: AtomicBool,
    /// Both triggers physically down. Gates the F-row layer; kept in sync
    /// with `held_keys` by the engine on every trigger edge.
    chord_active: AtomicBool,
}

impl Default for ClxState {
    fn default() -> Self {
        Self::new(ClxConfig::default())
    }
}

impl ClxState {
    pub fn new(config: ClxConfig) -> Self {
        Self {
            config: RwLock::new(config),
            mode: AtomicU32::new(CM_NORMAL),
            paused: AtomicBool::new(false),
            shift_held: AtomicBool::new(false),
            chord_active: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn mode(&self) -> u32 {
        self.mode.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn is_clx_active(&self) -> bool {
        self.mode() != CM_NORMAL && !self.paused.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn is_clx_locked(&self) -> bool {
        self.mode() & CM_CLX != 0
    }
    #[inline]
    pub fn set_shift_held(&self, held: bool) {
        self.shift_held.store(held, Ordering::Relaxed);
    }
    #[inline]
    pub fn set_chord_active(&self, active: bool) {
        self.chord_active.store(active, Ordering::Relaxed);
    }
    #[inline]
    pub fn is_chord_active(&self) -> bool {
        self.chord_active.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn is_shift_held(&self) -> bool {
        self.shift_held.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_trigger_key(&self, key: KeyCode) -> bool {
        let cfg = self.config.read().unwrap();
        (cfg.use_capslock && key == KeyCode::CapsLock)
            || (cfg.use_space && key == KeyCode::Space)
            || (cfg.use_insert && key == KeyCode::Insert)
            || (cfg.use_scroll_lock && key == KeyCode::ScrollLock)
            || (cfg.use_ralt && key == KeyCode::RAlt)
    }

    pub fn enter_fn_mode(&self) {
        self.mode.fetch_or(CM_FN, Ordering::Relaxed);
        self.mode.fetch_and(!CM_CLX, Ordering::Relaxed);
    }
    pub fn exit_fn_mode(&self) {
        self.mode.fetch_and(!CM_FN, Ordering::Relaxed);
    }
    pub fn enter_clx_mode(&self) {
        self.mode.fetch_or(CM_CLX, Ordering::Relaxed);
    }
    pub fn exit_clx_mode(&self) {
        self.mode.fetch_and(!CM_CLX, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_config_default_values() {
        let s = SpeedConfig::default();
        assert_eq!(s.cursor_speed, 60.0);
        assert_eq!(s.page_speed, 30.0);
        assert_eq!(s.tab_speed, 30.0);
        assert_eq!(s.action_speed, 30.0);
        assert_eq!(s.mouse_speed, 1600.0);
        assert_eq!(s.scroll_speed, 1600.0);
    }

    #[test]
    fn clx_config_default_values() {
        let c = ClxConfig::default();
        assert!(c.use_capslock);
        assert!(c.use_space);
        assert!(!c.use_insert);
        assert!(!c.use_scroll_lock);
        assert!(!c.use_ralt);
        assert_eq!(c.gemini_api_key, "");
        assert_eq!(c.openai_api_key, "");
        assert_eq!(c.anthropic_api_key, "");
    }

    #[test]
    fn best_llm_key_prefers_gemini() {
        let mut c = ClxConfig::default();
        c.gemini_api_key = "g".into();
        c.openai_api_key = "o".into();
        c.anthropic_api_key = "a".into();
        assert_eq!(c.best_llm_key_and_model(), ("g".to_string(), String::new()));
    }

    #[test]
    fn best_llm_key_falls_back_to_openai() {
        let mut c = ClxConfig::default();
        c.openai_api_key = "o".into();
        c.anthropic_api_key = "a".into();
        assert_eq!(c.best_llm_key_and_model(), ("o".to_string(), String::new()));
    }

    #[test]
    fn best_llm_key_falls_back_to_anthropic() {
        let mut c = ClxConfig::default();
        c.anthropic_api_key = "a".into();
        assert_eq!(c.best_llm_key_and_model(), ("a".to_string(), String::new()));
    }

    #[test]
    fn best_llm_key_falls_back_to_ollama() {
        let c = ClxConfig::default();
        assert_eq!(
            c.best_llm_key_and_model(),
            ("ollama".to_string(), String::new())
        );
    }

    #[test]
    fn prefer_local_defaults_true() {
        assert!(ClxConfig::default().prefer_local);
    }

    #[test]
    fn brainstorm_llm_local_first_ignores_cloud_keys() {
        let mut c = ClxConfig::default();
        c.gemini_api_key = "g".into();
        c.local_model = "qwen2.5:7b".into();
        // prefer_local is true by default → always Ollama, despite the cloud key.
        assert_eq!(
            c.brainstorm_llm_key_and_model(),
            ("ollama".to_string(), "qwen2.5:7b".to_string())
        );
    }

    #[test]
    fn brainstorm_llm_falls_back_to_cloud_when_prefer_local_off() {
        let mut c = ClxConfig::default();
        c.prefer_local = false;
        c.gemini_api_key = "g".into();
        assert_eq!(
            c.brainstorm_llm_key_and_model(),
            ("g".to_string(), String::new())
        );
    }

    #[test]
    fn state_default_starts_in_normal_mode() {
        let s = ClxState::default();
        assert_eq!(s.mode(), CM_NORMAL);
        assert!(!s.is_clx_active());
        assert!(!s.is_clx_locked());
        assert!(!s.is_shift_held());
    }

    #[test]
    fn state_enter_fn_sets_fn_and_clears_clx() {
        let s = ClxState::default();
        s.enter_clx_mode();
        assert!(s.is_clx_locked());
        s.enter_fn_mode();
        assert_eq!(s.mode() & CM_FN, CM_FN);
        assert!(!s.is_clx_locked());
    }

    #[test]
    fn state_exit_fn_clears_only_fn() {
        let s = ClxState::default();
        s.enter_fn_mode();
        s.enter_clx_mode();
        s.exit_fn_mode();
        assert_eq!(s.mode() & CM_FN, 0);
        assert!(s.is_clx_locked());
    }

    #[test]
    fn state_enter_exit_clx_mode() {
        let s = ClxState::default();
        s.enter_clx_mode();
        assert!(s.is_clx_locked());
        assert!(s.is_clx_active());
        s.exit_clx_mode();
        assert!(!s.is_clx_locked());
        assert!(!s.is_clx_active());
    }

    #[test]
    fn state_shift_held_toggle() {
        let s = ClxState::default();
        s.set_shift_held(true);
        assert!(s.is_shift_held());
        s.set_shift_held(false);
        assert!(!s.is_shift_held());
    }

    #[test]
    fn state_is_trigger_key_respects_config() {
        let s = ClxState::default();
        assert!(s.is_trigger_key(KeyCode::CapsLock));
        assert!(s.is_trigger_key(KeyCode::Space));
        assert!(!s.is_trigger_key(KeyCode::Insert));
        assert!(!s.is_trigger_key(KeyCode::ScrollLock));
        assert!(!s.is_trigger_key(KeyCode::RAlt));
        assert!(!s.is_trigger_key(KeyCode::A));
    }

    #[test]
    fn state_is_trigger_key_with_all_enabled() {
        let mut cfg = ClxConfig::default();
        cfg.use_insert = true;
        cfg.use_scroll_lock = true;
        cfg.use_ralt = true;
        let s = ClxState::new(cfg);
        assert!(s.is_trigger_key(KeyCode::Insert));
        assert!(s.is_trigger_key(KeyCode::ScrollLock));
        assert!(s.is_trigger_key(KeyCode::RAlt));
    }

    #[test]
    fn state_is_clx_active_false_when_paused() {
        let s = ClxState::default();
        s.enter_clx_mode();
        assert!(s.is_clx_active());
        s.paused.store(true, Ordering::Relaxed);
        assert!(!s.is_clx_active());
    }
}
