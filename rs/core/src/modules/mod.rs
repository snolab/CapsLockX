pub mod brainstorm;
pub mod edit;
pub mod media;
pub mod mouse;
pub mod virtual_desktop;
pub mod voice;
pub mod window_manager;

use std::sync::Arc;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;
use crate::state::{ClxConfig, ClxState, SpeedConfig};

use brainstorm::BrainstormModule;
use edit::EditModule;
use media::MediaModule;
use mouse::MouseModule;
use virtual_desktop::VirtualDesktopModule;
use voice::VoiceModule;
use window_manager::WindowManagerModule;

pub struct Modules {
    brainstorm:      BrainstormModule,
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
        let s = Self {
            brainstorm:      BrainstormModule::new(
                Arc::clone(&platform),
                cfg.brainstorm_origin.clone(),
                cfg.llm_api_key.clone(),
                cfg.llm_model.clone(),
            ),
            edit:            EditModule::new(Arc::clone(&platform), Arc::clone(&state)),
            mouse:           MouseModule::new(Arc::clone(&platform), Arc::clone(&state)),
            media:           MediaModule::new(Arc::clone(&platform)),
            virtual_desktop: VirtualDesktopModule::new(Arc::clone(&platform)),
            voice:           VoiceModule::with_stt_engine(
                Arc::clone(&platform),
                cfg.stt_engine.clone(),
            ).with_llm_config(
                cfg.llm_api_key.clone(),
                cfg.llm_model.clone(),
                cfg.stt_correction,
            ),
            window_manager:  WindowManagerModule::new(Arc::clone(&platform)),
            platform,
        };
        drop(cfg);
        // Preload Whisper model in background so first Space+V is instant.
        s.voice.preload();
        s
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        self.brainstorm.on_key_down(key, mods)
            || self.edit.on_key_down(key, &*self.platform)
            || self.mouse.on_key_down(key)
            || self.media.on_key_down(key)
            || self.virtual_desktop.on_key_down(key, mods)
            || self.voice.on_key_down(key)
            || self.window_manager.on_key_down(key, mods)
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        self.brainstorm.on_key_up(key)
            || self.edit.on_key_up(key)
            || self.mouse.on_key_up(key)
            || self.media.on_key_up(key)
            || self.voice.on_key_up(key)
            || self.window_manager.on_key_up(key)
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        self.brainstorm.is_mapped_key(key)
            || self.edit.is_mapped_key(key)
            || self.mouse.is_mapped_key(key)
            || self.media.is_mapped_key(key)
            || self.virtual_desktop.is_mapped_key(key)
            || self.voice.is_mapped_key(key)
            || self.window_manager.is_mapped_key(key)
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.edit .apply_speeds(s);
        self.mouse.apply_speeds(s);
    }

    /// Hot-reload voice/brainstorm config from updated preferences.
    pub fn apply_config(&self, cfg: &ClxConfig) {
        self.voice.update_config(
            cfg.stt_engine.clone(),
            cfg.llm_api_key.clone(),
            cfg.llm_model.clone(),
            cfg.stt_correction,
        );
        self.brainstorm.update_llm_config(&cfg.llm_api_key, &cfg.llm_model);
    }

    /// Advance all AccModel physics by one step (WASM adapter tick loop).
    pub fn tick(&self) {
        self.edit.tick();
        self.mouse.tick();
    }

    /// Stop all ongoing AccModel physics (called when CLX mode exits).
    pub fn stop_all(&self) {
        self.edit.stop();
        self.mouse.stop();
        self.voice.stop();
    }
}
