pub mod edit;
pub mod media;
pub mod mouse;
pub mod virtual_desktop;
pub mod window_manager;

#[cfg(not(target_arch = "wasm32"))]
pub mod agent;
#[cfg(not(target_arch = "wasm32"))]
pub mod brainstorm;
#[cfg(not(target_arch = "wasm32"))]
pub mod voice;

use std::sync::Arc;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;
use crate::state::{ClxConfig, ClxState, SpeedConfig};

use edit::EditModule;
use media::MediaModule;
use mouse::MouseModule;
use virtual_desktop::VirtualDesktopModule;
use window_manager::WindowManagerModule;

#[cfg(not(target_arch = "wasm32"))]
use agent::AgentModule;
#[cfg(not(target_arch = "wasm32"))]
use brainstorm::BrainstormModule;
#[cfg(not(target_arch = "wasm32"))]
use voice::VoiceModule;

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
    #[cfg(not(target_arch = "wasm32"))]
    pub agent:       AgentModule,
    #[cfg(not(target_arch = "wasm32"))]
    pub brainstorm:  BrainstormModule,
    edit:            EditModule,
    mouse:           MouseModule,
    media:           MediaModule,
    virtual_desktop: VirtualDesktopModule,
    #[cfg(not(target_arch = "wasm32"))]
    voice:           VoiceModule,
    window_manager:  WindowManagerModule,
    platform:        Arc<dyn Platform>,
}

impl Modules {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let cfg_guard = state.config.read().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        let cfg = &*cfg_guard;
        #[cfg(not(target_arch = "wasm32"))]
        let (best_key, best_model) = cfg.best_llm_key_and_model();

        let s = Self {
            #[cfg(not(target_arch = "wasm32"))]
            agent:       AgentModule::new(Arc::clone(&platform)),
            #[cfg(not(target_arch = "wasm32"))]
            brainstorm:  BrainstormModule::new(
                Arc::clone(&platform),
                best_key.clone(),
                best_model.clone(),
            ),
            edit:        EditModule::new(Arc::clone(&platform), Arc::clone(&state)),
            mouse:       MouseModule::new(Arc::clone(&platform), Arc::clone(&state)),
            media:       MediaModule::new(Arc::clone(&platform)),
            virtual_desktop: VirtualDesktopModule::new(Arc::clone(&platform)),
            #[cfg(not(target_arch = "wasm32"))]
            voice:       VoiceModule::with_stt_engine(
                Arc::clone(&platform),
                cfg.stt_engine.clone(),
            ).with_llm_config(
                best_key,
                best_model,
                cfg.stt_correction,
            ),
            window_manager: WindowManagerModule::new(Arc::clone(&platform), Arc::clone(&state)),
            platform,
        };
        // NOTE: voice model is loaded lazily on first Space+V press (~3s startup).
        // Eager preload was removed to reduce memory footprint (SenseVoice = ~2.4GB RAM).
        s
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        // Space+Comma → open preferences (like AHK implementation).
        if key == KeyCode::Comma {
            self.platform.open_preferences();
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
        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("agent", || self.agent.on_key_down(key, mods)) { return true; }
        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("brainstorm", || self.brainstorm.on_key_down(key, mods)) { return true; }
        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("voice", || self.voice.on_key_down(key)) { return true; }
        false
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if self.edit.on_key_up(key) { return true; }
        if self.mouse.on_key_up(key) { return true; }
        if self.media.on_key_up(key) { return true; }
        if self.window_manager.on_key_up(key) { return true; }

        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("agent", || self.agent.on_key_up(key)) { return true; }
        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("brainstorm", || self.brainstorm.on_key_up(key)) { return true; }
        #[cfg(not(target_arch = "wasm32"))]
        if safe_call("voice", || self.voice.on_key_up(key)) { return true; }
        false
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        let native = {
            #[cfg(not(target_arch = "wasm32"))]
            { self.agent.is_mapped_key(key) || self.brainstorm.is_mapped_key(key) || self.voice.is_mapped_key(key) }
            #[cfg(target_arch = "wasm32")]
            { false }
        };
        key == KeyCode::Comma  // Space+Comma = preferences
            || self.edit.is_mapped_key(key)
            || self.mouse.is_mapped_key(key)
            || self.media.is_mapped_key(key)
            || self.virtual_desktop.is_mapped_key(key)
            || self.window_manager.is_mapped_key(key)
            || native
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.edit.apply_speeds(s);
        self.mouse.apply_speeds(s);
        self.window_manager.apply_speeds(s);
    }

    /// Hot-reload voice/brainstorm config from updated preferences.
    pub fn apply_config(&self, cfg: &ClxConfig) {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
            );
            self.brainstorm.update_llm_config(&best_key, &best_model);
        }
        let _ = cfg;
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
        #[cfg(not(target_arch = "wasm32"))]
        self.voice.stop();
    }
}
