//! Otoji-only voice module — the `VoiceModule` compiled when the `stt`
//! feature is off (Windows builds, where whisper-rs / sherpa-rs don't build).
//!
//! Space+V behaves like the full module's otoji path in `voice.rs`:
//!   hold       — push-to-talk: otoji transcribes, clx types at the cursor
//!   tap        — toggle note mode (live subtitles in the overlay)
//!   double-tap — locked PTT (tap again to commit)
//!
//! All audio capture + STT lives in the external `otoji listen` process
//! (`voice_otoji.rs`); PTT typing is driven by `voice_ptt.rs`. There is no
//! in-process STT fallback: when `otoji` isn't on PATH the hotkey just shows
//! an install hint in the overlay.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::key_code::KeyCode;
use crate::platform::{Platform, PttTrayState};

use super::voice_otoji::{self, OtojiBackend, TrayState};
use super::voice_ptt::{PttRelease, PttSession};
use super::wake_word::{WakeWordConfig, WakeWordListener};

/// Warm-pool idle window after note mode stops (see `arm_idle_stop`).
const IDLE_MS: u64 = 30_000;
const IDLE_TICK_MS: u64 = 1_000;

/// Settings forwarded to `OtojiBackend::start`, hot-reloadable from prefs.
struct LiveConfig {
    stt_engine: String,
    whisper_model_path: String,
    whisper_language: String,
    aec_mode: String,
}

pub struct VoiceModule {
    platform: Arc<dyn Platform>,
    otoji: Arc<OtojiBackend>,
    ptt: Arc<PttSession>,
    /// Text typed by the continuous-listen path. PTT owns typing, so this
    /// stays empty (`input_active` is never raised) — kept for the
    /// `OtojiBackend::start` signature.
    otoji_typed: Arc<Mutex<String>>,
    input_active: Arc<AtomicBool>,
    /// Note mode (tap V): overlay shows live subtitles, nothing is typed.
    note_active: Arc<AtomicBool>,
    /// Generation counter for the warm-pool idle teardown; a new press or a
    /// hard stop bumps it so a pending timer aborts.
    idle_gen: Arc<AtomicU64>,
    wake_word_listener: Mutex<Option<WakeWordListener>>,
    live_config: Mutex<LiveConfig>,
}

impl VoiceModule {
    pub fn with_stt_engine(platform: Arc<dyn Platform>, stt_engine: String) -> Self {
        let otoji = Arc::new(OtojiBackend::new());
        let ptt = PttSession::new(Arc::clone(&platform), Arc::clone(&otoji));
        Self {
            platform,
            otoji,
            ptt,
            otoji_typed: Arc::new(Mutex::new(String::new())),
            input_active: Arc::new(AtomicBool::new(false)),
            note_active: Arc::new(AtomicBool::new(false)),
            idle_gen: Arc::new(AtomicU64::new(0)),
            wake_word_listener: Mutex::new(None),
            live_config: Mutex::new(LiveConfig {
                stt_engine,
                whisper_model_path: String::new(),
                whisper_language: "ja".to_string(),
                aec_mode: "always".to_string(),
            }),
        }
    }

    /// LLM settings only matter to the in-process polish path, which this
    /// build doesn't have (otoji polishes on its own). Accepted for API parity.
    pub fn with_llm_config(self, _api_key: String, _model: String, _correction: bool) -> Self {
        self
    }

