pub mod agent;
pub mod brainstorm;
pub mod edit;
pub mod fn_row;
pub mod media;
pub mod mouse;
pub mod rdp_escape;

/// Build the runaway watchdog for an `AccModel2D` driven by `keys`.
///
/// Every physics model in CLX is "hold to keep moving", and every one of them
/// used to have exactly one brake: the key-UP reaching the engine. That brake
/// fails whenever the foreground window belongs to a higher-integrity process
/// — an elevated app, or the DWM-owned `Ghost` window Windows raises in front
/// of an unresponsive one — because a non-elevated `WH_KEYBOARD_LL` hook then
/// receives nothing at all. The direction stays latched, the model integrates
/// up to `max_speed`, and a single tap becomes an endless stream of commands.
///
/// `GetAsyncKeyState` still tells the truth in that state, because it reads
/// hardware rather than the (now silent) event pipe. Measured cost on Windows:
/// ~350 ns per call, ~0.01% of a core at the ticker's rate — free next to the
/// sub-millisecond spin the ticker already does.
///
/// Self-calibration: `Platform::is_key_physically_down` defaults to a flat
/// `false` for adapters that cannot answer, so a bare negative would stop
/// every model instantly on those platforms. The verdict is therefore only
/// believed once the same API has been seen to return `true` at least once —
/// the same rule the Space auto-repeat fix uses in `engine.rs`.
/// **DISABLED 2026-09-22 — do not re-enable without reading this.**
///
/// The premise below is wrong for the keys CLX actually drives. The hook
/// *suppresses* those keys (`LRESULT(1)`), and a suppressed key does not reach
/// the asynchronous key state, so `GetAsyncKeyState` reports it **up while the
/// user is holding it down**. The watchdog therefore fired constantly during
/// ordinary use — `crash.log` filled with `[on_input_lost] ... blocked=false`
/// against Chrome, Terminal and Firefox — and killed hold-to-repeat mid-
/// gesture, which is most of what CLX does. Symptom: clx looks dead.
///
/// The runaway it was meant to stop is real (see
/// `tmp/clx-z-runaway-handoff.md`); the detection mechanism has to be
/// something the hook has not already swallowed. Call sites are commented out
/// rather than deleted so the replacement has a place to land.
pub fn key_watchdog(
    platform: std::sync::Arc<dyn Platform>,
    keys: &'static [KeyCode],
) -> std::sync::Arc<crate::acc_model::WatchdogFn> {
    use std::sync::atomic::{AtomicBool, Ordering};
    let trusted = AtomicBool::new(false);
    std::sync::Arc::new(move || {
        if keys.iter().any(|k| platform.is_key_physically_down(*k)) {
            trusted.store(true, Ordering::Relaxed);
            return true;
        }
        // Nothing reports down. Only act on that if this adapter has ever
        // reported `true` — otherwise it simply cannot answer the question.
        if !trusted.load(Ordering::Relaxed) {
            return true;
        }
        // Reaching here means the model is still latched while the hardware
        // says the key is up: the key-up event was swallowed. That is not a
        // normal release (a normal one calls `stop()` and the model is never
        // ticked again), so it is worth telling the user about.
        platform.on_input_lost();
        false
    })
}
pub mod virtual_desktop;
#[cfg(feature = "stt")]
pub mod voice;
pub mod voice_otoji;
pub mod voice_ptt;
#[cfg(test)]
mod voice_ptt_test;
pub mod wake_word;
#[cfg(not(feature = "stt"))]
mod voice {
    //! No-op stub used when the `stt` feature is disabled (e.g. Windows
    //! builds, where `whisper-rs 0.13` fails to compile). Mirrors the
    //! public surface of `VoiceModule` so the rest of `Modules` compiles
    //! unchanged. All hotkeys silently fall through.
    use crate::key_code::KeyCode;
    use crate::platform::Platform;
    use std::sync::Arc;

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
        pub fn on_key_down(&self, _key: KeyCode) -> bool {
            false
        }
        pub fn on_key_up(&self, _key: KeyCode) -> bool {
            false
        }
        pub fn is_mapped_key(&self, _key: KeyCode) -> bool {
            false
        }
        pub fn stop(&self) {}
        #[allow(clippy::too_many_arguments)]
        pub fn update_config(
            &self,
            _stt_engine: String,
            _api_key: String,
            _model: String,
            _correction: bool,
            _tts_chain: String,
            _stt_polish_chain: String,
            _aec_gain: f32,
            _noise_gate: f32,
            _speech_start_prob: f32,
            _speech_end_prob: f32,
            _speech_start_frames: usize,
            _silence_end_frames: usize,
            _aec_mode: String,
            _whisper_model_path: String,
            _whisper_language: String,
            _ptt_vad_auto_release_ms: u64,
        ) {
        }
    }
}
pub mod window_manager;

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;
use crate::state::{ClxConfig, ClxState, SpeedConfig};
use std::sync::Arc;

