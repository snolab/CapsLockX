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
    /// Allow overlay to be visible in screenshots/screen sharing.
    #[serde(default)]
    pub overlay_sharing:     bool,
}

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
            overlay_sharing: false,
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
            overlay_sharing: false, // not in ClxConfig, default false
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
