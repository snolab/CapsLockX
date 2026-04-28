pub mod agent;
pub mod brainstorm;
pub mod edit;
pub mod media;
pub mod mouse;
pub mod virtual_desktop;
#[cfg(feature = "stt")]
pub mod voice;
pub mod voice_otoji;
pub mod voice_ptt;
pub mod wake_word;
#[cfg(test)]
mod voice_ptt_test;
#[cfg(not(feature = "stt"))]
mod voice {
    //! No-op stub used when the `stt` feature is disabled (e.g. Windows
    //! builds, where `whisper-rs 0.13` fails to compile). Mirrors the
    //! public surface of `VoiceModule` so the rest of `Modules` compiles
    //! unchanged. All hotkeys silently fall through.
    use std::sync::Arc;
    use crate::key_code::KeyCode;
    use crate::platform::Platform;

    pub struct VoiceModule;

    impl VoiceModule {
        pub fn with_stt_engine(_platform: Arc<dyn Platform>, _stt_engine: String) -> Self {
            Self
        }
        pub fn with_llm_config(self, _api_key: String, _model: String, _correction: bool) -> Self {
            self
        }
        pub fn preload(&self) {}
        pub fn start_wake_word(&self, _cfg: super::wake_word::WakeWordConfig) {}
        pub fn on_key_down(&self, _key: KeyCode) -> bool { false }
        pub fn on_key_up(&self, _key: KeyCode) -> bool { false }
        pub fn is_mapped_key(&self, _key: KeyCode) -> bool { false }
        pub fn stop(&self) {}
        #[allow(clippy::too_many_arguments)]
        pub fn update_config(
            &self,
            _stt_engine: String, _api_key: String, _model: String, _correction: bool,
            _tts_chain: String, _stt_polish_chain: String,
            _aec_gain: f32, _noise_gate: f32,
            _speech_start_prob: f32, _speech_end_prob: f32,
            _speech_start_frames: usize, _silence_end_frames: usize,
            _aec_mode: String,
        ) {}
    }
}
pub mod window_manager;

use std::sync::Arc;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;
use crate::state::{ClxConfig, ClxState, SpeedConfig};

use agent::AgentModule;
use brainstorm::BrainstormModule;
use edit::EditModule;
use media::MediaModule;
use mouse::MouseModule;
use virtual_desktop::VirtualDesktopModule;
use voice::VoiceModule;
use window_manager::WindowManagerModule;

/// Call a module function with panic isolation. If the module panics,
/// log the error and return false — the core keyboard/mouse keeps working.
fn safe_call(module: &str, f: impl FnOnce() -> bool) -> bool {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            eprintln!("[CLX] PANIC in {} module (isolated, core unaffected): {}", module, msg);
            false
        }
    }
}

pub struct Modules {
    pub agent:       AgentModule,
    pub brainstorm:  BrainstormModule,
    edit:            EditModule,
    mouse:           MouseModule,
    media:           MediaModule,
    virtual_desktop: VirtualDesktopModule,
    voice:           VoiceModule,
    window_manager:  WindowManagerModule,
    platform:        Arc<dyn Platform>,
}

impl Modules {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let cfg = state.config.read().unwrap();
        let (best_key, best_model) = cfg.best_llm_key_and_model();
        let s = Self {
            agent:           AgentModule::new(Arc::clone(&platform)),
            brainstorm:      BrainstormModule::new(
                Arc::clone(&platform),
                best_key.clone(),
                best_model.clone(),
            ),
            edit:            EditModule::new(Arc::clone(&platform), Arc::clone(&state)),
            mouse:           MouseModule::new(Arc::clone(&platform), Arc::clone(&state)),
            media:           MediaModule::new(Arc::clone(&platform)),
            virtual_desktop: VirtualDesktopModule::new(Arc::clone(&platform)),
            voice:           VoiceModule::with_stt_engine(
                Arc::clone(&platform),
                cfg.stt_engine.clone(),
            ).with_llm_config(
                best_key,
                best_model,
                cfg.stt_correction,
            ),
            window_manager:  WindowManagerModule::new(Arc::clone(&platform), Arc::clone(&state)),
            platform,
        };
        let ww_cfg = wake_word::WakeWordConfig {
            enabled:       cfg.wake_word_enabled,
            model_dir:     cfg.wake_word_model_dir.clone(),
            keywords_file: cfg.wake_word_keywords_file.clone(),
            threshold:     cfg.wake_word_threshold,
            hold_ms:       cfg.wake_word_hold_ms,
        };
        drop(cfg);
        // Preload Whisper model in background so first Space+V is instant.
        s.voice.preload();
        // Start wake-word listener (no-op unless enabled + paths valid).
        s.voice.start_wake_word(ww_cfg);
        s
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        // Space+Comma → open preferences (like AHK implementation).
        if key == KeyCode::Comma {
            self.platform.open_preferences();
            return true;
        }