use agent::AgentModule;
use brainstorm::BrainstormModule;
use edit::EditModule;
use fn_row::FnRowModule;
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
            eprintln!(
                "[CLX] PANIC in {} module (isolated, core unaffected): {}",
                module, msg
            );
            false
        }
    }
}

pub struct Modules {
    pub agent: AgentModule,
    pub brainstorm: BrainstormModule,
    edit: EditModule,
    fn_row: FnRowModule,
    mouse: MouseModule,
    media: MediaModule,
    virtual_desktop: VirtualDesktopModule,
    voice: VoiceModule,
    window_manager: WindowManagerModule,
    platform: Arc<dyn Platform>,
}

impl Modules {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let cfg = state.config.read().unwrap();
        let (best_key, best_model) = cfg.best_llm_key_and_model();
        // Brainstorm is local-first (Ollama by default); voice/STT keep the
        // cloud-priority key above.
        let (bs_key, bs_model) = cfg.brainstorm_llm_key_and_model();
        let s = Self {
            agent: AgentModule::new(Arc::clone(&platform)),
            brainstorm: BrainstormModule::new(Arc::clone(&platform), bs_key, bs_model),
            edit: EditModule::new(Arc::clone(&platform), Arc::clone(&state)),
            fn_row: FnRowModule::new(Arc::clone(&platform), Arc::clone(&state)),
            mouse: MouseModule::new(Arc::clone(&platform), Arc::clone(&state)),
            media: MediaModule::new(Arc::clone(&platform)),
            virtual_desktop: VirtualDesktopModule::new(Arc::clone(&platform), Arc::clone(&state)),
            voice: VoiceModule::with_stt_engine(Arc::clone(&platform), cfg.stt_engine.clone())
                .with_llm_config(best_key, best_model, cfg.stt_correction),
            window_manager: WindowManagerModule::new(Arc::clone(&platform), Arc::clone(&state)),
            platform,
        };
        let ww_cfg = wake_word::WakeWordConfig {
            enabled: cfg.wake_word_enabled,
            model_dir: cfg.wake_word_model_dir.clone(),
            keywords_file: cfg.wake_word_keywords_file.clone(),
            threshold: cfg.wake_word_threshold,
            hold_ms: cfg.wake_word_hold_ms,
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
        // The F-row layer runs before virtual_desktop: both claim the number
        // row, and the chord is the more specific gesture.
        if self.fn_row.on_key_down(key, mods) {
            return true;
        }
        if self.edit.on_key_down(key, &*self.platform) {
            return true;
        }
        if self.mouse.on_key_down(key) {
            return true;
        }
        if self.media.on_key_down(key) {
            return true;
        }
        if self.virtual_desktop.on_key_down(key, mods) {
            return true;
        }
        if self.window_manager.on_key_down(key, mods) {
            return true;
        }

        // Heavy modules (LLM/voice/agent) — isolated with catch_unwind.
        // A panic here logs an error but does NOT crash the core.
        if safe_call("agent", || self.agent.on_key_down(key, mods)) {
            return true;
        }
        if safe_call("brainstorm", || self.brainstorm.on_key_down(key, mods)) {
            return true;
        }
        if safe_call("voice", || self.voice.on_key_down(key)) {
            return true;
        }
        false
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if self.edit.on_key_up(key) {
            return true;
        }
        if self.mouse.on_key_up(key) {
            return true;
        }
        if self.media.on_key_up(key) {
            return true;
        }
        if self.window_manager.on_key_up(key) {
            return true;
        }

        if safe_call("agent", || self.agent.on_key_up(key)) {
            return true;
        }
        if safe_call("brainstorm", || self.brainstorm.on_key_up(key)) {
            return true;
        }
        if safe_call("voice", || self.voice.on_key_up(key)) {
            return true;
        }
        false
    }

    /// Release only the input modules after an adapter observes a missed UP.
    /// Keep this free of keyboard injection: it may run off the hook thread.
    pub fn release_missed_key(&self, key: KeyCode) {
        if !self.edit.on_key_up(key) && !self.mouse.on_key_up(key) {
            self.window_manager.on_key_up(key);
        }
    }

    pub fn refresh_held_key(&self, key: KeyCode) {
        self.edit.refresh_held_key(key);
        self.mouse.refresh_held_key(key);
        self.window_manager.refresh_held_key(key);
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::Comma  // Space+Comma = preferences
            || key == KeyCode::Slash  // Space+Slash = keyboard layout HUD
            || self.fn_row.is_mapped_key(key)
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
        self.edit.apply_speeds(s);
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
            cfg.whisper_model_path.clone(),
            cfg.whisper_language.clone(),
            cfg.ptt_vad_auto_release_ms,
        );
        let (bs_key, bs_model) = cfg.brainstorm_llm_key_and_model();
        self.brainstorm.update_llm_config(&bs_key, &bs_model);
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