    pub fn start_wake_word(&self, cfg: WakeWordConfig) {
        let new = WakeWordListener::try_start(Arc::clone(&self.ptt), cfg);
        // Replacing drops the old listener; its Drop kills the otoji kws child.
        *self.wake_word_listener.lock().unwrap() = new;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_config(
        &self,
        stt_engine: String,
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
        aec_mode: String,
        whisper_model_path: String,
        whisper_language: String,
        ptt_vad_auto_release_ms: u64,
    ) {
        self.ptt.set_vad_auto_release_ms(ptt_vad_auto_release_ms);
        let mut cfg = self.live_config.lock().unwrap();
        cfg.stt_engine = stt_engine;
        cfg.whisper_model_path = whisper_model_path;
        cfg.whisper_language = whisper_language;
        cfg.aec_mode = aec_mode;
        eprintln!(
            "[CLX] voice-lite: config hot-reloaded (engine={}, whisper_lang={}, vad_release={}ms, aec_mode={})",
            cfg.stt_engine, cfg.whisper_language, ptt_vad_auto_release_ms, cfg.aec_mode
        );
    }

    /// Pre-warm otoji at startup (standby) when `CLX_PTT_PREWARM` is set;
    /// otherwise the first Space+V spawns it.
    pub fn preload(&self) {
        if !voice_otoji::prewarm_enabled() || !OtojiBackend::is_available() {
            return;
        }
        eprintln!("[CLX] voice-lite: pre-warming otoji at startup (standby)");
        self.spawn_otoji();
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    /// Runs inside the low-level keyboard hook — nothing here may block.
    /// Subprocess spawns happen on a background thread.
    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }
        // Hand off to the out-of-process voice host (clx-voice.exe) when the
        // platform runs one. Keeps the always-on hook process a thin trigger:
        // otoji/STT live in the host, which can be rebuilt without restarting
        // core. Falls through to the in-process path if the host is unreachable.
        if self.platform.voice_delegate(true) {
            return true;
        }
        eprintln!("[CLX] voice-lite: V pressed");
        // Locked-mode exit: the press commits + unlocks, nothing else to do.
        if self.ptt.on_press() {
            return true;
        }
        self.ensure_otoji_running();
        true
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }
        if self.platform.voice_delegate(false) {
            return true;
        }
        match self.ptt.on_release() {
            PttRelease::Hold => {
                eprintln!("[CLX] voice-lite: hold release → waiting for otoji ptt_final");
                if !self.note_active.load(Ordering::Relaxed) {
                    self.platform.hide_voice_overlay();
                }
            }
            PttRelease::Locked => {
                eprintln!("[CLX] voice-lite: double-tap → PTT locked mode");
            }
            PttRelease::Tap => {
                if self.note_active.swap(false, Ordering::Relaxed) {
                    // Leaving long-text listening input mode: stop typing
                    // committed sentences into the field.
                    self.input_active.store(false, Ordering::Relaxed);
                    eprintln!("[CLX] voice-lite: tap → listening-input stopped");
                    if voice_otoji::prewarm_enabled() {
                        self.otoji.send_control("STANDBY");
                    } else {
                        self.arm_idle_stop();
                    }
                    self.platform.hide_voice_overlay();
                    self.platform.set_ptt_tray_state(PttTrayState::Idle);
                } else {
                    self.note_active.store(true, Ordering::Relaxed);
                    // Long-text listening input mode: each committed sentence
                    // is appended at the cursor (otoji reader, gated on
                    // input_active); the live partial previews in the overlay.
                    self.input_active.store(true, Ordering::Relaxed);
                    eprintln!("[CLX] voice-lite: tap → listening-input started");
                    if voice_otoji::prewarm_enabled() {
                        self.otoji.send_control("RESUME");
                    }
                    self.platform.show_voice_overlay();
                    self.platform.set_ptt_tray_state(PttTrayState::NoteMode);
                    // Cold start: the tap that turns the mode on won't have
                    // spawned otoji (on_press → ensure_otoji_running only runs
                    // before a hold). Bring it up now so listening begins.
                    self.ensure_otoji_running();
                }
            }
        }
        true
    }

    /// Called when CLX mode deactivates. Normally V is released first and
    /// `on_key_up` commits; if the trigger went up while V is still held, the
    /// V key-up will pass through unrouted, so commit the segment here. A
    /// bare tap in this situation just cancels — it never toggles note mode.
    pub fn stop(&self) {
        if !self.ptt.is_active() || self.ptt.is_locked() {
            return;
        }
        eprintln!("[CLX] voice-lite: CLX deactivated during PTT hold → committing");
        let _ = self.ptt.on_release();
        if !self.note_active.load(Ordering::Relaxed) {
            self.platform.hide_voice_overlay();
        }
    }

    pub fn is_listening(&self) -> bool {
        self.note_active.load(Ordering::Relaxed) || self.ptt.is_active()
    }

    // ── Internal ────────────────────────────────────────────────────────────

    fn ensure_otoji_running(&self) {
        // A new press supersedes any pending idle teardown.
        self.idle_gen.fetch_add(1, Ordering::Relaxed);
        voice_otoji::notify_tray(TrayState::Starting);
        if self.otoji.is_running() {
            return;
        }
        // Re-check on every cold start (cheap PATH scan, no subprocess) so
        // installing otoji after clx launched works without a restart.
        if !OtojiBackend::is_available() {
            eprintln!("[CLX] voice-lite: `otoji` not found on PATH");
            self.platform.show_voice_overlay();
            self.platform.update_voice_subtitle(
                "otoji not found — install it (cargo install otoji) and put it on PATH",
            );
            let platform = Arc::clone(&self.platform);
            std::thread::Builder::new()
                .name("voice-hint-hide".into())
                .spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(4));
                    platform.hide_voice_overlay();
                })
                .ok();
            return;
        }
        eprintln!("[CLX] voice-lite: launching otoji backend (bg thread)");
        self.platform.show_voice_overlay();
        self.platform.update_voice_subtitle("otoji starting...");
        self.spawn_otoji();
    }

    /// Spawn `otoji listen` on a background thread — `Command::spawn` takes
    /// far longer than the hook callback budget allows.
    fn spawn_otoji(&self) {
        *self.otoji_typed.lock().unwrap() = String::new();
        let (aec_enabled, stt_engine, whisper_model_path, whisper_language) = {
            let lc = self.live_config.lock().unwrap();
            (
                lc.aec_mode == "always",
                lc.stt_engine.clone(),
                lc.whisper_model_path.clone(),
                lc.whisper_language.clone(),
            )
        };
        let otoji = Arc::clone(&self.otoji);
        let platform = Arc::clone(&self.platform);
        let input_active = Arc::clone(&self.input_active);
        let otoji_typed = Arc::clone(&self.otoji_typed);
        let ptt = Arc::clone(&self.ptt);
        std::thread::Builder::new()
            .name("otoji-launch".into())
            .spawn(move || {
                if !otoji.start(
                    Arc::clone(&platform),
                    input_active,
                    otoji_typed,
                    Some(ptt),
                    aec_enabled,
                    stt_engine,
                    whisper_model_path,
                    whisper_language,
                ) {
                    eprintln!("[CLX] voice-lite: otoji failed to start");
                    platform.update_voice_subtitle("otoji failed to start");
                }
            })
            .ok();
    }

    /// Warm-pool teardown after note mode stops: keep otoji alive for
    /// `IDLE_MS` so a quick re-toggle or follow-up PTT reuses it without a
    /// model reload, then release the mic. A newer press bumps `idle_gen`
    /// and aborts the timer.
    fn arm_idle_stop(&self) {
        if voice_otoji::prewarm_enabled() {
            return;
        }
        let my_gen = self.idle_gen.fetch_add(1, Ordering::Relaxed) + 1;
        let idle_gen = Arc::clone(&self.idle_gen);
        let otoji = Arc::clone(&self.otoji);
        let platform = Arc::clone(&self.platform);
        let note_active = Arc::clone(&self.note_active);
        let ptt = Arc::clone(&self.ptt);
        std::thread::Builder::new()
            .name("otoji-idle-stop".into())
            .spawn(move || {
                let mut idle_elapsed = 0u64;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(IDLE_TICK_MS));
                    if idle_gen.load(Ordering::Relaxed) != my_gen {
                        return;
                    }
                    if note_active.load(Ordering::Relaxed) || ptt.is_active() {
                        idle_elapsed = 0;
                        continue;
                    }
                    idle_elapsed += IDLE_TICK_MS;
                    if idle_elapsed < IDLE_MS {
                        continue;
                    }
                    if idle_gen.load(Ordering::Relaxed) != my_gen {
                        return;
                    }
                    eprintln!(
                        "[CLX] voice-lite: otoji idle {}s → warm-pool teardown",
                        IDLE_MS / 1000
                    );
                    voice_otoji::notify_tray(TrayState::Idle);
                    if otoji.is_running() {
                        otoji.stop();
                        platform.hide_voice_overlay();
                    }
                    return;
                }
            })
            .ok();
    }
}
