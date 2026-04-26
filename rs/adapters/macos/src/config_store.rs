/// Persistent configuration – reads/writes ~/.config/CapsLockX/config.json.
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
    #[serde(default = "default_edit_speed")]
    pub page_speed:      f64,
    #[serde(default = "default_edit_speed")]
    pub tab_speed:       f64,
    #[serde(default = "default_edit_speed")]
    pub action_speed:    f64,
    pub mouse_speed:     f64,
    pub scroll_speed:    f64,
    /// STT engine: "sherpa" (SenseVoice) or "whisper"
    #[serde(default = "default_stt_engine")]
    pub stt_engine:          String,
    #[serde(default)]
    pub gemini_api_key:      String,
    #[serde(default)]
    pub openai_api_key:      String,
    #[serde(default)]
    pub anthropic_api_key:   String,
    #[serde(default)]
    pub elevenlabs_api_key:  String,
    /// Backwards compat: old single key migrates to gemini/openai based on prefix.
    #[serde(default)]
    pub llm_api_key:         String,
    #[serde(default)]
    pub llm_model:           String,
    #[serde(default)]
    pub stt_correction:      bool,
    /// TTS fallback chain (comma-separated provider names).
    #[serde(default = "default_tts_chain")]
    pub tts_chain:           String,
    /// STT polishing fallback chain (comma-separated stage names).
    #[serde(default = "default_stt_polish_chain")]
    pub stt_polish_chain:    String,
    // Advanced voice/AEC thresholds
    #[serde(default = "default_aec_gain")]
    pub aec_gain:            f32,
    #[serde(default = "default_noise_gate")]
    pub noise_gate:          f32,
    #[serde(default = "default_speech_start_prob")]
    pub speech_start_prob:   f32,
    #[serde(default = "default_speech_end_prob")]
    pub speech_end_prob:     f32,
    #[serde(default = "default_speech_start_frames")]
    pub speech_start_frames: usize,
    #[serde(default = "default_silence_end_frames")]
    pub silence_end_frames:  usize,
    /// VPIO AEC mode: "off" | "dual-only" | "always".
    #[serde(default = "default_aec_mode")]
    pub aec_mode:            String,
    /// Allow overlay to be visible in screenshots/screen sharing.
    #[serde(default)]
    pub overlay_sharing:     bool,
    /// Window cycle order (Space+Z): "column", "row", "x,y", "y,x",
    /// "diagonal", "linear", "id".
    #[serde(default = "default_window_cycle_order")]
    pub window_cycle_order:  String,
    /// Window arrange order (Space+C tile/cascade): same options as cycle order.
    #[serde(default = "default_window_arrange_order")]
    pub window_arrange_order: String,

    // ── Voice translation (Phase 2) ───────────────────────────────────────
    /// Master toggle for PTT voice translation.
    #[serde(default)]
    pub translate_enabled: bool,
    /// Preset: "off", "learning", "interpreter", "chat", "conversation", "custom".
    #[serde(default = "default_translate_preset")]
    pub translate_preset: String,
    /// Target language BCP-47 code when direction=one_way.
    #[serde(default = "default_translate_target")]
    pub translate_target: String,
    /// Second language when direction=between (lang_a is translate_target).
    #[serde(default = "default_translate_other")]
    pub translate_other: String,
    /// "one_way" or "between".
    #[serde(default = "default_translate_direction")]
    pub translate_direction: String,
    /// What to type: "original", "translated", or "both".
    #[serde(default = "default_translate_type")]
    pub translate_type: String,
    /// Template for "both" mode. Placeholders: __ORIGINAL__, __TRANSLATION__.
    #[serde(default = "default_translate_both_template")]
    pub translate_both_template: String,
    /// TTS speech source: "original", "translated", or "off".
    #[serde(default = "default_translate_tts_source")]
    pub translate_tts_source: String,
    /// LLM provider for polish/translation: "gemini", "openai", "anthropic".
    #[serde(default = "default_translate_polish_provider")]
    pub translate_polish_provider: String,
    /// TTS provider: "gemini", "openai", "elevenlabs", "piper", "iflytek".
    #[serde(default = "default_translate_tts_provider")]
    pub translate_tts_provider: String,

    // ── Wake-word ─────────────────────────────────────────────────────────
    #[serde(default)]
    pub wake_word_enabled: bool,
    #[serde(default)]
    pub wake_word_model_dir: String,
    #[serde(default)]
    pub wake_word_keywords_file: String,
    #[serde(default = "default_wake_word_threshold")]
    pub wake_word_threshold: f32,
    #[serde(default = "default_wake_word_hold_ms")]
    pub wake_word_hold_ms: u64,

    // ── Note-mode translation ─────────────────────────────────────────────
    #[serde(default)]
    pub note_translate_enabled: bool,
    #[serde(default)]
    pub note_translate_target: String,
}

