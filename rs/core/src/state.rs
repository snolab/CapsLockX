/// Global CapsLockX mode state (atomics → no locking in hot path).
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;
use crate::key_code::KeyCode;

// ── Mode bitmask constants ─────────────────────────────────────────────────────
pub const CM_NORMAL: u32 = 0;
pub const CM_FN:     u32 = 1;  // trigger key held
pub const CM_CLX:    u32 = 2;  // CapsLock locked mode

// ── Speed configuration ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SpeedConfig {
    pub cursor_speed: f64,  // HJKL arrow keys — units/second, default 60
    pub page_speed:   f64,  // YUIO page nav   — units/second, default 30
    pub tab_speed:    f64,  // NP tab switch    — units/second, default 30
    pub action_speed: f64,  // TG enter/delete  — units/second, default 30
    pub mouse_speed:  f64,  // WASD mouse       — px/first-second, default 1600
    pub scroll_speed: f64,  // RF scroll        — px/first-second, default 1600
}

impl Default for SpeedConfig {
    fn default() -> Self {
        Self {
            cursor_speed: 60.0,
            page_speed:   30.0,
            tab_speed:    30.0,
            action_speed: 30.0,
            mouse_speed:  1600.0,
            scroll_speed: 1600.0,
        }
    }
}

// ── Trigger-key configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClxConfig {
    pub use_capslock:    bool,
    pub use_space:       bool,
    pub use_insert:      bool,
    pub use_scroll_lock: bool,
    pub use_ralt:        bool,
    pub speed:           SpeedConfig,
    /// STT engine: "sherpa" (SenseVoice) or "whisper" (whisper.cpp)
    pub stt_engine:          String,
    /// VAD-based PTT auto-release: silence duration in ms before auto-commit.
    /// 0 = disabled (hold-to-release only). Suggested: 1500.
    pub ptt_vad_auto_release_ms: u64,
    /// PTT polish LLM provider: "gemini" | "openai" | "anthropic" | "auto".
    /// "openai" uses OTOJI_POLISH_BASE_URL (default localhost:11434 = Ollama).
    pub ptt_polish_provider: String,
    /// PTT polish model override (e.g. "qwen2.5:7b"). Empty = otoji default.
    pub ptt_polish_model:    String,
    /// Path to a whisper.cpp GGML model file. Empty = auto-detect.
    pub whisper_model_path:  String,
    /// BCP-47 language code for whisper-cli --language. Default "ja".
    pub whisper_language:    String,
    /// Per-provider API keys.
    pub gemini_api_key:      String,
    pub openai_api_key:      String,
    pub anthropic_api_key:   String,
    pub elevenlabs_api_key:  String,
    /// Enable LLM-based STT correction.
    pub stt_correction:      bool,
    /// TTS fallback chain (comma-separated model names).
    pub tts_chain:           String,
    /// STT polish fallback chain (comma-separated model names).
    pub stt_polish_chain:    String,
    // Advanced voice/AEC thresholds
    pub aec_gain:            f32,
    pub noise_gate:          f32,
    pub speech_start_prob:   f32,
    pub speech_end_prob:     f32,
    pub speech_start_frames: usize,
    pub silence_end_frames:  usize,
    /// VPIO acoustic echo cancellation mode: "off" | "dual-only" | "always".
    /// - "off":       never use VPIO; raw mic via cpal.
    /// - "dual-only": VPIO only when sys-audio is also captured (Shift+V).
    /// - "always":    VPIO for every PTT session (cancels speaker bleed even in mic-only mode). Default.
    pub aec_mode:            String,
    // ── Wake-word ─────────────────────────────────────────────────────────
    /// Master toggle for the always-on KWS listener.
    pub wake_word_enabled:     bool,
    /// Path to the sherpa-onnx KWS model directory.
    pub wake_word_model_dir:   String,
    /// Path to the BPE-encoded keywords.txt used by sherpa.
    pub wake_word_keywords_file: String,
    /// Detection threshold (0.0 – 1.0). Higher = stricter.
    pub wake_word_threshold:   f32,
    /// Safety cap on how long PTT stays held after a wake event (ms).
    /// The VAD watchdog normally releases sooner, after ~1.2s of silence.
    pub wake_word_hold_ms:     u64,
    // ── Note-mode translation ─────────────────────────────────────────────
    /// Translate continuous (non-PTT) transcripts independently of PTT.
    pub note_translate_enabled: bool,
    /// Target language for note-mode translation (e.g. "Japanese", "ja").
    pub note_translate_target:  String,
}

