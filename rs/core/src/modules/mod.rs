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
#[cfg(test)]
mod voice_ptt_test;
#[cfg(not(feature = "stt"))]
mod voice {
    //! Otoji-only voice module for builds without in-process STT (e.g.
    //! Windows, where whisper-rs 0.13 fails to compile on MSVC).
    //! Delegates all speech recognition to the external `otoji` binary
    //! via voice_otoji + voice_ptt.
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::time::Instant;
    use crate::key_code::KeyCode;
    use crate::platform::{Platform, PttTrayState};

    pub struct VoiceModule {
        platform: Arc<dyn Platform>,
        otoji: Arc<super::voice_otoji::OtojiBackend>,
        otoji_available: bool,
        otoji_typed: Arc<Mutex<String>>,
        ptt: Arc<super::voice_ptt::PttSession>,
        input_active: Arc<AtomicBool>,
        v_held: Arc<AtomicBool>,
        note_active: Arc<AtomicBool>,
        press_time: Mutex<Option<Instant>>,
    }

    impl VoiceModule {
        pub fn with_stt_engine(platform: Arc<dyn Platform>, _stt_engine: String) -> Self {
            let otoji = Arc::new(super::voice_otoji::OtojiBackend::new());
            let ptt = super::voice_ptt::PttSession::new(
                Arc::clone(&platform),
                Arc::clone(&otoji),
            );
            let available = super::voice_otoji::OtojiBackend::is_available();
            if available {
                eprintln!("[CLX] voice(otoji-only): otoji detected on PATH");
            } else {
                eprintln!("[CLX] voice(otoji-only): otoji NOT found — Space+V disabled");
            }
            Self {
                platform,
                otoji,
                otoji_available: available,
                otoji_typed: Arc::new(Mutex::new(String::new())),
                ptt,
                input_active: Arc::new(AtomicBool::new(false)),
                v_held: Arc::new(AtomicBool::new(false)),
                note_active: Arc::new(AtomicBool::new(false)),
                press_time: Mutex::new(None),
            }
        }

        pub fn with_llm_config(self, _api_key: String, _model: String, _correction: bool) -> Self {
            self
        }

        pub fn preload(&self) {
            if self.otoji_available {
                eprintln!("[CLX] voice(otoji-only): skipping in-process STT preload");
            }
        }

        pub fn on_key_down(&self, key: KeyCode) -> bool {
            if key != KeyCode::V { return false; }
            if !self.otoji_available { return false; }

            *self.press_time.lock().unwrap() = Some(Instant::now());
            self.v_held.store(true, Ordering::Relaxed);

            if self.ptt.on_press() {
                return true;
            }

            self.ensure_otoji_running();
            true
        }

        pub fn on_key_up(&self, key: KeyCode) -> bool {
            if key != KeyCode::V { return false; }
            if !self.otoji_available { return false; }

            self.v_held.store(false, Ordering::Relaxed);

            let result = self.ptt.on_release();
            match result {
                super::voice_ptt::PttRelease::Hold => {
                    eprintln!("[CLX] voice: hold release → waiting for otoji ptt_final");
                    if !self.note_active.load(Ordering::Relaxed) {
                        self.platform.hide_voice_overlay();
                    }
                }
                super::voice_ptt::PttRelease::Locked => {
                    eprintln!("[CLX] voice: double-tap → PTT locked mode");
                }
                super::voice_ptt::PttRelease::Tap => {
                    if self.note_active.load(Ordering::Relaxed) {
                        self.note_active.store(false, Ordering::Relaxed);
                        self.platform.hide_voice_overlay();
                        self.platform.set_ptt_tray_state(PttTrayState::Idle);
                    } else {
                        self.note_active.store(true, Ordering::Relaxed);
                        self.platform.show_voice_overlay();
                        self.platform.set_ptt_tray_state(PttTrayState::NoteMode);
                    }
                }
            }
            true
        }

        pub fn is_mapped_key(&self, key: KeyCode) -> bool {
            key == KeyCode::V && self.otoji_available
        }

        pub fn stop(&self) {
            self.otoji.stop();
        }

        #[allow(clippy::too_many_arguments)]
        pub fn update_config(
            &self,
            _stt_engine: String, _api_key: String, _model: String, _correction: bool,
            _tts_chain: String, _stt_polish_chain: String,
            _aec_gain: f32, _noise_gate: f32,
            _speech_start_prob: f32, _speech_end_prob: f32,
            _speech_start_frames: usize, _silence_end_frames: usize,
        ) {}

        fn ensure_otoji_running(&self) {
            if self.otoji.is_running() { return; }

            eprintln!("[CLX] voice(otoji-only): launching otoji backend");
            self.platform.show_voice_overlay();
            self.platform.update_voice_subtitle("otoji starting...");
            *self.otoji_typed.lock().unwrap() = String::new();

            let otoji = Arc::clone(&self.otoji);
            let platform = Arc::clone(&self.platform);
            let input_active = Arc::clone(&self.input_active);
            let otoji_typed = Arc::clone(&self.otoji_typed);
            let ptt = Arc::clone(&self.ptt);
            std::thread::Builder::new()
                .name("otoji-launch".into())
                .spawn(move || {
                    if !otoji.start(platform.clone(), input_active, otoji_typed, Some(ptt)) {
                        eprintln!("[CLX] voice(otoji-only): otoji failed to start");
                        platform.update_voice_subtitle("otoji failed");
                    }
                })
                .ok();
        }
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
        drop(cfg);
        // Preload Whisper model in background so first Space+V is instant.
        s.voice.preload();
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