fn default_wake_word_threshold() -> f32 { 0.25 }
fn default_wake_word_hold_ms() -> u64 { 8000 }

fn default_translate_preset() -> String { "off".to_string() }
fn default_translate_target() -> String { "English".to_string() }
fn default_translate_other() -> String { "Japanese".to_string() }
fn default_translate_direction() -> String { "one_way".to_string() }
fn default_translate_type() -> String { "translated".to_string() }
fn default_translate_both_template() -> String { "__ORIGINAL__\n__TRANSLATION__".to_string() }
fn default_translate_tts_source() -> String { "original".to_string() }
fn default_translate_polish_provider() -> String { "gemini".to_string() }
fn default_translate_tts_provider() -> String { "gemini".to_string() }

fn default_window_cycle_order() -> String { "y,x".to_string() }
fn default_window_arrange_order() -> String { "y,x".to_string() }

fn default_stt_engine() -> String { "sherpa".to_string() }
fn default_tts_chain() -> String { "elevenlabs:rachel,gemini-2.5-flash-preview-tts,openai:tts-1,msedge,native".to_string() }
fn default_stt_polish_chain() -> String { "mlx:qwen2.5-3b,llm-corrector,raw".to_string() }
fn default_edit_speed() -> f64 { 30.0 }
fn default_aec_gain() -> f32 { 15.0 }
fn default_noise_gate() -> f32 { 0.003 }
fn default_speech_start_prob() -> f32 { 0.8 }
fn default_speech_end_prob() -> f32 { 0.6 }
fn default_speech_start_frames() -> usize { 10 }
fn default_silence_end_frames() -> usize { 20 }
fn default_aec_mode() -> String { "always".to_string() }

impl Default for FullConfig {
    fn default() -> Self {
        Self {
            use_capslock:    true,
            use_space:       true,
            use_insert:      false,
            use_scroll_lock: false,
            use_ralt:        false,
            cursor_speed:    60.0,
            page_speed:      30.0,
            tab_speed:       30.0,
            action_speed:    30.0,
            mouse_speed:     1600.0,
            scroll_speed:    1600.0,
            stt_engine:      "sherpa".to_string(),
            gemini_api_key: String::new(),
            openai_api_key: String::new(),
            anthropic_api_key: String::new(),
            elevenlabs_api_key: String::new(),
            llm_api_key: String::new(),
            llm_model: String::new(),
            stt_correction: false,
            tts_chain: default_tts_chain(),
            stt_polish_chain: default_stt_polish_chain(),
            aec_gain: default_aec_gain(),
            noise_gate: default_noise_gate(),
            speech_start_prob: default_speech_start_prob(),
            speech_end_prob: default_speech_end_prob(),
            speech_start_frames: default_speech_start_frames(),
            silence_end_frames: default_silence_end_frames(),
            aec_mode: default_aec_mode(),
            overlay_sharing: false,
            window_cycle_order: default_window_cycle_order(),
            window_arrange_order: default_window_arrange_order(),
            translate_enabled: false,
            translate_preset: default_translate_preset(),
            translate_target: default_translate_target(),
            translate_other: default_translate_other(),
            translate_direction: default_translate_direction(),
            translate_type: default_translate_type(),
            translate_both_template: default_translate_both_template(),
            translate_tts_source: default_translate_tts_source(),
            translate_polish_provider: default_translate_polish_provider(),
            translate_tts_provider: default_translate_tts_provider(),
            wake_word_enabled: false,
            wake_word_model_dir: String::new(),
            wake_word_keywords_file: String::new(),
            wake_word_threshold: default_wake_word_threshold(),
            wake_word_hold_ms: default_wake_word_hold_ms(),
            note_translate_enabled: false,
            note_translate_target: String::new(),
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
            cursor_speed:      cfg.speed.cursor_speed,
            page_speed:        cfg.speed.page_speed,
            tab_speed:         cfg.speed.tab_speed,
            action_speed:      cfg.speed.action_speed,
            mouse_speed:       cfg.speed.mouse_speed,
            scroll_speed:      cfg.speed.scroll_speed,
            stt_engine:        cfg.stt_engine.clone(),
            gemini_api_key: cfg.gemini_api_key.clone(),
            openai_api_key: cfg.openai_api_key.clone(),
            anthropic_api_key: cfg.anthropic_api_key.clone(),
            elevenlabs_api_key: cfg.elevenlabs_api_key.clone(),
            llm_api_key: String::new(),
            llm_model: String::new(),
            stt_correction: cfg.stt_correction,
            tts_chain: cfg.tts_chain.clone(),
            stt_polish_chain: cfg.stt_polish_chain.clone(),
            aec_gain: cfg.aec_gain,
            noise_gate: cfg.noise_gate,
            speech_start_prob: cfg.speech_start_prob,
            speech_end_prob: cfg.speech_end_prob,
            speech_start_frames: cfg.speech_start_frames,
            silence_end_frames: cfg.silence_end_frames,
            aec_mode: cfg.aec_mode.clone(),
            overlay_sharing: false, // not in ClxConfig, default false
            window_cycle_order: default_window_cycle_order(),
            window_arrange_order: default_window_arrange_order(),
            translate_enabled: false,
            translate_preset: default_translate_preset(),
            translate_target: default_translate_target(),
            translate_other: default_translate_other(),
            translate_direction: default_translate_direction(),
            translate_type: default_translate_type(),
            translate_both_template: default_translate_both_template(),
            translate_tts_source: default_translate_tts_source(),
            translate_polish_provider: default_translate_polish_provider(),
            translate_tts_provider: default_translate_tts_provider(),
            wake_word_enabled: cfg.wake_word_enabled,
            wake_word_model_dir: cfg.wake_word_model_dir.clone(),
            wake_word_keywords_file: cfg.wake_word_keywords_file.clone(),
            wake_word_threshold: cfg.wake_word_threshold,
            wake_word_hold_ms: cfg.wake_word_hold_ms,
            note_translate_enabled: cfg.note_translate_enabled,
            note_translate_target: cfg.note_translate_target.clone(),
        }
    }