impl Default for ClxConfig {
    fn default() -> Self {
        Self {
            use_capslock:       true,
            use_space:          true,
            use_insert:         false,
            use_scroll_lock:    false,
            use_ralt:           false,
            speed:              SpeedConfig::default(),
            stt_engine:         "sherpa".to_string(),
            ptt_vad_auto_release_ms: 0,
            ptt_polish_provider: "openai".to_string(),
            ptt_polish_model:    String::new(),
            whisper_model_path: String::new(),
            whisper_language:   "ja".to_string(),
            gemini_api_key:     String::new(),
            openai_api_key:     String::new(),
            anthropic_api_key:  String::new(),
            elevenlabs_api_key: String::new(),
            stt_correction:     false,
            tts_chain:          "elevenlabs:rachel,gemini-2.5-flash-preview-tts,openai:tts-1,msedge,native".to_string(),
            stt_polish_chain:   "min-chars:15,min-duration:5s,mlx:qwen2.5-3b,llm-corrector,raw".to_string(),
            aec_gain:            15.0,
            noise_gate:          0.003,
            speech_start_prob:   0.8,
            speech_end_prob:     0.6,
            speech_start_frames: 10,
            silence_end_frames:  20,
            aec_mode:            "always".to_string(),
            wake_word_enabled:     false,
            wake_word_model_dir:   String::new(),
            wake_word_keywords_file: String::new(),
            wake_word_threshold:   0.25,
            wake_word_hold_ms:     8000,
            note_translate_enabled: false,
            note_translate_target:  String::new(),
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
}

// ── State struct ──────────────────────────────────────────────────────────────

pub struct ClxState {
    pub config:  RwLock<ClxConfig>,
    mode:        AtomicU32,
    paused:      AtomicBool,
    shift_held:  AtomicBool,
}

impl Default for ClxState {
    fn default() -> Self {
        Self::new(ClxConfig::default())
    }
}

impl ClxState {
    pub fn new(config: ClxConfig) -> Self {
        Self {
            config:     RwLock::new(config),
            mode:       AtomicU32::new(CM_NORMAL),
            paused:     AtomicBool::new(false),
            shift_held: AtomicBool::new(false),
        }
    }

    #[inline] pub fn mode(&self)            -> u32  { self.mode.load(Ordering::Relaxed) }
    #[inline] pub fn is_clx_active(&self)   -> bool { self.mode() != CM_NORMAL && !self.paused.load(Ordering::Relaxed) }
    #[inline] pub fn is_clx_locked(&self)   -> bool { self.mode() & CM_CLX != 0 }
    #[inline] pub fn set_shift_held(&self, held: bool) { self.shift_held.store(held, Ordering::Relaxed); }
    #[inline] pub fn is_shift_held(&self)   -> bool { self.shift_held.load(Ordering::Relaxed) }

    #[inline]
    pub fn is_trigger_key(&self, key: KeyCode) -> bool {
        let cfg = self.config.read().unwrap();
        (cfg.use_capslock    && key == KeyCode::CapsLock)
        || (cfg.use_space       && key == KeyCode::Space)
        || (cfg.use_insert      && key == KeyCode::Insert)
        || (cfg.use_scroll_lock && key == KeyCode::ScrollLock)
        || (cfg.use_ralt        && key == KeyCode::RAlt)
    }

    pub fn enter_fn_mode(&self) {
        self.mode.fetch_or(CM_FN, Ordering::Relaxed);
        self.mode.fetch_and(!CM_CLX, Ordering::Relaxed);
    }
    pub fn exit_fn_mode(&self)  { self.mode.fetch_and(!CM_FN,  Ordering::Relaxed); }
    pub fn enter_clx_mode(&self){ self.mode.fetch_or(CM_CLX,  Ordering::Relaxed); }
    pub fn exit_clx_mode(&self) { self.mode.fetch_and(!CM_CLX, Ordering::Relaxed); }
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
        assert_eq!(c.stt_engine, "sherpa");
        assert_eq!(c.gemini_api_key, "");
        assert_eq!(c.openai_api_key, "");
        assert_eq!(c.anthropic_api_key, "");
        assert_eq!(c.elevenlabs_api_key, "");
        assert!(!c.stt_correction);
        assert!(c.tts_chain.contains("elevenlabs"));
        assert!(c.stt_polish_chain.contains("raw"));
        assert_eq!(c.aec_gain, 15.0);
        assert_eq!(c.noise_gate, 0.003);
        assert_eq!(c.speech_start_prob, 0.8);
        assert_eq!(c.speech_end_prob, 0.6);
        assert_eq!(c.speech_start_frames, 10);
        assert_eq!(c.silence_end_frames, 20);
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
        assert_eq!(c.best_llm_key_and_model(), ("ollama".to_string(), String::new()));
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