        // Space+Slash → toggle keyboard layout HUD.
        if key == KeyCode::Slash {
            self.platform.toggle_keyboard_layout_hud();
            return true;
        }

        // Core modules (keyboard/mouse) — must NEVER crash. Run directly.
        if self.edit.on_key_down(key, &*self.platform) { return true; }
        if self.mouse.on_key_down(key) { return true; }
        if self.media.on_key_down(key) { return true; }
        if self.virtual_desktop.on_key_down(key, mods) { return true; }
        if self.window_manager.on_key_down(key, mods) { return true; }

        // Heavy modules (LLM/voice/agent) — isolated with catch_unwind.
        // A panic here logs an error but does NOT crash the core.
        if safe_call("agent", || self.agent.on_key_down(key, mods)) { return true; }
        if safe_call("brainstorm", || self.brainstorm.on_key_down(key, mods)) { return true; }
        if safe_call("voice", || self.voice.on_key_down(key)) { return true; }
        false
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if self.edit.on_key_up(key) { return true; }
        if self.mouse.on_key_up(key) { return true; }
        if self.media.on_key_up(key) { return true; }
        if self.window_manager.on_key_up(key) { return true; }

        if safe_call("agent", || self.agent.on_key_up(key)) { return true; }
        if safe_call("brainstorm", || self.brainstorm.on_key_up(key)) { return true; }
        if safe_call("voice", || self.voice.on_key_up(key)) { return true; }
        false
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::Comma  // Space+Comma = preferences
            || key == KeyCode::Slash  // Space+Slash = keyboard layout HUD
            || self.edit.is_mapped_key(key)
            || self.mouse.is_mapped_key(key)
            || self.media.is_mapped_key(key)
            || self.virtual_desktop.is_mapped_key(key)
            || self.window_manager.is_mapped_key(key)
            || self.agent.is_mapped_key(key)
            || self.brainstorm.is_mapped_key(key)
            || self.voice.is_mapped_key(key)
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.edit .apply_speeds(s);
        self.mouse.apply_speeds(s);
        self.window_manager.apply_speeds(s);
    }

    /// Hot-reload voice/brainstorm config from updated preferences.
    pub fn apply_config(&self, cfg: &ClxConfig) {
        let (best_key, best_model) = cfg.best_llm_key_and_model();
        self.voice.update_config(
            cfg.stt_engine.clone(),
            best_key.clone(),
            best_model.clone(),
            cfg.stt_correction,
            cfg.tts_chain.clone(),
            cfg.stt_polish_chain.clone(),
            cfg.aec_gain,
            cfg.noise_gate,
            cfg.speech_start_prob,
            cfg.speech_end_prob,
            cfg.speech_start_frames,
            cfg.silence_end_frames,
            cfg.aec_mode.clone(),
        );
        self.brainstorm.update_llm_config(&best_key, &best_model);
    }

    /// Advance all AccModel physics by one step (WASM adapter tick loop).
    pub fn tick(&self) {
        self.edit.tick();
        self.mouse.tick();
        self.window_manager.tick();
    }

    /// Stop all ongoing AccModel physics (called when CLX mode exits).
    pub fn stop_all(&self) {
        self.edit.stop();
        self.mouse.stop();
        self.window_manager.stop();
        self.voice.stop();
    }
}