    pub fn into_clx_config(self) -> ClxConfig {
        ClxConfig {
            use_capslock:       self.use_capslock,
            use_space:          self.use_space,
            use_insert:         self.use_insert,
            use_scroll_lock:    self.use_scroll_lock,
            use_ralt:           self.use_ralt,
            speed: SpeedConfig {
                cursor_speed: self.cursor_speed,
                page_speed:   self.page_speed,
                tab_speed:    self.tab_speed,
                action_speed: self.action_speed,
                mouse_speed:  self.mouse_speed,
                scroll_speed: self.scroll_speed,
            },
            stt_engine:         self.stt_engine,
            // Migrate old single llm_api_key to per-provider keys.
            gemini_api_key:     if !self.gemini_api_key.is_empty() { self.gemini_api_key }
                                else if self.llm_api_key.starts_with("AIza") { self.llm_api_key.clone() }
                                else { String::new() },
            openai_api_key:     if !self.openai_api_key.is_empty() { self.openai_api_key }
                                else if self.llm_api_key.starts_with("sk-") && !self.llm_api_key.starts_with("sk-ant-") { self.llm_api_key.clone() }
                                else { String::new() },
            anthropic_api_key:  if !self.anthropic_api_key.is_empty() { self.anthropic_api_key }
                                else if self.llm_api_key.starts_with("sk-ant-") { self.llm_api_key.clone() }
                                else { String::new() },
            elevenlabs_api_key: self.elevenlabs_api_key,
            stt_correction:     self.stt_correction,
            tts_chain:          self.tts_chain,
            stt_polish_chain:   self.stt_polish_chain,
            aec_gain:            self.aec_gain,
            noise_gate:          self.noise_gate,
            speech_start_prob:   self.speech_start_prob,
            speech_end_prob:     self.speech_end_prob,
            speech_start_frames: self.speech_start_frames,
            silence_end_frames:  self.silence_end_frames,
            aec_mode:            self.aec_mode,
            wake_word_enabled:       self.wake_word_enabled,
            wake_word_model_dir:     self.wake_word_model_dir,
            wake_word_keywords_file: self.wake_word_keywords_file,
            wake_word_threshold:     self.wake_word_threshold,
            wake_word_hold_ms:       self.wake_word_hold_ms,
            note_translate_enabled:  self.note_translate_enabled,
            note_translate_target:   self.note_translate_target,
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("config.json")
}

pub fn load() -> FullConfig {
    let path = config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        match serde_json::from_str::<FullConfig>(&data) {
            Ok(cfg) => {
                eprintln!("[CLX] config: stt_correction={} llm_key={}... llm_model={}",
                    cfg.stt_correction, &cfg.llm_api_key[..cfg.llm_api_key.len().min(10)], cfg.llm_model);
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
