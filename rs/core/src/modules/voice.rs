/// CLX-Voice -- V key toggles voice listening (toggle / hold modes).
///
/// Architecture: dual-track STT worker
///   Audio loop (capture thread): AudioCapture -> resample -> NLMS echo cancel -> VAD -> push AudioChunk
///   STT worker thread: receive AudioChunk -> accumulate -> transcribe -> LLM correct -> type/SRT/overlay
///
/// The audio loop is non-blocking — it only captures audio, runs VAD, and pushes
/// segments to a bounded channel. All transcription happens on the worker thread.
///
/// State machine:
///   - on_key_down(V): Idle->Listening (start), Listening->Idle (toggle off)
///   - on_key_up(V):   if held >300ms -> stop (hold mode); else keep listening (toggle mode)
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::audio_capture::AudioCapture;
use crate::key_code::KeyCode;
use crate::local_sherpa::LocalSherpa;
use crate::local_whisper::LocalWhisper;
use crate::platform::Platform;

// ── STT segment types for the dual-track worker ───────────────────────────────

/// Which audio source produced a segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SttSource {
    Mic,
    Sys,
}

/// Commands sent from the audio loop to the STT worker.
enum SttCommand {
    /// New audio chunk while in speech — worker accumulates and transcribes.
    AudioChunk { source: SttSource, samples: Vec<f32>, timestamp_secs: f64 },
    /// Speech ended for the given source — worker finalizes and commits.
    SpeechEnd { source: SttSource, timestamp_secs: f64 },
    /// Flush pending buffers (input mode starts fresh).
    Flush,
    /// Session is ending — worker should clean up and exit.
    Quit,
    /// Final polish: re-transcribe full session audio with best LLM, replace typed text.
    FinalPolish { full_audio: Vec<f32> },
}

/// Unified speech-to-text engine — sherpa (SenseVoice) or whisper.
enum SttEngine {
    Sherpa(LocalSherpa),
    Whisper(LocalWhisper),
}

impl SttEngine {
    /// Create an STT engine based on preference. Falls back to the other engine.
    fn new_with_preference(prefer: &str) -> Option<Self> {
        let (first, second): (fn() -> Result<SttEngine, String>, fn() -> Result<SttEngine, String>) =
            if prefer == "whisper" {
                (
                    || LocalWhisper::new().map(SttEngine::Whisper).map_err(|e| e.to_string()),
                    || LocalSherpa::new().map(SttEngine::Sherpa).map_err(|e| e.to_string()),
                )
            } else {
                (
                    || LocalSherpa::new().map(SttEngine::Sherpa).map_err(|e| e.to_string()),
                    || LocalWhisper::new().map(SttEngine::Whisper).map_err(|e| e.to_string()),
                )
            };

        match first() {
            Ok(engine) => {
                eprintln!("[CLX] voice: {} ready", engine.engine_name());
                return Some(engine);
            }
            Err(e) => eprintln!("[CLX] voice: primary STT unavailable ({e}), trying fallback..."),
        }
        match second() {
            Ok(engine) => {
                eprintln!("[CLX] voice: {} ready (fallback)", engine.engine_name());
                Some(engine)
            }
            Err(e) => {
                eprintln!("[CLX] voice: all STT engines unavailable ({e})");
                None
            }
        }
    }

    fn engine_name(&self) -> &str {
        match self {
            SttEngine::Sherpa(_) => "SenseVoice (sherpa)",
            SttEngine::Whisper(_) => "Whisper",
        }
    }

    fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        match self {
            SttEngine::Sherpa(s) => s.transcribe(samples),
            SttEngine::Whisper(w) => w.transcribe(samples),
        }
    }

    /// Transcribe and return music detection flag alongside text.
    fn transcribe_tagged(&mut self, samples: &[f32]) -> Result<crate::local_sherpa::SttOutput, String> {
        match self {
            SttEngine::Sherpa(s) => s.transcribe_tagged(samples),
            // Whisper doesn't output music tags — just wrap normal result.
            SttEngine::Whisper(w) => w.transcribe(samples).map(|text| crate::local_sherpa::SttOutput { text, is_music: false }),
        }
    }

    fn check_pending_upgrade(&mut self) {
        if let SttEngine::Whisper(w) = self {
            let mut swap: Option<LocalWhisper> = None;
            w.check_pending_upgrade(&mut swap);
            if let Some(new_model) = swap {
                *self = SttEngine::Whisper(new_model);
            }
        }
    }

}

// SttEngine wraps sherpa-rs (ONNX Runtime) and whisper-rs (whisper.cpp),
// both of which are internally thread-safe C/C++ libraries. The Rust wrappers
// may not declare Send because they use raw pointers internally, but
// the underlying runtimes are safe to move between threads.
// SAFETY: SenseVoiceRecognizer and WhisperContext use opaque C handles that
// are thread-safe; we only access them from one thread at a time.
unsafe impl Send for SttEngine {}

#[allow(dead_code)]
const STATE_IDLE: u8 = 0;
#[allow(dead_code)]
const STATE_LISTENING: u8 = 1;
#[allow(dead_code)]
const STATE_STOPPING: u8 = 2;

/// Two independent voice features that can run simultaneously:
/// - Voice Note (click toggle): background recording to file + SRT + overlay
/// - Voice Input (hold): type transcription at cursor, stops on release

/// Hold threshold: if V is held longer than this, releasing V stops listening.
const HOLD_THRESHOLD_MS: u128 = 300;


/// Extract a string value for a given key from a JSON object.
fn extract_json_text_for_key(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = json.find(&needle)?;
    let after_key = &json[idx + needle.len()..];
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_colon = after_colon.trim_start();
    if !after_colon.starts_with('"') { return None; }
    let mut result = String::new();
    let mut chars = after_colon[1..].chars();
    loop {
        match chars.next()? {
            '"' => break,
            '\\' => { result.push(chars.next()?); }
            ch => result.push(ch),
        }
    }
    Some(result)
}

// ── VAD constants ─────────────────────────────────────────────────────────────

// TEN VAD constants are defined near VadState below.
/// Speech probability threshold to start recording.
/// Speech probability threshold — raised to avoid triggering on
/// YouTube/background audio from laptop speakers.
const SPEECH_START_PROB: f32 = 0.8;
/// Speech probability threshold below which silence is counted.
const SPEECH_END_PROB: f32 = 0.6;
/// Consecutive speech frames to trigger speech start (more = less sensitive).
const SPEECH_START_FRAMES: usize = 10;  // 160ms — filters AEC echo residue
/// Consecutive silence frames to end speech.
const SILENCE_END_FRAMES: usize = 20;  // 320ms
/// Streaming interval: transcribe after this many new samples accumulate.
/// Whisper inference is ~constant time regardless of audio length (~450ms
/// for base model), so the real cadence is limited by inference speed,
/// not this threshold. Setting it low (1600 = 100ms) means "transcribe
/// as fast as the model can keep up" — back-to-back inference during speech.
const STREAMING_CHUNK_SAMPLES: usize = 1_600; // 100ms at 16kHz
/// Maximum chunk duration in samples at 16kHz (25 seconds).
#[allow(dead_code)]
const VAD_MAX_CHUNK_SAMPLES: usize = 400_000;

pub struct VoiceModule {
    press_time: Mutex<Option<Instant>>,
    /// Atomic flag: true while V key is physically held down.
    v_held: Arc<AtomicBool>,
    platform: Arc<dyn Platform>,
    /// Voice Note mode: recording to file + SRT (toggle with click).
    note_active: Arc<AtomicBool>,
    /// Voice Input mode: typing at cursor (hold to activate).
    input_active: Arc<AtomicBool>,
    /// Signal to stop audio capture entirely.
    bg_stop: Arc<AtomicBool>,
    /// Signal to fully terminate the background thread.
    bg_quit: Arc<AtomicBool>,
    /// Handle to the background thread.
    bg_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Signal to wake the bg thread when any mode starts.
    bg_wake: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    /// Whether to capture system audio (Shift+V).
    with_system_audio: Arc<AtomicBool>,
    /// Signal to flush pending buffer (new input session starts).
    flush_pending: Arc<AtomicBool>,
    /// Signal: V released after hold — audio loop should send FinalPolish before Quit.
    final_polish_requested: Arc<AtomicBool>,
    /// Live config — behind Mutex so prefs changes take effect without restart.
    live_config: Arc<std::sync::Mutex<VoiceLiveConfig>>,
    /// Otoji subprocess backend — used instead of in-process SenseVoice when available.
    otoji: Arc<super::voice_otoji::OtojiBackend>,
    /// Text typed by otoji backend (for backspace replacement on final).
    otoji_typed: Arc<Mutex<String>>,
    /// Cached result of OtojiBackend::is_available() — checked once at startup.
    otoji_available: bool,
    /// Push-to-talk: hold V to record, release to transcribe + type.
    ptt: Arc<super::voice_ptt::PttSession>,
}

/// Config that can be hot-reloaded from preferences.
#[derive(Clone)]
struct VoiceLiveConfig {
    stt_engine: String,
    llm_api_key: String,
    llm_model: String,
    stt_correction: bool,
    tts_chain: String,
    stt_polish_chain: String,
    // Advanced voice/AEC thresholds
    aec_gain: f32,
    noise_gate: f32,
    speech_start_prob: f32,
    speech_end_prob: f32,
    speech_start_frames: usize,
    silence_end_frames: usize,
}

impl VoiceModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self::with_stt_engine(platform, "sherpa".to_string())
    }

    pub fn with_stt_engine(platform: Arc<dyn Platform>, stt_engine: String) -> Self {
        let otoji_backend = Arc::new(super::voice_otoji::OtojiBackend::new());
        let ptt = super::voice_ptt::PttSession::new(Arc::clone(&platform), Arc::clone(&otoji_backend));
        // Note: otoji_backend is shared between ptt and self.otoji below.
        Self {
            press_time: Mutex::new(None),
            v_held: Arc::new(AtomicBool::new(false)),
            platform,
            note_active: Arc::new(AtomicBool::new(false)),
            input_active: Arc::new(AtomicBool::new(false)),
            bg_stop: Arc::new(AtomicBool::new(false)),
            bg_quit: Arc::new(AtomicBool::new(false)),
            bg_thread: Mutex::new(None),
            bg_wake: Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new())),
            with_system_audio: Arc::new(AtomicBool::new(false)),
            flush_pending: Arc::new(AtomicBool::new(false)),
            final_polish_requested: Arc::new(AtomicBool::new(false)),
            otoji: otoji_backend,
            otoji_typed: Arc::new(Mutex::new(String::new())),
            otoji_available: super::voice_otoji::OtojiBackend::is_available(),
            ptt,
            live_config: Arc::new(std::sync::Mutex::new(VoiceLiveConfig {
                stt_engine,
                llm_api_key: String::new(),
                llm_model: String::new(),
                stt_correction: false,
                tts_chain: "elevenlabs:rachel,gemini-2.5-flash-preview-tts,openai:tts-1,msedge,native".to_string(),
                stt_polish_chain: "mlx:qwen2.5-3b,llm-corrector,raw".to_string(),
                aec_gain: 15.0,
                noise_gate: 0.003,
                speech_start_prob: 0.8,
                speech_end_prob: 0.6,
                speech_start_frames: 10,
                silence_end_frames: 20,
            })),
        }
    }

    pub fn with_llm_config(self, api_key: String, model: String, correction: bool) -> Self {
        {
            let mut cfg = self.live_config.lock().unwrap();
            cfg.llm_api_key = api_key;
            cfg.llm_model = model;
            cfg.stt_correction = correction;
        }
        self
    }

    /// Hot-reload config from preferences (takes effect on next voice session).
    pub fn update_config(
        &self, stt_engine: String, api_key: String, model: String, correction: bool,
        tts_chain: String, stt_polish_chain: String,
        aec_gain: f32, noise_gate: f32, speech_start_prob: f32, speech_end_prob: f32,
        speech_start_frames: usize, silence_end_frames: usize,
    ) {
        let mut cfg = self.live_config.lock().unwrap();
        cfg.stt_engine = stt_engine;
        cfg.llm_api_key = api_key;
        cfg.llm_model = model;
        cfg.stt_correction = correction;
        cfg.tts_chain = tts_chain;
        cfg.stt_polish_chain = stt_polish_chain;
        cfg.aec_gain = aec_gain;
        cfg.noise_gate = noise_gate;
        cfg.speech_start_prob = speech_start_prob;
        cfg.speech_end_prob = speech_end_prob;
        cfg.speech_start_frames = speech_start_frames;
        cfg.silence_end_frames = silence_end_frames;
        eprintln!("[CLX] voice: config hot-reloaded (engine={}, correction={}, aec_gain={}, noise_gate={})",
            cfg.stt_engine, cfg.stt_correction, cfg.aec_gain, cfg.noise_gate);
    }

    /// Check if voice-standalone is running via PID file.
    /// Uses kill(pid, 0) syscall instead of spawning a subprocess — safe for
    /// the CGEventTap callback which must return in <500ms.
    #[cfg(unix)]
    fn voice_standalone_pid() -> Option<u32> {
        let pid_str = std::fs::read_to_string("/tmp/clx-voice-standalone.pid").ok()?;
        let pid = pid_str.trim().parse::<u32>().ok()?;
        extern "C" { fn kill(pid: i32, sig: i32) -> i32; }
        let alive = unsafe { kill(pid as i32, 0) } == 0;
        if alive { Some(pid) } else { None }
    }

    #[cfg(not(unix))]
    fn voice_standalone_pid() -> Option<u32> { None }

    /// Send a Unix signal to a process (best-effort, no subprocess spawn).
    #[cfg(unix)]
    fn send_signal(pid: u32, sig: i32) {
        extern "C" { fn kill(pid: i32, sig: i32) -> i32; }
        unsafe { kill(pid as i32, sig); }
    }

    #[cfg(not(unix))]
    fn send_signal(_pid: u32, _sig: i32) {}

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        // Delegate to voice-standalone if it's running (avoids duplicate overlay).
        if let Some(pid) = Self::voice_standalone_pid() {
            eprintln!("[CLX] voice: delegating V key_down to voice-standalone (pid={})", pid);
            self.platform.hide_voice_overlay(); // hide CLX's own overlay
            self.stop_pipeline(); // stop CLX's pipeline if running
            *self.press_time.lock().unwrap() = Some(Instant::now());
            Self::send_signal(pid, 10); // SIGUSR1
            self.v_held.store(true, Ordering::Relaxed);
            return true;
        }

        eprintln!("[CLX] voice: V key pressed, activating...");

        *self.press_time.lock().unwrap() = Some(Instant::now());
        self.v_held.store(true, Ordering::Relaxed);

        // Push-to-talk: start/extend the PTT segment in parallel to the
        // always-on pipeline. On release, PTT transcribes the held segment
        // with SenseVoice and types it at the cursor.
        // If PTT was in locked mode, this press exits it — skip normal flow.
        if self.ptt.on_press() {
            return true;
        }

        // Always enable dual capture (mic+system) for both tracks.
        // Shift+V is no longer needed — system audio is always captured.
        self.with_system_audio.store(true, Ordering::Relaxed);

        // Keep the always-on pipeline running (overlay + note mode). It does
        // NOT type into the cursor — `input_active` is left false so otoji /
        // in-process STT only update the overlay. PTT owns typing.
        self.ensure_pipeline_running();
        true
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        // Delegate to voice-standalone if it's running.
        if let Some(pid) = Self::voice_standalone_pid() {
            eprintln!("[CLX] voice: delegating V key_up to voice-standalone (pid={})", pid);
            self.v_held.store(false, Ordering::Relaxed);
            Self::send_signal(pid, 12); // SIGUSR2
            *self.press_time.lock().unwrap() = None;
            return true;
        }

        self.v_held.store(false, Ordering::Relaxed);

        use super::voice_ptt::PttRelease;
        let ptt_result = self.ptt.on_release();

        match ptt_result {
            PttRelease::Hold => {
                eprintln!("[CLX] voice: hold release → waiting for otoji ptt_final");
                // Only hide the overlay if note mode isn't keeping it visible.
                if !self.note_active.load(Ordering::Relaxed) {
                    self.platform.hide_voice_overlay();
                }
            }
            PttRelease::Locked => {
                eprintln!("[CLX] voice: double-tap → PTT locked mode");
            }
            PttRelease::Tap => {
                // Single tap: toggle voice NOTE mode (live subtitles in overlay).
                use super::super::platform::PttTrayState;
                if self.note_active.load(Ordering::Relaxed) {
                    self.note_active.store(false, Ordering::Relaxed);
                    eprintln!("[CLX] voice: click → note stopped");
                    self.platform.hide_voice_overlay();
                    self.platform.set_ptt_tray_state(PttTrayState::Idle);
                } else {
                    self.note_active.store(true, Ordering::Relaxed);
                    eprintln!("[CLX] voice: click → note started");
                    self.platform.show_voice_overlay();
                    self.platform.set_ptt_tray_state(PttTrayState::NoteMode);
                }
            }
        }

        *self.press_time.lock().unwrap() = None;
        true
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    /// Called when CLX mode deactivates (Space released).
    /// If voice input was active, trigger final polish before stopping.
    pub fn stop(&self) {
        if self.input_active.load(Ordering::Relaxed) {
            // Input was active — request final polish (same as hold-release path).
            // Keep input_active=true so the worker can type the polished result.
            self.final_polish_requested.store(true, Ordering::Relaxed);
            eprintln!("[CLX] voice: CLX deactivated while input active → final polish requested");
            if !self.note_active.load(Ordering::Relaxed) {
                self.stop_pipeline();
            }
        }
    }

    pub fn is_listening(&self) -> bool {
        self.note_active.load(Ordering::Relaxed) || self.input_active.load(Ordering::Relaxed)
    }

    // ── Internal: audio lifecycle ─────────────────────────────────────────────

    /// Start the pipeline immediately (always-on mode).
    /// Used by standalone voice binaries that don't have a key trigger.
    pub fn start_always_on(&self, with_sys: bool) {
        self.with_system_audio.store(with_sys, Ordering::Relaxed);
        self.note_active.store(true, Ordering::Relaxed);
        // Bypass standalone check — we ARE the standalone.
        self.ensure_pipeline_running_force();
    }

    /// Spawn the background thread eagerly so the Whisper model loads at startup.
    pub fn preload(&self) {
        // Skip in-process STT preload when otoji is available — it provides
        // SenseVoice as an external subprocess, saving ~1 GB in this process.
        if super::voice_otoji::OtojiBackend::is_available() {
            eprintln!("[CLX] voice: otoji available, skipping in-process STT preload");
            return;
        }
        let mut bg = self.bg_thread.lock().unwrap();
        if bg.is_none() {
            self.spawn_bg_thread(&mut bg);
        }
    }

    fn spawn_bg_thread(&self, bg: &mut Option<std::thread::JoinHandle<()>>) {
        let bg_stop = Arc::clone(&self.bg_stop);
        let bg_quit = Arc::clone(&self.bg_quit);
        let bg_wake = Arc::clone(&self.bg_wake);
        let with_sys = Arc::clone(&self.with_system_audio);
        let note_active = Arc::clone(&self.note_active);
        let input_active = Arc::clone(&self.input_active);
        let flush_pending = Arc::clone(&self.flush_pending);
        let final_polish_requested = Arc::clone(&self.final_polish_requested);
        let platform = Arc::clone(&self.platform);

        let live_config = Arc::clone(&self.live_config);
        let cfg_snap = live_config.lock().unwrap().clone();

        let handle = std::thread::Builder::new()
            .name("clx-voice-bg".into())
            .spawn(move || {
                voice_bg_persistent(bg_stop, bg_quit, bg_wake, with_sys, note_active, input_active, flush_pending, final_polish_requested, platform, &cfg_snap.stt_engine, &cfg_snap.llm_api_key, &cfg_snap.llm_model, cfg_snap.stt_correction, live_config);
            })
            .expect("failed to spawn voice bg thread");

        *bg = Some(handle);
    }

    /// Ensure the audio pipeline bg thread is running and capturing.
    ///
    /// IMPORTANT: This runs inside the CGEventTap callback — must NOT block
    /// for >200ms or macOS disables the event tap and CLX dies. All subprocess
    /// spawns (otoji, pgrep) happen on a background thread.
    fn ensure_pipeline_running(&self) {
        // Prefer otoji as external STT backend (cached check — no subprocess).
        if self.otoji_available {
            if !self.otoji.is_running() {
                eprintln!("[CLX] voice: launching otoji backend (bg thread)");
                self.platform.show_voice_overlay();
                self.platform.update_voice_subtitle("otoji starting...");
                *self.otoji_typed.lock().unwrap() = String::new();
                // Spawn otoji on a background thread — Command::spawn() can
                // take 100ms+ and would timeout the CGEventTap callback.
                let otoji = Arc::clone(&self.otoji);
                let platform = Arc::clone(&self.platform);
                let input_active = Arc::clone(&self.input_active);
                let otoji_typed = Arc::clone(&self.otoji_typed);
                let ptt = Arc::clone(&self.ptt);
                std::thread::Builder::new()
                    .name("otoji-launch".into())
                    .spawn(move || {
                        if !otoji.start(platform.clone(), input_active, otoji_typed, Some(ptt)) {
                            eprintln!("[CLX] voice: otoji failed to start");
                            platform.update_voice_subtitle("otoji failed");
                        }
                    })
                    .ok();
            }
            return;
        }

        self.ensure_pipeline_running_force();
    }

    fn ensure_pipeline_running_force(&self) {
        eprintln!("[CLX] voice: ensuring pipeline running");

        // Spawn the bg thread once — it stays alive and waits between sessions.
        let mut bg = self.bg_thread.lock().unwrap();
        if bg.is_none() {
            self.spawn_bg_thread(&mut bg);
        }

        // Signal the bg thread to start recording.
        self.bg_stop.store(false, Ordering::Relaxed);
        let (lock, cvar) = &*self.bg_wake;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
        drop(started);
    }

    fn stop_pipeline(&self) {
        // Stop otoji backend if running.
        if self.otoji.is_running() {
            self.otoji.stop();
            self.platform.hide_voice_overlay();
        }
        if !self.bg_stop.load(Ordering::Relaxed) {
            eprintln!("[CLX] voice: stopping pipeline");
            self.bg_stop.store(true, Ordering::Relaxed);
        }
    }
}

// ── Persistent background thread ──────────────────────────────────────────────

/// Background thread that loads the Whisper model once, then loops:
/// wait for wake signal → spawn STT worker → audio capture + VAD → push segments → wait again.
fn voice_bg_persistent(
    bg_stop: Arc<AtomicBool>,
    bg_quit: Arc<AtomicBool>,
    bg_wake: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    with_system_audio: Arc<AtomicBool>,
    note_active: Arc<AtomicBool>,
    input_active: Arc<AtomicBool>,
    flush_pending: Arc<AtomicBool>,
    final_polish_requested: Arc<AtomicBool>,
    platform: Arc<dyn Platform>,
    stt_engine_pref: &str,
    llm_api_key: &str,
    llm_model: &str,
    stt_correction: bool,
    live_config: Arc<std::sync::Mutex<VoiceLiveConfig>>,
) {
    // Load STT engine based on preference.
    let mut local_whisper = SttEngine::new_with_preference(stt_engine_pref);

    // Create LLM-based STT corrector if configured.
    let mut corrector = if stt_correction && !llm_api_key.is_empty() {
        let config = crate::llm_client::LlmConfig::from_key_and_model(llm_api_key, llm_model);
        eprintln!("[CLX] voice: STT correction enabled ({:?} / {})", config.provider, config.model);
        Some(crate::stt_corrector::SttCorrector::new(config))
    } else {
        None
    };

    eprintln!("[CLX] voice: bg thread ready, waiting for activation");

    loop {
        // Wait for wake signal.
        {
            let (lock, cvar) = &*bg_wake;
            let mut started = lock.lock().unwrap();
            while !*started && !bg_quit.load(Ordering::Relaxed) {
                started = cvar.wait_timeout(started, std::time::Duration::from_secs(1)).unwrap().0;
            }
            if bg_quit.load(Ordering::Relaxed) { break; }
            *started = false;
        }

        // Create AudioCapture fresh each session (cpal::Stream is !Send).
        let t_wake = std::time::Instant::now();
        // Use VoiceProcessingIO AEC only when system audio is captured (Shift+V).
        // Ducking is minimized so speakers stay audible.
        let aec_mic: Option<Box<dyn crate::platform::SystemAudioStream>> = if with_system_audio.load(Ordering::Relaxed) {
            platform.start_aec_mic()
        } else {
            None
        };
        let use_aec = aec_mic.is_some();

        // Fall back to cpal AudioCapture if AEC not available.
        let ac = if aec_mic.is_none() {
            match AudioCapture::new() {
                Ok(ac) => { let _ = ac.start(); Some(ac) }
                Err(e) => { eprintln!("[CLX] voice: audio capture failed: {e}"); continue; }
            }
        } else { None };
        let sample_rate = if let Some(ref aec) = aec_mic { aec.sample_rate() } else { ac.as_ref().unwrap().sample_rate() };
        let vcfg = live_config.lock().unwrap().clone();
        let mut vad = VadState::with_thresholds(
            vcfg.speech_start_prob, vcfg.speech_end_prob,
            vcfg.speech_start_frames, vcfg.silence_end_frames,
        );

        // Start system audio capture if Shift+V was pressed.
        let has_sys_capture = with_system_audio.load(Ordering::Relaxed);
        let sys_capture = if has_sys_capture {
            match platform.start_system_audio() {
                Some(s) => {
                    eprintln!("[CLX] voice: system audio capture started");
                    Some(s)
                }
                None => {
                    eprintln!("[CLX] voice: system audio not available on this platform");
                    None
                }
            }
        } else {
            None
        };

        eprintln!("[CLX] voice: recording started ({:.0}ms startup, sr={sample_rate}{})",
            t_wake.elapsed().as_millis(),
            if sys_capture.is_some() { " +sysaudio" } else { "" });

        platform.show_voice_overlay();

        // Remind user to enable Voice Isolation (once per app session).
        {
            static REMINDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !REMINDED.swap(true, Ordering::Relaxed) {
                platform.update_voice_subtitle("🎤 Tip: Enable Voice Isolation in Control Center for best STT");
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }

        // ── Spawn STT worker thread for this session ──────────────────────
        // Bounded channel: 32 slots = ~1.6s of audio chunks at 50ms per chunk.
        // If the worker falls behind (transcription takes ~450ms), there's enough
        // buffer to avoid dropping audio chunks.
        let (stt_tx, stt_rx) = std::sync::mpsc::sync_channel::<SttCommand>(32);

        let worker_platform = Arc::clone(&platform);
        let worker_note_active = Arc::clone(&note_active);
        let worker_input_active = Arc::clone(&input_active);
        let worker_has_sys = sys_capture.is_some();

        // Share STT engine and corrector with the worker via Arc<Mutex>.
        // Only one thread accesses them at a time (worker during session,
        // bg thread only after worker exits).
        let shared_engine: Arc<Mutex<Option<SttEngine>>> = Arc::new(Mutex::new(local_whisper.take()));
        let shared_corrector: Arc<Mutex<Option<crate::stt_corrector::SttCorrector>>> = Arc::new(Mutex::new(corrector.take()));
        let worker_engine = Arc::clone(&shared_engine);
        let worker_corrector = Arc::clone(&shared_corrector);
        let worker_live_config = Arc::clone(&live_config);

        let stt_worker = std::thread::Builder::new()
            .name("clx-stt-worker".into())
            .spawn(move || {
                stt_worker_loop(
                    stt_rx,
                    &worker_engine,
                    &worker_corrector,
                    &worker_platform,
                    &worker_note_active,
                    &worker_input_active,
                    worker_has_sys,
                    &worker_live_config,
                )
            })
            .expect("failed to spawn STT worker thread");

        // WebRTC AEC3 echo canceller (replaces NLMS).
        // 16kHz mono, 10ms frames = 160 samples.
        let mut aec3 = aec3::voip::VoipAec3::builder(16_000, 1, 1)
            .initial_delay_ms(120)
            .enable_high_pass(true)
            .enable_noise_suppression(true)
            .build()
            .ok();
        if aec3.is_some() {
            eprintln!("[CLX] voice: AEC3 echo canceller initialized (16kHz mono)");
        } else {
            eprintln!("[CLX] voice: AEC3 initialization failed, falling back to no AEC");
        }
        let aec3_frame_size: usize = 160; // 10ms at 16kHz
        let mut aec3_mic_remainder: Vec<f32> = Vec::new();
        let mut aec3_ref_remainder: Vec<f32> = Vec::new();

        // ── System audio VAD (audio loop only runs VAD, worker handles transcription) ──
        let mut sys_vad = if sys_capture.is_some() {
            Some(VadState::with_thresholds(
                vcfg.speech_start_prob, vcfg.speech_end_prob,
                vcfg.speech_start_frames, vcfg.silence_end_frames,
            ))
        } else { None };

        // Pre-send accumulators: batch audio before sending to the STT worker.
        // Mic: 200ms batches for low latency.  Sys: 500ms batches — background audio
        // doesn't need to be as responsive, and this keeps mic getting priority.
        const MIC_SEND_THRESHOLD: usize = 1_600;  // 100ms at 16kHz
        const SYS_SEND_THRESHOLD: usize = 8_000;  // 500ms at 16kHz
        let mut mic_send_buf: Vec<f32> = Vec::new();
        let mut sys_send_buf: Vec<f32> = Vec::new();

        let note_start_time = std::time::Instant::now();

        // Accumulate all 16kHz mic audio for final polish on V release.
        let mut full_session_mic: Vec<f32> = Vec::new();

        // ffmpeg pipe for streaming WebM — lazily started when note_active becomes true.
        let mut ffmpeg_stdin: Option<std::process::ChildStdin> = None;
        let mut ffmpeg_child: Option<std::process::Child> = None;
        let note_dir = dirs::home_dir().map(|h| h.join(".capslockx").join("voice"));
        if let Some(ref dir) = note_dir {
            std::fs::create_dir_all(dir).ok();
        }
        let session_ts = chrono_timestamp();

        // Session log.
        let _ = std::fs::write("/tmp/clx-voice.log", "");

        // Record loop until bg_stop is set.
        let mut standalone_check_counter: u32 = 0;
        while !bg_stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Periodically check if voice-standalone started — yield to it.
            standalone_check_counter += 1;
            if standalone_check_counter % 100 == 0 { // every ~5 seconds
                if VoiceModule::voice_standalone_pid().is_some() {
                    eprintln!("[CLX] voice: voice-standalone detected, stopping internal pipeline");
                    platform.hide_voice_overlay();
                    bg_stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
            let samples = if let Some(ref aec) = aec_mic { aec.take_samples() } else { ac.as_ref().unwrap().take_samples() };
            if samples.is_empty() {
                platform.update_voice_overlay(&[], vad.in_speech, &[], false);
                continue;
            }

            // Get system audio samples separately (don't mix).
            let sys_samples = if let Some(ref sys) = sys_capture {
                sys.take_samples()
            } else {
                Vec::new()
            };

            // Check flush signal: discard ALL buffered audio when input mode starts fresh.
            if flush_pending.swap(false, Ordering::Relaxed) {
                // Drain the audio capture buffer so old samples don't leak in.
                if let Some(ref aec) = aec_mic { let _ = aec.take_samples(); }
                if let Some(ref a) = ac { let _ = a.take_samples(); }
                // Reset VAD state so it doesn't think we're mid-speech.
                let vcfg_flush = live_config.lock().unwrap().clone();
                vad = VadState::with_thresholds(
                    vcfg_flush.speech_start_prob, vcfg_flush.speech_end_prob,
                    vcfg_flush.speech_start_frames, vcfg_flush.silence_end_frames,
                );
                // Tell the worker to flush its transcription state and buffers.
                let _ = stt_tx.try_send(SttCommand::Flush);
                // Clear session audio so final polish starts fresh.
                full_session_mic.clear();
                eprintln!("[CLX] voice: flushed audio buffers for fresh input session");
            }

            // Resample both streams to 16kHz.
            let mic_16k_raw = resample(&samples, sample_rate, 16000);
            let sys_16k = if !sys_samples.is_empty() {
                resample(&sys_samples, 48000, 16000)
            } else {
                Vec::new()
            };

            // AEC3 echo cancellation on raw (pre-gain) signal, processed in 10ms frames.
            let mic_cancelled = if !sys_16k.is_empty() && aec3.is_some() {
                let aec = aec3.as_mut().unwrap();
                aec3_mic_remainder.extend_from_slice(&mic_16k_raw);
                aec3_ref_remainder.extend_from_slice(&sys_16k);

                let mut output = Vec::with_capacity(aec3_mic_remainder.len());
                while aec3_mic_remainder.len() >= aec3_frame_size
                    && aec3_ref_remainder.len() >= aec3_frame_size
                {
                    let ref_frame: Vec<f32> = aec3_ref_remainder.drain(..aec3_frame_size).collect();
                    let mic_frame: Vec<f32> = aec3_mic_remainder.drain(..aec3_frame_size).collect();
                    let _ = aec.handle_render_frame(&ref_frame);
                    let mut clean = vec![0.0f32; aec3_frame_size];
                    let _ = aec.process_capture_frame(&mic_frame, false, &mut clean);
                    output.extend_from_slice(&clean);
                }
                output
            } else {
                mic_16k_raw
            };

            // Log mic RMS before gain for AEC debug.
            let pre_gain_rms = if mic_cancelled.is_empty() { 0.0 } else {
                (mic_cancelled.iter().map(|x| x*x).sum::<f32>() / mic_cancelled.len() as f32).sqrt()
            };

            // Apply gain + noise gate AFTER AEC.
            // AEC3 includes AGC2 + noise suppression, so gain should be moderate.
            // Too high → echo residue fools VAD; too low → real speech too quiet.
            let mic_16k: Vec<f32> = if use_aec {
                let cfg_gain = vcfg.aec_gain;
                let cfg_gate = vcfg.noise_gate;
                mic_cancelled.iter()
                    .map(|&s| if s.abs() < cfg_gate { 0.0 } else { (s * cfg_gain).clamp(-1.0, 1.0) })
                    .collect()
            } else {
                mic_cancelled
            };

            let samples_16k = &mic_16k;

            // Log mic RMS for AEC debug.
            {
                static MIC_RMS_LOG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let n = MIC_RMS_LOG.fetch_add(1, Ordering::Relaxed);
                if n < 50 || n % 50 == 0 {
                    let rms = if mic_16k.is_empty() { 0.0 } else {
                        (mic_16k.iter().map(|x| x*x).sum::<f32>() / mic_16k.len() as f32).sqrt()
                    };
                    eprintln!("[CLX] voice: mic_rms raw={:.6} after_gain={:.4} (n={})", pre_gain_rms, rms, n);
                }
            }

            // Accumulate session audio for final polish (only during input mode).
            if input_active.load(Ordering::Relaxed) {
                full_session_mic.extend_from_slice(samples_16k);
            }

            // Lazily start ffmpeg — stereo if system audio present, mono otherwise.
            if note_active.load(Ordering::Relaxed) && ffmpeg_stdin.is_none() && ffmpeg_child.is_none() {
                if let Some(ref dir) = note_dir {
                    let webm_path = dir.join(format!("{}.webm", session_ts));
                    let channels = if sys_capture.is_some() { "2" } else { "1" };
                    let bitrate = if sys_capture.is_some() { "64k" } else { "48k" };
                    match std::process::Command::new("ffmpeg")
                        .args([
                            "-y", "-f", "s16le", "-ar", "16000", "-ac", channels,
                            "-i", "pipe:0", "-c:a", "libopus", "-b:a", bitrate,
                            "-f", "webm", webm_path.to_str().unwrap_or("out.webm"),
                        ])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(mut child) => {
                            ffmpeg_stdin = child.stdin.take();
                            ffmpeg_child = Some(child);
                            eprintln!("[CLX] voice: streaming WebM to {} ({}ch)", webm_path.display(), channels);
                        }
                        Err(e) => eprintln!("[CLX] voice: ffmpeg spawn failed: {e}"),
                    }
                }
            }

            // Pipe PCM to ffmpeg — stereo interleaved (mic=L, sys=R) or mono.
            if let Some(ref mut stdin) = ffmpeg_stdin {
                use std::io::Write;
                let pcm_bytes: Vec<u8> = if sys_capture.is_some() {
                    // Stereo: interleave mic (left) and system (right).
                    let max_len = mic_16k.len().max(sys_16k.len());
                    (0..max_len).flat_map(|i| {
                        let m = (mic_16k.get(i).copied().unwrap_or(0.0).clamp(-1.0, 1.0) * 32767.0) as i16;
                        let s = (sys_16k.get(i).copied().unwrap_or(0.0).clamp(-1.0, 1.0) * 32767.0) as i16;
                        let mut bytes = [0u8; 4];
                        bytes[0..2].copy_from_slice(&m.to_le_bytes());
                        bytes[2..4].copy_from_slice(&s.to_le_bytes());
                        bytes
                    }).collect()
                } else {
                    // Mono: mic only.
                    mic_16k.iter()
                        .flat_map(|&s| ((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())
                        .collect()
                };
                if stdin.write_all(&pcm_bytes).is_err() {
                    ffmpeg_stdin = None;
                    eprintln!("[CLX] voice: ffmpeg pipe broken");
                }
            }

            // Process system VAD FIRST so we know if system is playing
            // before deciding whether to feed mic to VAD.
            let sys_was_speech_before = sys_vad.as_ref().map(|v| v.in_speech).unwrap_or(false);
            if let Some(ref mut svad) = sys_vad {
                if !sys_16k.is_empty() {
                    svad.feed(&sys_16k);
                }
            }

            // Compute RMS levels for overlay (dual waveforms).
            let mic_rms: Vec<f32> = mic_16k.chunks(TEN_VAD_FRAME_SIZE.max(1))
                .map(|f| { let s: f32 = f.iter().map(|x| x*x).sum(); (s / f.len() as f32).sqrt() })
                .collect();
            let sys_rms: Vec<f32> = if !sys_16k.is_empty() {
                sys_16k.chunks(TEN_VAD_FRAME_SIZE.max(1))
                    .map(|f| { let s: f32 = f.iter().map(|x| x*x).sum(); (s / f.len() as f32).sqrt() })
                    .collect()
            } else {
                Vec::new()
            };
            let sys_vad_active = sys_vad.as_ref().map(|v| v.in_speech).unwrap_or(false);
            if !sys_rms.is_empty() && sys_rms.iter().any(|&v| v > 0.001) {
                static SYS_LOG_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let c = SYS_LOG_COUNT.fetch_add(1, Ordering::Relaxed);
                if c < 5 { eprintln!("[CLX] voice: sys_rms samples={} max={:.4}", sys_rms.len(), sys_rms.iter().cloned().fold(0.0f32, f32::max)); }
            }
            platform.update_voice_overlay(&mic_rms, vad.in_speech, &sys_rms, sys_vad_active);

            // ── Mic VAD + push audio chunks to worker ──────────────────────
            let was_in_speech = vad.in_speech;
            let _chunks = vad.feed(samples_16k);

            if vad.in_speech {
                // Accumulate locally; only send once SEND_THRESHOLD reached.
                mic_send_buf.extend_from_slice(samples_16k);
                static SPEECH_LOG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let n = SPEECH_LOG.fetch_add(1, Ordering::Relaxed);
                if n < 3 || n % 100 == 0 {
                    eprintln!("[CLX] voice: speech buf={} threshold={} samples_16k={}",
                        mic_send_buf.len(), MIC_SEND_THRESHOLD, samples_16k.len());
                }
                if mic_send_buf.len() >= MIC_SEND_THRESHOLD {
                    let cmd = SttCommand::AudioChunk {
                        source: SttSource::Mic,
                        samples: mic_send_buf.clone(),
                        timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                    };
                    if stt_tx.try_send(cmd).is_ok() {
                        mic_send_buf.clear();
                    } else {
                        // Channel full: drop oldest half, keep newest audio.
                        let keep = mic_send_buf.len() / 2;
                        mic_send_buf.drain(..mic_send_buf.len() - keep);
                    }
                }
            } else {
                if was_in_speech {
                    // Flush any remaining accumulated audio on speech end.
                    if !mic_send_buf.is_empty() {
                        let cmd = SttCommand::AudioChunk {
                            source: SttSource::Mic,
                            samples: mic_send_buf.clone(),
                            timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                        };
                        let _ = stt_tx.try_send(cmd);
                        mic_send_buf.clear();
                    }
                    // Speech ended — tell the worker to finalize.
                    let cmd = SttCommand::SpeechEnd {
                        source: SttSource::Mic,
                        timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                    };
                    if stt_tx.try_send(cmd).is_err() {
                        eprintln!("[CLX] voice: STT channel full, dropping mic speech-end");
                    }
                } else {
                    mic_send_buf.clear(); // discard stale samples when not in speech
                }
            }

            // ── System audio VAD + push audio chunks to worker ────────────
            if let Some(ref mut svad) = sys_vad {
                if !sys_16k.is_empty() {
                    let sys_was_speech = sys_was_speech_before;

                    if svad.in_speech {
                        // Accumulate locally; only send once SEND_THRESHOLD reached.
                        sys_send_buf.extend_from_slice(&sys_16k);
                        if sys_send_buf.len() >= SYS_SEND_THRESHOLD {
                            let cmd = SttCommand::AudioChunk {
                                source: SttSource::Sys,
                                samples: sys_send_buf.clone(),
                                timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                            };
                            if stt_tx.try_send(cmd).is_ok() {
                                sys_send_buf.clear();
                            } else {
                                // Channel full: drop oldest half.
                                let keep = sys_send_buf.len() / 2;
                                sys_send_buf.drain(..sys_send_buf.len() - keep);
                            }
                        }
                    } else if sys_was_speech {
                        // Flush remaining on speech end.
                        if !sys_send_buf.is_empty() {
                            let cmd = SttCommand::AudioChunk {
                                source: SttSource::Sys,
                                samples: sys_send_buf.clone(),
                                timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                            };
                            let _ = stt_tx.try_send(cmd);
                            sys_send_buf.clear();
                        }
                        // Sys speech ended.
                        let cmd = SttCommand::SpeechEnd {
                            source: SttSource::Sys,
                            timestamp_secs: note_start_time.elapsed().as_secs_f64(),
                        };
                        if stt_tx.try_send(cmd).is_err() {
                            eprintln!("[CLX] voice: STT channel full, dropping sys speech-end");
                        }
                    }
                }
            }
        }

        // ── Session ending: send FinalPolish if requested, then Quit ─────
        if final_polish_requested.swap(false, Ordering::Relaxed) && !full_session_mic.is_empty() {
            eprintln!("[CLX] voice: sending FinalPolish ({} samples = {:.1}s)",
                full_session_mic.len(), full_session_mic.len() as f64 / 16000.0);
            let _ = stt_tx.send(SttCommand::FinalPolish { full_audio: full_session_mic.clone() });
        }
        full_session_mic.clear();

        // Tell the worker to quit and wait for it.
        let _ = stt_tx.send(SttCommand::Quit);
        drop(stt_tx);

        // Wait for the worker to finish and get its results.
        let worker_result = stt_worker
            .join()
            .unwrap_or_else(|_| {
                eprintln!("[CLX] voice: STT worker panicked");
                SttWorkerResult::default()
            });
        // Recover engine and corrector from shared state.
        local_whisper = shared_engine.lock().unwrap().take();
        corrector = shared_corrector.lock().unwrap().take();

        // Now that FinalPolish is done, clear input_active.
        input_active.store(false, Ordering::Relaxed);

        platform.hide_voice_overlay();

        // Save SRT from worker result.
        let note_srt = worker_result.note_srt;

        // Close ffmpeg stdin -> ffmpeg finalizes the WebM file.
        drop(ffmpeg_stdin);
        if let Some(mut child) = ffmpeg_child.take() {
            std::thread::spawn(move || {
                match child.wait() {
                    Ok(s) if s.success() => eprintln!("[CLX] voice: WebM saved"),
                    Ok(s) => eprintln!("[CLX] voice: ffmpeg exit {s}"),
                    Err(e) => eprintln!("[CLX] voice: ffmpeg wait error: {e}"),
                }
            });
        }
        // Save SRT, then remux WebM + SRT -> WebM with embedded subtitles.
        if let Some(ref dir) = note_dir {
            let srt_path = dir.join(format!("{}.srt", session_ts));
            let webm_path = dir.join(format!("{}.webm", session_ts));
            let webm_sub_path = dir.join(format!("{}_sub.webm", session_ts));

            if !note_srt.is_empty() {
                let _ = std::fs::write(&srt_path, &note_srt);
                eprintln!("[CLX] voice: SRT saved");
            }

            // Remux: embed SRT as WebVTT subtitle track into WebM.
            if webm_path.exists() && srt_path.exists() {
                let srt = srt_path.clone();
                let webm = webm_path.clone();
                let webm_sub = webm_sub_path.clone();
                std::thread::spawn(move || {
                    let result = std::process::Command::new("ffmpeg")
                        .args([
                            "-y",
                            "-i", webm.to_str().unwrap_or(""),
                            "-i", srt.to_str().unwrap_or(""),
                            "-c:a", "copy",
                            "-c:s", "webvtt",
                            "-f", "webm",
                            webm_sub.to_str().unwrap_or(""),
                        ])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                    match result {
                        Ok(s) if s.success() => {
                            // Replace original with subtitle version.
                            let _ = std::fs::rename(&webm_sub, &webm);
                            eprintln!("[CLX] voice: WebM remuxed with embedded subtitles");
                        }
                        Ok(s) => eprintln!("[CLX] voice: ffmpeg remux failed (exit {s})"),
                        Err(e) => eprintln!("[CLX] voice: ffmpeg not found: {e}"),
                    }
                });
            }
        }

        if let Some(ref sys) = sys_capture { sys.stop(); }
        if let Some(ref aec) = aec_mic { aec.stop(); }
        if let Some(ref a) = ac { a.stop(); }
        eprintln!("[CLX] voice: session ended, waiting for next activation");
    }

    eprintln!("[CLX] voice: bg thread exiting");
}

// ── STT Worker Thread ─────────────────────────────────────────────────────────

/// Result returned by the STT worker when it exits.
#[derive(Default)]
struct SttWorkerResult {
    note_srt: String,
}

/// The STT worker loop: receives segments from the audio loop, transcribes,
/// applies corrections, types text, writes SRT, and updates overlay.
///
/// Mic segments are prioritized over sys segments when both are pending.
fn stt_worker_loop(
    rx: std::sync::mpsc::Receiver<SttCommand>,
    engine_lock: &Arc<Mutex<Option<SttEngine>>>,
    corrector_lock: &Arc<Mutex<Option<crate::stt_corrector::SttCorrector>>>,
    platform: &Arc<dyn Platform>,
    note_active: &Arc<AtomicBool>,
    input_active: &Arc<AtomicBool>,
    has_sys: bool,
    live_config: &Arc<std::sync::Mutex<VoiceLiveConfig>>,
) -> SttWorkerResult {
    // Take ownership of engine/corrector from the shared locks for the session.
    // This avoids holding the mutex during transcription.
    let mut engine = engine_lock.lock().unwrap().take();
    let mut corrector = corrector_lock.lock().unwrap().take();
    // Snapshot polish chain once at session start (avoids Mutex lock on every speech event).
    let polish_chain = live_config.lock().unwrap().stt_polish_chain.clone();
    // ── Mic track state ──
    let mut mic_pending_buf: Vec<f32> = Vec::new();
    let mut mic_pending_since: usize = 0;
    let mut mic_committed = String::new();
    let mut mic_whisper_pending = String::new();
    let mut mic_typed_pending = String::new();
    let mut mic_prev_whisper = String::new();
    let mut mic_stable: usize = 0;
    let mut mic_new_samples_since_commit: usize = 0; // reset on commit; prevents context re-transcription from triggering stability
    // ── Humming track state ──
    let mut hum_pending = String::new();
    // Tracks the total text typed into the input box during this input session.
    // Used by FinalPolish to know how many chars to backspace before typing polished text.
    let mut session_input_typed = String::new();

    // ── Speech speed detection ──
    let mut mic_syl_detector = SyllableRateDetector::new();
    let mut sys_syl_detector = SyllableRateDetector::new();

    // ── System audio track state ──
    let mut sys_pending_buf: Vec<f32> = Vec::new();
    let mut sys_pending_since: usize = 0;
    let mut sys_committed = String::new();
    let mut sys_whisper_pending = String::new();
    let mut sys_prev_whisper = String::new();
    let mut sys_stable: usize = 0;

    // ── SRT state ──
    let mut note_srt = String::new();
    let mut srt_index: usize = 0;

    // ── Unified subtitle state — rebuilt after any track update ──
    let mut subtitle_dirty = false;

    eprintln!("[CLX] stt-worker: started");

    loop {
        // Priority drain: collect all pending commands, process mic before sys.
        let cmd = match rx.recv() {
            Ok(cmd) => cmd,
            Err(_) => break, // channel closed
        };

        // Drain any additional pending commands for priority sorting.
        let mut mic_cmds: Vec<SttCommand> = Vec::new();
        let mut sys_cmds: Vec<SttCommand> = Vec::new();
        let mut flush = false;
        let mut quit = false;
        let mut final_polish_audio: Option<Vec<f32>> = None;

        // Classify and route all pending commands. Mic gets priority.
        let mut pending = vec![cmd];
        while let Ok(extra) = rx.try_recv() {
            pending.push(extra);
        }
        for cmd in pending {
            let source = match &cmd {
                SttCommand::AudioChunk { source, .. } => Some(*source),
                SttCommand::SpeechEnd { source, .. } => Some(*source),
                SttCommand::Flush => { flush = true; None }
                SttCommand::Quit => { quit = true; None }
                SttCommand::FinalPolish { .. } => None,
            };
            if let SttCommand::FinalPolish { full_audio } = cmd {
                final_polish_audio = Some(full_audio);
            } else {
                match source {
                    Some(SttSource::Mic) => mic_cmds.push(cmd),
                    Some(SttSource::Sys) => sys_cmds.push(cmd),
                    None => {} // Flush/Quit already handled above
                }
            }
        }

        // Handle flush first.
        if flush {
            // Commit whatever was pending to note (if active).
            if !mic_whisper_pending.is_empty() {
                if !mic_committed.is_empty() && !mic_committed.ends_with(' ') {
                    mic_committed.push(' ');
                }
                mic_committed.push_str(&mic_whisper_pending);
            }
            if let Some(ref mut c) = corrector { c.reset(); }
            mic_pending_buf.clear();
            mic_whisper_pending.clear();
            mic_typed_pending.clear();
            mic_prev_whisper.clear();
            mic_pending_since = 0;
            mic_stable = 0;
            session_input_typed.clear();
            sys_pending_buf.clear();
            sys_pending_since = 0;
            eprintln!("[CLX] stt-worker: flushed transcription state");
        }

        // Process mic commands first (priority).
        for cmd in mic_cmds {
            match cmd {
                SttCommand::AudioChunk { samples, timestamp_secs, .. } => {
                    mic_pending_buf.extend_from_slice(&samples);
                    mic_pending_since += samples.len();
                    mic_syl_detector.push_chunk(&samples, timestamp_secs);

                    // Hard cap: if mic buffer exceeds 10s, it's a VAD false-hold.
                    // Force-finalize so we don't waste CPU on ever-growing nospeech.
                    const MIC_PENDING_MAX: usize = 160_000; // 10s at 16kHz
                    if mic_pending_buf.len() > MIC_PENDING_MAX {
                        eprintln!("[CLX] stt-worker: mic pending >10s — forcing finalize");
                        process_mic_speech_end(
                            &mic_pending_buf, timestamp_secs,
                            &mut engine, &mut corrector, platform,
                            note_active, input_active, has_sys,
                            &mut mic_committed, &mut mic_whisper_pending,
                            &mut mic_typed_pending, &mut mic_prev_whisper,
                            &mut mic_stable, &mut session_input_typed,
                            &sys_committed, &sys_whisper_pending,
                            &mut note_srt, &mut srt_index,
                            &polish_chain,
                        );
                        mic_pending_buf.clear();
                        mic_pending_since = 0;
                    }

                    // Transcribe when enough new samples have accumulated.
                    if mic_pending_since >= STREAMING_CHUNK_SAMPLES {
                        mic_new_samples_since_commit += STREAMING_CHUNK_SAMPLES;
                        let result = process_mic_streaming(
                            &mic_pending_buf, timestamp_secs,
                            &mut engine, &mut corrector, platform,
                            note_active, input_active, has_sys,
                            &mut mic_committed, &mut mic_whisper_pending,
                            &mut mic_typed_pending, &mut mic_prev_whisper,
                            &mut mic_stable,
                            mic_new_samples_since_commit,
                            &mut session_input_typed,
                            &sys_committed, &sys_whisper_pending,
                            &mut note_srt, &mut srt_index,
                            &mut hum_pending,
                            mic_syl_detector.speed_factor(4.5), // 4.5 syl/s = normal English
                        );
                        mic_pending_since = 0;
                        subtitle_dirty = true; // mic text changed
                        match result {
                            Some(true) => {
                                // Clear buffer on commit to prevent re-transcribing
                                // committed words (causes overlapping text).
                                mic_pending_buf.clear();
                                mic_new_samples_since_commit = 0;
                            }
                            Some(false) => {
                                // Speech detected but not yet stable — keep accumulating context.
                                // Safety cap: trim to 5s to prevent runaway growth.
                                const MIC_SPEECH_MAX: usize = 80_000; // 5s at 16kHz
                                if mic_pending_buf.len() > MIC_SPEECH_MAX {
                                    let drain = mic_pending_buf.len() - MIC_SPEECH_MAX / 2;
                                    mic_pending_buf.drain(..drain);
                                }
                            }
                            None => {
                                // Nospeech/silence — trim to 1s to avoid noise contamination.
                                const MIC_CONTEXT_KEEP: usize = 16_000; // 1s at 16kHz
                                if mic_pending_buf.len() > MIC_CONTEXT_KEEP {
                                    let drain = mic_pending_buf.len() - MIC_CONTEXT_KEEP;
                                    mic_pending_buf.drain(..drain);
                                }
                            }
                        }
                    }
                }
                SttCommand::SpeechEnd { timestamp_secs, .. } => {
                    let pc = live_config.lock().unwrap().stt_polish_chain.clone();
                    process_mic_speech_end(
                        &mic_pending_buf, timestamp_secs,
                        &mut engine, &mut corrector, platform,
                        note_active, input_active, has_sys,
                        &mut mic_committed, &mut mic_whisper_pending,
                        &mut mic_typed_pending, &mut mic_prev_whisper,
                        &mut mic_stable, &mut session_input_typed,
                        &sys_committed, &sys_whisper_pending,
                        &mut note_srt, &mut srt_index,
                        &pc,
                    );
                    // Clear buffer after speech end to prevent overlap.
                    mic_pending_buf.clear();
                    mic_pending_since = 0;
                    mic_new_samples_since_commit = 0;
                }
                _ => {}
            }
        }

        // Then process sys commands.
        let mut sys_subtitle_dirty = false;
        for cmd in sys_cmds {
            match cmd {
                SttCommand::AudioChunk { samples, timestamp_secs, .. } => {
                    sys_pending_buf.extend_from_slice(&samples);
                    sys_pending_since += samples.len();
                    sys_syl_detector.push_chunk(&samples, timestamp_secs);

                    // Use larger streaming interval for sys (3x slower than mic).
                    const SYS_STREAMING_INTERVAL: usize = STREAMING_CHUNK_SAMPLES * 3;
                    if sys_pending_since >= SYS_STREAMING_INTERVAL {
                        let committed = process_sys_streaming(
                            &sys_pending_buf, timestamp_secs,
                            &mut engine,
                            &mut sys_committed, &mut sys_whisper_pending,
                            &mut sys_prev_whisper, &mut sys_stable,
                            note_active, has_sys,
                            &mut note_srt, &mut srt_index,
                            sys_syl_detector.speed_factor(5.5), // ~5.5 syl/s avg (mixed lang)
                        );
                        sys_pending_since = 0;
                        sys_subtitle_dirty = true;
                        if committed {
                            sys_pending_buf.clear();
                        }
                    }
                }
                SttCommand::SpeechEnd { timestamp_secs, .. } => {
                    process_sys_speech_end(
                        &sys_pending_buf, timestamp_secs,
                        &mut engine,
                        &mut sys_committed, &mut sys_whisper_pending,
                        &mut sys_prev_whisper, &mut sys_stable,
                        note_active, has_sys,
                        &mut note_srt, &mut srt_index,
                        sys_syl_detector.speed_factor(5.5),
                    );
                    sys_pending_buf.clear();
                    sys_pending_since = 0;
                    sys_subtitle_dirty = true;
                }
                _ => {}
            }
        }
        // Unified subtitle update — triggered by ANY track change (mic, sys, or hum).
        if sys_subtitle_dirty || subtitle_dirty {
            subtitle_dirty = false;
            let sys_raw = format!("{}{}", sys_committed, sys_whisper_pending);
            let sys_display = last_n_chars(&sys_raw, 200);
            // Only show mic text if there's active pending speech (not stale committed text).
            let mic_raw_tmp;
            let mic_display = if !mic_whisper_pending.is_empty() {
                mic_raw_tmp = format!("{}{}", mic_committed, mic_whisper_pending);
                last_n_chars(&mic_raw_tmp, 80)
            } else {
                ""
            };
            let hum_display = last_n_chars(&hum_pending, 80);
            let mut parts = Vec::new();
            if !mic_display.is_empty() { parts.push(format!("\u{1F3A4} {}", mic_display)); }
            if !sys_display.is_empty() { parts.push(format!("\u{1F50A} {}", sys_display)); }
            if !hum_display.is_empty() { parts.push(format!("\u{1F3B5} {}", hum_display)); }
            if !parts.is_empty() {
                let subtitle = parts.join("\n");
                let preview: String = subtitle.chars().take(60).collect();
                eprintln!("[CLX] stt-worker: pushing sys subtitle ({} chars): {:?}", subtitle.chars().count(), preview);
                platform.update_voice_subtitle(&subtitle);
            }
            // Log sys track + combined log for debugging.
            {
                use std::io::Write;
                use std::time::SystemTime;
                let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
                if !sys_display.is_empty() {
                    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open("/tmp/clx-voice-sys.log") {
                        let _ = writeln!(f, "[{:.3}] {}", secs, sys_display.trim());
                    }
                }
                // Also write combined subtitle to the main log file.
                if !parts.is_empty() {
                    let combined = parts.join("\n");
                    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
                        let _ = writeln!(f, "{}", combined);
                    }
                }
            }
        }

        // ── FinalPolish: re-transcribe entire session with best quality ───
        if let Some(full_audio) = final_polish_audio {
            if !full_audio.is_empty() && input_active.load(Ordering::Relaxed) {
                eprintln!("[CLX] stt-worker: final polish ({} samples = {:.1}s, typed {:?})",
                    full_audio.len(), full_audio.len() as f64 / 16000.0, session_input_typed);
                platform.update_voice_subtitle("✨ polishing...");

                // Transcribe with local engine first to get raw text.
                let raw_text = transcribe_local(&full_audio, &mut engine);
                // Apply best polish chain: cloud (Gemini) first for highest quality.
                let polished = if !raw_text.is_empty() {
                    polish_stt_result(&raw_text, &full_audio, &mut corrector, "gemini,mlx,llm-corrector,raw")
                } else {
                    String::new()
                };

                if !polished.is_empty() && polished != session_input_typed {
                    eprintln!("[CLX] stt-worker: final polish: {:?} → {:?}", session_input_typed, polished);
                    type_replace(&session_input_typed, &polished, platform);
                    session_input_typed = polished.clone();
                    platform.update_voice_subtitle(&format!("✨ {}", polished));
                } else {
                    eprintln!("[CLX] stt-worker: final polish: no change (raw={:?})", raw_text);
                }
            }
        }

        if quit {
            // Flush remaining mic audio on session end (non-input mode).
            if mic_pending_buf.len() > 4800 && !input_active.load(Ordering::Relaxed) {
                let mut final_text = transcribe_local(&mic_pending_buf, &mut engine);
                if !final_text.is_empty() {
                    if let Some(ref mut c) = corrector {
                        final_text = c.correct(&final_text);
                    }
                }
            }
            break;
        }
    }

    // Return engine and corrector to the shared locks for reuse.
    *engine_lock.lock().unwrap() = engine;
    *corrector_lock.lock().unwrap() = corrector;

    eprintln!("[CLX] stt-worker: exiting");
    SttWorkerResult { note_srt }
}

/// Process streaming mic transcription (mid-speech update from accumulated buffer).
/// Returns `true` if a commit happened (caller should clear the pending buffer).
#[allow(clippy::too_many_arguments)]
fn process_mic_streaming(
    pending_buf: &[f32],
    timestamp_secs: f64,
    engine: &mut Option<SttEngine>,
    corrector: &mut Option<crate::stt_corrector::SttCorrector>,
    platform: &Arc<dyn Platform>,
    note_active: &Arc<AtomicBool>,
    input_active: &Arc<AtomicBool>,
    has_sys: bool,
    mic_committed: &mut String,
    mic_whisper_pending: &mut String,
    mic_typed_pending: &mut String,
    mic_prev_whisper: &mut String,
    mic_stable: &mut usize,
    mic_new_samples: usize,
    session_input_typed: &mut String,
    sys_committed: &str,
    sys_whisper_pending: &str,
    note_srt: &mut String,
    srt_index: &mut usize,
    hum_pending: &mut String,
    speed_factor: f32,
) -> Option<bool> {
    // Time-stretch audio if speech speed > 1.2x (e.g., 2x playback).
    let stretched;
    let buf = if speed_factor > 1.2 {
        eprintln!("[CLX] stt-worker: mic time-stretch {:.1}x → 1x", speed_factor);
        stretched = time_stretch(pending_buf, speed_factor);
        &stretched[..]
    } else {
        pending_buf
    };
    let (new_text, is_music_tag) = transcribe_local_tagged(buf, engine);

    // Always run pitch detection — it's cheap (McLeod on 2048 samples).
    let pitch_result = detect_pitch(pending_buf);

    // Detect humming: SenseVoice tagged [music], OR pitch found with no/short text.
    // Humming often transcribes as short noise like "嗯", "Mmm", "Hmm", etc.
    let text_is_noise = new_text.is_empty() || new_text.chars().count() <= 4;
    let is_humming = pitch_result.is_some() && (is_music_tag || text_is_noise);

    if pitch_result.is_some() {
        eprintln!("[CLX] voice: pitch={:?} text={:?} music_tag={} → humming={}",
            pitch_result, new_text, is_music_tag, is_humming);
    }

    // If music/humming detected, fork audio to pitch detection instead of mic text.
    if is_humming {
        if let Some(note) = pitch_result {
            if !hum_pending.is_empty() { hum_pending.push(' '); }
            hum_pending.push_str(&note);
            // Log to hum file.
            {
                use std::io::Write;
                use std::time::SystemTime;
                let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open("/tmp/clx-voice-hum.log") {
                    let _ = writeln!(f, "[{:.3}] {}", secs, note);
                }
            }
        }
        return None; // don't update mic text path — subtitle updated by caller
    }

    if new_text.is_empty() {
        return None; // nospeech — caller should trim context window
    }
    *mic_whisper_pending = new_text.clone();

    // Voice Input: type at cursor only if input mode is active.
    let is_input = input_active.load(Ordering::Relaxed);
    if is_input {
        let old_len = mic_typed_pending.chars().count();
        type_diff(mic_typed_pending, &new_text, platform);
        if new_text.starts_with(mic_typed_pending.as_str()) {
            *mic_typed_pending = new_text.clone();
            // Update session tracking: type_diff only appends, so extend session_input_typed.
            let new_suffix: String = mic_typed_pending.chars().skip(old_len).collect();
            session_input_typed.push_str(&new_suffix);
        }
    }

    // Update overlay subtitle — show committed + pending as one continuous text.
    // The committed prefix is locked, the pending tail keeps updating.
    let display_pending = if is_input { mic_typed_pending.as_str() } else { mic_whisper_pending.as_str() };
    let separator = if mic_committed.is_empty() || display_pending.is_empty() {
        ""
    } else if mic_committed.ends_with('\n') || display_pending.starts_with('\n') {
        ""
    } else {
        let last_committed = mic_committed.chars().last().unwrap_or(' ');
        if matches!(last_committed, '.' | '?' | '!' | '。' | '？' | '！') {
            " "
        } else if last_committed == ' ' || display_pending.starts_with(' ') {
            ""
        } else {
            " "
        }
    };
    let full_text = format!("{}{}{}", mic_committed, separator, display_pending);
    // Show only the last ~200 chars so the overlay doesn't overflow.
    let visible_text = last_n_chars(&full_text, 200);
    let visible: Vec<&str> = if visible_text.is_empty() { vec![] } else { vec![&visible_text] };

    let subtitle = if has_sys {
        let sys_text = format!("{}{}", sys_committed, sys_whisper_pending);
        let sys_last: String = sys_text.lines().last().unwrap_or("").chars().take(80).collect();
        let mut parts = Vec::new();
        for l in &visible {
            parts.push(format!("🎤 {}", l));
        }
        if !sys_last.is_empty() {
            parts.push(format!("🔊 {}", sys_last));
        }
        parts.join("\n")
    } else {
        visible.join("\n")
    };
    // Subtitle update removed — handled by unified path in stt_worker_loop.

    // Log combined + separate mic/sys logs for debugging.
    {
        use std::io::Write;
        use std::time::SystemTime;
        let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
            let _ = writeln!(f, "{}", subtitle);
        }
        let mic_full = format!("{}{}{}", mic_committed, separator, display_pending);
        if !mic_full.trim().is_empty() {
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open("/tmp/clx-voice-mic.log") {
                let _ = writeln!(f, "[{:.3}] {}", secs, mic_full.trim());
            }
        }
        if has_sys {
            let sys_text = format!("{}{}", sys_committed, sys_whisper_pending);
            if !sys_text.trim().is_empty() {
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open("/tmp/clx-voice-sys.log") {
                    let _ = writeln!(f, "[{:.3}] {}", secs, sys_text.trim());
                }
            }
        }
    }

    // ── Sliding-window commit: only lock in the stable prefix ──────────
    // Find the common prefix (by chars) between current and previous transcription.
    let common_prefix_len = mic_whisper_pending.chars()
        .zip(mic_prev_whisper.chars())
        .take_while(|(a, b)| a == b)
        .count();

    // Track how long the stable prefix has been unchanged.
    if common_prefix_len > 0 && common_prefix_len >= *mic_stable {
        // Prefix grew or stayed the same — keep counting.
        // mic_stable tracks the stable prefix length in chars.
        // mic_prev_stable_time tracks when this prefix length was first seen.
        if *mic_stable == common_prefix_len {
            // Same prefix length — stability counter increments (each cycle ~100ms).
            // We reuse mic_stable as a cycle counter for how long the prefix has been stable.
        } else {
            // Prefix grew — new stable frontier, reset timer.
            *mic_stable = common_prefix_len;
        }
    } else if common_prefix_len < *mic_stable {
        // Prefix shrank (Whisper rewrote earlier text) — reset to new common prefix.
        *mic_stable = common_prefix_len;
    }
    *mic_prev_whisper = mic_whisper_pending.clone();

    // Count how many cycles the prefix at `mic_stable` chars has been unchanged.
    // We track this separately: compare prefix of current vs prefix of previous.
    let prefix_unchanged = {
        let cur_prefix: String = mic_whisper_pending.chars().take(*mic_stable).collect();
        let prev_prefix: String = mic_prev_whisper.chars().take(*mic_stable).collect();
        cur_prefix == prev_prefix && !cur_prefix.is_empty()
    };
    // Use mic_new_samples as a rough timer: 16000 samples = 1s.
    // Commit only on stability (3s) or force (5s). Punctuation is NOT used
    // for commit timing — STT may emit spurious periods. Punctuation is only
    // used for \n insertion when text is committed.
    let stable_duration_samples = mic_new_samples;
    let should_commit_prefix = prefix_unchanged
        && *mic_stable > 0
        && stable_duration_samples > 48_000; // ~3s of new audio
    let force_commit_all = pending_buf.len() > 80_000; // 5s

    if should_commit_prefix || force_commit_all {
        let commit_chars = if force_commit_all {
            mic_whisper_pending.chars().count()
        } else {
            *mic_stable
        };
        let commit_text: String = mic_whisper_pending.chars().take(commit_chars).collect();
        let remaining: String = mic_whisper_pending.chars().skip(commit_chars).collect();

        if !commit_text.is_empty() {
            // In input mode: the stable prefix is already typed. We just need to
            // track it in session_input_typed and update mic_typed_pending to reflect
            // that only the remaining tail is "pending".
            if input_active.load(Ordering::Relaxed) {
                // No need to retype — the prefix is already in the input box.
                // Just adjust tracking: committed portion moves out of mic_typed_pending.
                let typed_chars = mic_typed_pending.chars().count();
                if typed_chars >= commit_chars {
                    *mic_typed_pending = mic_typed_pending.chars().skip(commit_chars).collect();
                } else {
                    mic_typed_pending.clear();
                }
            }

            // Add \n if previous committed text ends with sentence punctuation.
            // Otherwise add space so words don't stick.
            if !mic_committed.is_empty() {
                let last_char = mic_committed.chars().last().unwrap_or(' ');
                if matches!(last_char, '.' | '?' | '!' | '。' | '？' | '！') {
                    mic_committed.push(' ');
                } else if !mic_committed.ends_with(' ') && !commit_text.starts_with(' ') {
                    mic_committed.push(' ');
                }
            }
            mic_committed.push_str(&commit_text);

            // Add SRT entry for voice note.
            if note_active.load(Ordering::Relaxed) {
                *srt_index += 1;
                let start_srt = format_srt_time(timestamp_secs - (pending_buf.len() as f64 / 16000.0).max(0.0));
                let end_srt = format_srt_time(timestamp_secs);
                let label = if has_sys { "\u{1F3A4} " } else { "" };
                note_srt.push_str(&format!("{}\n{} --> {}\n{}{}\n\n", srt_index, start_srt, end_srt, label, commit_text));
            }
            eprintln!("[CLX] stt-worker: sliding commit {:?} (remaining={:?})", commit_text, remaining);

            // Keep the remaining tail as new pending.
            *mic_whisper_pending = remaining;
            mic_prev_whisper.clear();
            *mic_stable = 0;
            return Some(true);
        }
    }
    Some(false) // speech detected but not yet committed
}

/// Process mic speech end (finalize and commit).
#[allow(clippy::too_many_arguments)]
fn process_mic_speech_end(
    pending_buf: &[f32],
    timestamp_secs: f64,
    engine: &mut Option<SttEngine>,
    corrector: &mut Option<crate::stt_corrector::SttCorrector>,
    platform: &Arc<dyn Platform>,
    note_active: &Arc<AtomicBool>,
    input_active: &Arc<AtomicBool>,
    has_sys: bool,
    mic_committed: &mut String,
    mic_whisper_pending: &mut String,
    mic_typed_pending: &mut String,
    mic_prev_whisper: &mut String,
    mic_stable: &mut usize,
    session_input_typed: &mut String,
    _sys_committed: &str,
    _sys_whisper_pending: &str,
    note_srt: &mut String,
    srt_index: &mut usize,
    polish_chain: &str,
) {
    let is_input = input_active.load(Ordering::Relaxed);
    if pending_buf.len() > 4800 {
        let raw_text = transcribe_local(pending_buf, engine);

        // During input mode, skip per-utterance polish — final polish on V release handles it.
        let final_text = if is_input {
            raw_text
        } else {
            polish_stt_result(&raw_text, pending_buf, corrector, polish_chain)
        };

        if !final_text.is_empty() {
            // At speech end, accept rewrites (input mode only).
            if is_input && *mic_typed_pending != final_text {
                // Update session tracking.
                let old_pending_chars = mic_typed_pending.chars().count();
                let session_chars = session_input_typed.chars().count();
                *session_input_typed = session_input_typed.chars().take(session_chars.saturating_sub(old_pending_chars)).collect();
                type_replace(mic_typed_pending, &final_text, platform);
                session_input_typed.push_str(&final_text);
            }
            *mic_whisper_pending = final_text;
        }
    }

    if !mic_committed.is_empty() && !mic_committed.ends_with(' ') {
        mic_committed.push(' ');
    }
    mic_committed.push_str(mic_whisper_pending);
    // Add SRT entry for the utterance end.
    if note_active.load(Ordering::Relaxed) && !mic_whisper_pending.is_empty() {
        *srt_index += 1;
        let start_srt = format_srt_time(timestamp_secs - (pending_buf.len() as f64 / 16000.0).max(0.0));
        let end_srt = format_srt_time(timestamp_secs);
        let label = if has_sys { "\u{1F3A4} " } else { "" };
        note_srt.push_str(&format!("{}\n{} --> {}\n{}{}\n\n", srt_index, start_srt, end_srt, label, mic_whisper_pending));
    }
    eprintln!("[CLX] stt-worker: mic utterance done: {:?}", mic_committed);

    // Log final text.
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
            let _ = writeln!(f, "{}", mic_committed);
        }
    }

    mic_whisper_pending.clear();
    mic_typed_pending.clear();
    mic_prev_whisper.clear();
    *mic_stable = 0;
    mic_committed.clear();
}

/// Process streaming sys transcription (mid-speech update from accumulated buffer).
/// Returns `true` if a commit happened (caller should clear the pending buffer).
#[allow(clippy::too_many_arguments)]
fn process_sys_streaming(
    pending_buf: &[f32],
    timestamp_secs: f64,
    engine: &mut Option<SttEngine>,
    sys_committed: &mut String,
    sys_whisper_pending: &mut String,
    sys_prev_whisper: &mut String,
    sys_stable: &mut usize,
    note_active: &Arc<AtomicBool>,
    has_sys: bool,
    note_srt: &mut String,
    srt_index: &mut usize,
    speed_factor: f32,
) -> bool {
    let stretched;
    let buf = if speed_factor > 1.2 {
        eprintln!("[CLX] stt-worker: sys time-stretch {:.1}x → 1x", speed_factor);
        stretched = time_stretch(pending_buf, speed_factor);
        &stretched[..]
    } else {
        pending_buf
    };
    let text = transcribe_local(buf, engine);
    if !text.is_empty() {
        *sys_whisper_pending = text;
    }

    // Stability commit for sys track.
    if *sys_whisper_pending == *sys_prev_whisper && !sys_whisper_pending.is_empty() {
        *sys_stable += 1;
    } else {
        *sys_stable = 0;
    }
    *sys_prev_whisper = sys_whisper_pending.clone();

    if (*sys_stable >= 2 && pending_buf.len() > 32_000) || pending_buf.len() > 80_000 {
        // Add SRT entry.
        if note_active.load(Ordering::Relaxed) && !sys_whisper_pending.is_empty() && has_sys {
            *srt_index += 1;
            let start_srt = format_srt_time(timestamp_secs - (pending_buf.len() as f64 / 16000.0).max(0.0));
            let end_srt = format_srt_time(timestamp_secs);
            note_srt.push_str(&format!("{}\n{} --> {}\n\u{1F50A} {}\n\n", srt_index, start_srt, end_srt, sys_whisper_pending));
        }
        if !sys_committed.is_empty() && !sys_committed.ends_with(' ') { sys_committed.push(' '); }
        sys_committed.push_str(sys_whisper_pending);
        // Cap committed buffer to last 200 chars (char boundary safe).
        const MAX_COMMITTED: usize = 200;
        if sys_committed.chars().count() > MAX_COMMITTED {
            let drop = sys_committed.chars().count() - MAX_COMMITTED;
            let byte_pos = sys_committed.char_indices().nth(drop).map(|(i, _)| i).unwrap_or(0);
            sys_committed.drain(..byte_pos);
        }
        eprintln!("[CLX] stt-worker: sys committed {:?}", sys_whisper_pending);
        sys_whisper_pending.clear();
        sys_prev_whisper.clear();
        *sys_stable = 0;
        return true;
    }
    false
}

/// Process sys speech end (finalize and commit).
#[allow(clippy::too_many_arguments)]
fn process_sys_speech_end(
    pending_buf: &[f32],
    timestamp_secs: f64,
    engine: &mut Option<SttEngine>,
    sys_committed: &mut String,
    sys_whisper_pending: &mut String,
    sys_prev_whisper: &mut String,
    sys_stable: &mut usize,
    note_active: &Arc<AtomicBool>,
    has_sys: bool,
    note_srt: &mut String,
    srt_index: &mut usize,
    speed_factor: f32,
) {
    if pending_buf.len() > 4800 {
        let stretched;
        let buf = if speed_factor > 1.2 {
            stretched = time_stretch(pending_buf, speed_factor);
            &stretched[..]
        } else {
            pending_buf
        };
        let text = transcribe_local(buf, engine);
        if !text.is_empty() { *sys_whisper_pending = text; }
    }
    if note_active.load(Ordering::Relaxed) && !sys_whisper_pending.is_empty() && has_sys {
        *srt_index += 1;
        let start_srt = format_srt_time(timestamp_secs - (pending_buf.len() as f64 / 16000.0).max(0.0));
        let end_srt = format_srt_time(timestamp_secs);
        note_srt.push_str(&format!("{}\n{} --> {}\n\u{1F50A} {}\n\n", srt_index, start_srt, end_srt, sys_whisper_pending));
    }
    if !sys_committed.is_empty() && !sys_committed.ends_with(' ') { sys_committed.push(' '); }
    sys_committed.push_str(sys_whisper_pending);
    const MAX_COMMITTED_END: usize = 200;
    if sys_committed.chars().count() > MAX_COMMITTED_END {
        let drop = sys_committed.chars().count() - MAX_COMMITTED_END;
        let byte_pos = sys_committed.char_indices().nth(drop).map(|(i, _)| i).unwrap_or(0);
        sys_committed.drain(..byte_pos);
    }
    eprintln!("[CLX] stt-worker: sys utterance: {:?}", sys_whisper_pending);
    sys_whisper_pending.clear();
    sys_prev_whisper.clear();
    *sys_stable = 0;
}

/// Generate a timestamp string like "2026-03-19T120000".
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let d = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let mut days = (secs / 86400) as i64;
    // Civil date from days since 1970-01-01 (Rata Die algorithm).
    let mut y: i64 = 1970;
    loop {
        let ylen: i64 = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if days < ylen { break; }
        days -= ylen;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays: [i64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mon: i64 = 0;
    for md in &mdays {
        if days < *md { break; }
        days -= *md;
        mon += 1;
    }
    let day = days + 1;
    let mon = mon + 1;
    format!("{y:04}-{mon:02}-{day:02}T{h:02}{m:02}{s:02}")
}

/// Keep only the last `max_chars` characters of a string (char-boundary safe).
/// Used to cap each subtitle track to ~4 lines of display text.
fn last_n_chars(s: &str, max_chars: usize) -> &str {
    let total = s.chars().count();
    if total <= max_chars { return s; }
    let skip = total - max_chars;
    let byte_pos = s.char_indices().nth(skip).map(|(i, _)| i).unwrap_or(0);
    &s[byte_pos..]
}

/// Polish STT output using the best available method.
/// Cascade: MLX local (if running + enough RAM) → Gemini cloud → LLM corrector → raw.
fn polish_stt_result(
    raw_text: &str,
    audio_samples: &[f32],
    corrector: &mut Option<crate::stt_corrector::SttCorrector>,
    chain: &str,
) -> String {
    let chain = if chain.is_empty() { "mlx,gemini,llm,raw" } else { chain };
    polish_stt_with_chain(raw_text, audio_samples, corrector, chain)
}

/// Polish STT result using a configurable fallback chain.
/// `chain` is comma-separated: "mlx,gemini,llm,raw".
fn polish_stt_with_chain(
    raw_text: &str,
    audio_samples: &[f32],
    corrector: &mut Option<crate::stt_corrector::SttCorrector>,
    chain: &str,
) -> String {
    if raw_text.trim().is_empty() { return raw_text.to_string(); }

    for stage in chain.split(',').map(|s| s.trim()) {
        match stage {
            s if s.starts_with("mlx") => {
                if has_enough_free_memory(2_000) {
                    if let Ok(polished) = polish_via_local_llm(raw_text) {
                        if !polished.is_empty() {
                            eprintln!("[CLX] stt-polish: MLX local: {:?} → {:?}", raw_text, polished);
                            return polished;
                        }
                    }
                }
            }
            s if s.starts_with("gemini") => {
                let gemini_key = read_gemini_key();
                if !gemini_key.is_empty() {
                    match crate::cloud_stt::transcribe_gemini(audio_samples, &gemini_key) {
                        Ok(text) if !text.is_empty() => {
                            eprintln!("[CLX] stt-polish: Gemini cloud: {:?}", text);
                            return text;
                        }
                        _ => {}
                    }
                }
            }
            "llm-corrector" | "llm" => {
                if let Some(ref mut c) = corrector {
                    let corrected = c.correct(raw_text);
                    if corrected != raw_text {
                        eprintln!("[CLX] stt-polish: LLM corrector: {:?} → {:?}", raw_text, corrected);
                        return corrected;
                    }
                }
            }
            "raw" => return raw_text.to_string(),
            _ => {}
        }
    }

    raw_text.to_string()
}

/// Polish raw STT text via local MLX LLM server (OpenAI-compat at port 8321).
#[cfg(not(target_arch = "wasm32"))]
fn polish_via_local_llm(raw_text: &str) -> Result<String, String> {
    // Quick check: is MLX server running?
    let base = "http://localhost:8321";
    if ureq::get(&format!("{}/v1/models", base)).call().is_err() {
        return Err("MLX server not running".into());
    }

    let body = serde_json::json!({
        "model": "mlx-community/Qwen2.5-3B-Instruct-4bit",
        "messages": [
            {"role": "system", "content": "You correct speech-to-text errors. Fix obvious mistakes. Keep the original language. Return ONLY the corrected text, nothing else."},
            {"role": "user", "content": raw_text}
        ],
        "stream": false,
        "max_tokens": 200,
        "temperature": 0.0
    });

    let resp = ureq::post(&format!("{}/v1/chat/completions", base))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("MLX: {}", e))?;

    let result: serde_json::Value = serde_json::from_str(
        &resp.into_string().map_err(|e| format!("read: {}", e))?
    ).map_err(|e| format!("parse: {}", e))?;

    let text = result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .trim_end_matches("<|im_end|>") // strip Qwen stop token
        .trim()
        .to_string();

    Ok(text)
}

#[cfg(target_arch = "wasm32")]
fn polish_via_local_llm(_raw_text: &str) -> Result<String, String> {
    Err("not on wasm".into())
}

/// Check if the system has enough free memory (in MB).
fn has_enough_free_memory(required_mb: u64) -> bool {
    #[cfg(target_os = "macos")]
    {
        // Use vm_stat to check free + inactive pages.
        if let Ok(output) = std::process::Command::new("vm_stat").output() {
            let text = String::from_utf8_lossy(&output.stdout);
            let page_size: u64 = 16384; // macOS ARM page size
            let mut free: u64 = 0;
            let mut inactive: u64 = 0;
            for line in text.lines() {
                if line.contains("Pages free") {
                    if let Some(n) = line.split(':').nth(1) {
                        free = n.trim().trim_end_matches('.').parse().unwrap_or(0);
                    }
                }
                if line.contains("Pages inactive") {
                    if let Some(n) = line.split(':').nth(1) {
                        inactive = n.trim().trim_end_matches('.').parse().unwrap_or(0);
                    }
                }
            }
            let available_mb = (free + inactive) * page_size / 1024 / 1024;
            return available_mb > required_mb;
        }
    }
    // Default: assume enough.
    true
}

/// Read Gemini API key from env or .env.local.
fn read_gemini_key() -> String {
    std::env::var("GEMINI_API_KEY")
        .or_else(|_| {
            for dir in &[".", &format!("{}/CapsLockX", std::env::var("HOME").unwrap_or_default())] {
                let path = std::path::Path::new(dir).join(".env.local");
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(val) = content.lines().find_map(|l| l.strip_prefix("GEMINI_API_KEY=")) {
                        return Ok(val.to_string());
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_or_default()
}

/// Format seconds as SRT timestamp: HH:MM:SS,mmm
fn format_srt_time(secs: f64) -> String {
    let secs = secs.max(0.0);
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    let ms = ((secs % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

// ── Streaming diff helpers ────────────────────────────────────────────────────

/// Time-stretch audio by a factor (e.g., 2.0 = slow down to half speed).
/// Uses linear interpolation resampling. Factor > 1 = slow down (more samples).
fn time_stretch(samples: &[f32], factor: f32) -> Vec<f32> {
    if (factor - 1.0).abs() < 0.1 || samples.is_empty() { return samples.to_vec(); }
    let out_len = (samples.len() as f64 * factor as f64) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / factor as f64;
        let idx = src as usize;
        let frac = (src - idx as f64) as f32;
        let s0 = samples.get(idx).copied().unwrap_or(0.0);
        let s1 = samples.get(idx + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

/// Transcribe audio locally (sherpa or whisper). Returns text or empty.
fn transcribe_local(samples: &[f32], engine: &mut Option<SttEngine>) -> String {
    transcribe_local_tagged(samples, engine).0
}

/// Transcribe and also return whether music/humming was detected.
fn transcribe_local_tagged(samples: &[f32], engine: &mut Option<SttEngine>) -> (String, bool) {
    // Pad to 1s minimum if needed; avoid cloning when not padding.
    let padded;
    let samples = if samples.len() < 16000 {
        padded = {
            let mut v = samples.to_vec();
            v.resize(16000, 0.0);
            v
        };
        &padded[..]
    } else {
        samples
    };

    if let Some(ref mut stt) = engine {
        match stt.transcribe_tagged(samples) {
            Ok(output) => {
                stt.check_pending_upgrade();
                (output.text, output.is_music)
            }
            Err(e) => {
                eprintln!("[CLX] voice: STT error: {e}");
                (String::new(), false)
            }
        }
    } else {
        (String::new(), false)
    }
}

/// Type only the difference between old and new text.
/// **Append-only during streaming** — only types new characters when
/// the new text starts with the old text (pure extension). If Whisper
/// rewrites earlier text, we skip the update and wait for it to stabilize.
/// Full replacement only happens at commit/speech-end boundaries.
fn type_diff(old: &str, new: &str, platform: &Arc<dyn Platform>) {
    if new.starts_with(old) {
        // Pure extension — just type the new suffix.
        let suffix: String = new.chars().skip(old.chars().count()).collect();
        if !suffix.is_empty() {
            eprintln!("[CLX] voice: +{:?}", suffix);
            platform.type_text(&suffix);
        }
    } else {
        // Whisper rewrote earlier text — skip this update during streaming.
        // The caller will handle it at commit time.
        let common = old.chars().zip(new.chars()).take_while(|(a, b)| a == b).count();
        let old_len = old.chars().count();
        eprintln!("[CLX] voice: rewrite detected (common={}/{}, new_len={}), skipping",
            common, old_len, new.chars().count());
        return;
    }

    // Log current state of what's in the input box.
    if !new.is_empty() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
            let _ = writeln!(f, "{}", new);
        }
    }
}

/// Full replacement: delete old text and type new text.
/// Used at commit/speech-end boundaries when we accept a rewrite.
fn type_replace(old: &str, new: &str, platform: &Arc<dyn Platform>) {
    let old_chars = old.chars().count();
    for _ in 0..old_chars {
        platform.key_tap(KeyCode::Backspace);
    }
    if !new.is_empty() {
        platform.type_text(new);
    }
    eprintln!("[CLX] voice: replace {:?} → {:?}", old, new);
}

// NLMS removed — replaced by WebRTC AEC3 (aec3 crate).

// ── Syllable Rate Detector (speech speed estimation) ─────────────────────────
/// Estimates speech speed by counting syllable nuclei via zero-crossings
/// on high-pass filtered RMS energy. ~4.5 syl/s = normal English, ~7 = Japanese.
struct SyllableRateDetector {
    prev_rms: f32,
    filtered: f32,
    prev_positive: bool,
    last_crossing_t: f64,
    crossing_times: std::collections::VecDeque<f64>,
}

impl SyllableRateDetector {
    fn new() -> Self {
        Self {
            prev_rms: 0.0,
            filtered: 0.0,
            prev_positive: false,
            last_crossing_t: 0.0,
            crossing_times: std::collections::VecDeque::new(),
        }
    }

    /// Feed an audio chunk. `timestamp` = seconds since session start.
    fn push_chunk(&mut self, samples: &[f32], timestamp: f64) {
        if samples.is_empty() { return; }
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        // High-pass filter: keeps 2-10 Hz syllable-rate modulation.
        const ALPHA: f32 = 0.9;
        self.filtered = ALPHA * (self.filtered + rms - self.prev_rms);
        self.prev_rms = rms;

        // Detect positive-going zero-crossing with minimum 70ms interval.
        let positive = self.filtered > 0.0;
        if positive && !self.prev_positive && (timestamp - self.last_crossing_t) > 0.070 {
            self.crossing_times.push_back(timestamp);
            self.last_crossing_t = timestamp;
            // Keep only last 5 seconds.
            while self.crossing_times.front().map_or(false, |&t| timestamp - t > 5.0) {
                self.crossing_times.pop_front();
            }
        }
        self.prev_positive = positive;
    }

    /// Syllables per second over the last `window` seconds.
    fn syllables_per_sec(&self, window: f64) -> f32 {
        if self.crossing_times.len() < 2 { return 0.0; }
        let now = self.crossing_times.back().copied().unwrap_or(0.0);
        let count = self.crossing_times.iter()
            .filter(|&&t| now - t <= window)
            .count();
        if count < 2 { return 0.0; }
        count as f32 / window as f32
    }

    /// Estimate speed factor relative to expected syllable rate.
    /// Returns 1.0 for normal speed, 2.0 for double speed, etc.
    fn speed_factor(&self, expected_syl_per_sec: f32) -> f32 {
        let measured = self.syllables_per_sec(3.0);
        if measured < 1.0 || expected_syl_per_sec < 1.0 { return 1.0; }
        (measured / expected_syl_per_sec).clamp(0.5, 4.0)
    }
}

// ── Chord / Pitch Detection (humming → chord names or note names) ────────────

const NOTE_NAMES: [&str; 12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];

/// Convert frequency in Hz to note name (e.g., 440.0 → "A4").
fn hz_to_note(freq: f32) -> String {
    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    let note_i = (midi.round() as i32).rem_euclid(12) as usize;
    let octave = (midi.round() as i32) / 12 - 1;
    format!("{}{}", NOTE_NAMES[note_i], octave)
}

/// Chord templates: (name_suffix, intervals from root).
/// Each chord type is defined by its intervals in semitones.
const CHORD_TEMPLATES: &[(&str, &[usize])] = &[
    ("",     &[0, 4, 7]),       // major
    ("m",    &[0, 3, 7]),       // minor
    ("7",    &[0, 4, 7, 10]),   // dominant 7th
    ("m7",   &[0, 3, 7, 10]),   // minor 7th
    ("maj7", &[0, 4, 7, 11]),   // major 7th
    ("dim",  &[0, 3, 6]),       // diminished
    ("aug",  &[0, 4, 8]),       // augmented
    ("sus4", &[0, 5, 7]),       // suspended 4th
    ("sus2", &[0, 2, 7]),       // suspended 2nd
    ("5",    &[0, 7]),          // power chord
];

/// Compute 12-bin chroma vector from audio using FFT.
/// Each bin = total energy for that pitch class (C=0, C#=1, ... B=11).
fn compute_chroma(samples: &[f32], sample_rate: u32) -> [f32; 12] {
    let n = samples.len().next_power_of_two();
    let mut padded = vec![0.0f32; n];
    padded[..samples.len()].copy_from_slice(samples);

    // Apply Hann window.
    for i in 0..samples.len() {
        let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / samples.len() as f32).cos());
        padded[i] *= w;
    }

    // Compute chroma by targeting specific musical frequencies directly.
    // Instead of full DFT, compute Goertzel magnitude for each semitone
    // across octaves 2-6 (C2=65Hz to B6=1976Hz). Much faster than full DFT.
    let mut chroma = [0.0f32; 12];

    for octave in 2..=6 {
        for pitch_class in 0..12 {
            // Frequency of this note: C2=65.41, C#2=69.30, ...
            let midi = (octave + 1) * 12 + pitch_class as i32; // C2 = MIDI 36
            let freq = 440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0);
            if freq < 50.0 || freq > 2500.0 { continue; }

            // Goertzel algorithm: O(N) for single frequency bin.
            let k = (freq * n as f32 / sample_rate as f32).round();
            let w = 2.0 * std::f32::consts::PI * k / n as f32;
            let coeff = 2.0 * w.cos();
            let (mut s0, mut s1, mut s2) = (0.0f32, 0.0f32, 0.0f32);
            for &x in &padded {
                s0 = x + coeff * s1 - s2;
                s2 = s1;
                s1 = s0;
            }
            let mag = (s1 * s1 + s2 * s2 - coeff * s1 * s2).sqrt();
            chroma[pitch_class as usize] += mag;
        }
    }

    // Normalize.
    let max = chroma.iter().cloned().fold(0.0f32, f32::max);
    if max > 0.0 {
        for c in &mut chroma { *c /= max; }
    }
    chroma
}

/// Match a chroma vector against chord templates. Returns chord name (e.g. "Am7").
fn match_chord(chroma: &[f32; 12]) -> Option<String> {
    let mut best_score = 0.0f32;
    let mut best_name = String::new();

    for root in 0..12 {
        for &(suffix, intervals) in CHORD_TEMPLATES {
            // Build template: 1.0 at chord tones, 0.0 elsewhere.
            let mut template = [0.0f32; 12];
            for &interval in intervals {
                template[(root + interval) % 12] = 1.0;
            }

            // Cosine similarity.
            let dot: f32 = chroma.iter().zip(template.iter()).map(|(a, b)| a * b).sum();
            let norm_a: f32 = chroma.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = template.iter().map(|x| x * x).sum::<f32>().sqrt();
            let score = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };

            if score > best_score {
                best_score = score;
                best_name = format!("{}{}", NOTE_NAMES[root], suffix);
            }
        }
    }

    // Require minimum confidence.
    if best_score > 0.6 { Some(best_name) } else { None }
}

/// Detect chord or single pitch from audio samples.
/// Tries chord detection first, falls back to single note via McLeod.
fn detect_pitch(samples: &[f32]) -> Option<String> {
    if samples.len() < 2048 { return None; }

    // Try chord detection via chroma.
    let start = samples.len().saturating_sub(4096).max(0);
    let chunk = &samples[start..];
    let chroma = compute_chroma(chunk, 16000);

    // Check if there's enough energy for detection.
    let energy: f32 = chroma.iter().sum();
    if energy < 0.5 { return None; }

    if let Some(chord) = match_chord(&chroma) {
        return Some(chord);
    }

    // Fallback: single note via McLeod pitch detector.
    use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
    let start = samples.len().saturating_sub(2048);
    let chunk = &samples[start..start + 2048];
    let mut detector = McLeodDetector::<f32>::new(2048, 1024);
    let result = detector.get_pitch(chunk, 16000, 0.15, 0.5)?;
    if result.frequency >= 80.0 && result.frequency <= 1000.0 && result.clarity > 0.5 {
        Some(hz_to_note(result.frequency))
    } else {
        None
    }
}

// ── Resampling ───────────────────────────────────────────────────────────────

/// Resample audio from `from_rate` to `to_rate` using linear interpolation.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate { return samples.to_vec(); }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 * ratio;
        let idx0 = src_idx as usize;
        let frac = (src_idx - idx0 as f64) as f32;
        let s0 = samples.get(idx0).copied().unwrap_or(0.0);
        let s1 = samples.get(idx0 + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

// ── VAD (Voice Activity Detection) ───────────────────────────────────────────

struct VadState {
    vad: ten_vad_rs::TenVad,
    chunk: Vec<f32>,
    remainder: Vec<f32>,
    pub in_speech: bool,
    speech_frames: usize,
    silence_frames: usize,
    // Configurable thresholds
    cfg_speech_start_prob: f32,
    cfg_speech_end_prob: f32,
    cfg_speech_start_frames: usize,
    cfg_silence_end_frames: usize,
}

/// TEN VAD frame size: 256 samples at 16kHz (16ms).
const TEN_VAD_FRAME_SIZE: usize = 256;

impl VadState {
    fn new() -> Self {
        Self::with_thresholds(SPEECH_START_PROB, SPEECH_END_PROB, SPEECH_START_FRAMES, SILENCE_END_FRAMES)
    }

    fn with_thresholds(start_prob: f32, end_prob: f32, start_frames: usize, end_frames: usize) -> Self {
        let model_bytes = include_bytes!(concat!(env!("CARGO_HOME"), "/registry/src/index.crates.io-1949cf8c6b5b557f/ten-vad-rs-0.1.6/onnx/ten-vad.onnx"));
        let vad = ten_vad_rs::TenVad::new_from_bytes(model_bytes, 16000)
            .expect("failed to create TEN VAD");
        eprintln!("[CLX] vad: TEN VAD neural network initialized (start_prob={}, end_prob={}, start_frames={}, silence_frames={})",
            start_prob, end_prob, start_frames, end_frames);
        Self {
            vad,
            chunk: Vec::new(),
            remainder: Vec::new(),
            in_speech: false,
            speech_frames: 0,
            silence_frames: 0,
            cfg_speech_start_prob: start_prob,
            cfg_speech_end_prob: end_prob,
            cfg_speech_start_frames: start_frames,
            cfg_silence_end_frames: end_frames,
        }
    }

    /// Feed 16kHz f32 samples. Returns completed speech chunks.
    fn feed(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        let mut completed = Vec::new();

        self.remainder.extend_from_slice(samples);
        let all = std::mem::take(&mut self.remainder);

        let mut offset = 0;
        while offset + TEN_VAD_FRAME_SIZE <= all.len() {
            let frame = &all[offset..offset + TEN_VAD_FRAME_SIZE];
            offset += TEN_VAD_FRAME_SIZE;

            // Convert f32 → i16 for TEN VAD
            let frame_i16: Vec<i16> = frame.iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                .collect();
            let prob = self.vad.process_frame(&frame_i16).unwrap_or(0.0);
            let is_speech = prob > self.cfg_speech_start_prob;

            if !self.in_speech {
                if is_speech {
                    self.speech_frames += 1;
                    // Buffer pre-speech frames so we don't cut the start.
                    self.chunk.extend_from_slice(frame);
                    if self.speech_frames >= self.cfg_speech_start_frames {
                        self.in_speech = true;
                        self.silence_frames = 0;
                        eprintln!("[CLX] vad: speech started (prob={:.2})", prob);
                    }
                } else {
                    // Reset speech frame counter and discard buffered pre-speech.
                    self.speech_frames = 0;
                    self.chunk.clear();
                }
            } else {
                // Currently in speech.
                self.chunk.extend_from_slice(frame);

                if prob > self.cfg_speech_end_prob {
                    self.silence_frames = 0;
                } else {
                    self.silence_frames += 1;
                    if self.silence_frames >= self.cfg_silence_end_frames {
                        // Speech ended -- emit chunk.
                        eprintln!("[CLX] vad: speech ended ({:.1}s chunk)",
                            self.chunk.len() as f64 / 16000.0);
                        completed.push(std::mem::take(&mut self.chunk));
                        self.in_speech = false;
                        self.speech_frames = 0;
                        self.silence_frames = 0;
                    }
                }

                // Stream partial: emit every ~3s of continuous speech for IME-like feel.
                if self.chunk.len() >= STREAMING_CHUNK_SAMPLES {
                    completed.push(std::mem::take(&mut self.chunk));
                    // Stay in speech mode — just emit what we have so far.
                    self.silence_frames = 0;
                }
            }
        }

        // Save leftover samples that didn't fill a complete frame.
        if offset < all.len() {
            self.remainder = all[offset..].to_vec();
        }

        completed
    }

    /// Flush any remaining buffered speech (called when listening stops).
    #[allow(dead_code)]
    fn flush(&mut self) -> Option<Vec<f32>> {
        if !self.chunk.is_empty() && self.in_speech {
            self.in_speech = false;
            self.speech_frames = 0;
            self.silence_frames = 0;
            Some(std::mem::take(&mut self.chunk))
        } else {
            self.chunk.clear();
            None
        }
    }
}

// compute_rms removed — replaced by TEN VAD neural network

// ── WAV encoding ─────────────────────────────────────────────────────────────

/// Encode f32 samples as a WAV file (16-bit PCM, mono).
fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes()); // sample rate
    let byte_rate = sample_rate * 2; // 16-bit mono
    buf.extend_from_slice(&byte_rate.to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data sub-chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * 32767.0) as i16;
        buf.extend_from_slice(&pcm.to_le_bytes());
    }

    buf
}

/// Two-phase transcription pipeline:
/// 1. Local Whisper (if available): type rough draft instantly for feedback.
/// 2. Server Whisper + LLM: when the polished result arrives, delete the rough
///    draft (via Backspace) and type the polished text.
///
/// If local Whisper is not available, falls through to server-only.
#[allow(dead_code)]
fn transcribe_and_type(
    samples: &[f32],
    sample_rate: u32,
    server_url: &str,
    platform: &Arc<dyn Platform>,
    engine: &mut Option<SttEngine>,
) {
    // Resample to 16kHz (most mics capture at 44.1/48kHz).
    let samples_16k = resample(samples, sample_rate, 16000);

    // Skip very short chunks. Pad to 1s if between 0.3-1s.
    if samples_16k.len() < 4800 { // < 0.3s — too short, skip
        return;
    }
    let samples_16k = if samples_16k.len() < 16000 {
        let mut padded = samples_16k;
        padded.resize(16000, 0.0);
        padded
    } else {
        samples_16k
    };

    let mut rough_len: usize = 0;

    // Phase 1: instant local transcription.
    if let Some(ref mut stt) = engine {
        match stt.transcribe(&samples_16k) {
            Ok(rough) if !rough.is_empty() => {
                eprintln!("[CLX] voice: local rough: {:?}", rough);
                if crate::modules::agent::is_agent_mode() {
                    // In agent mode, don't type rough — wait for polished.
                } else {
                    platform.type_text(&rough);
                }
                rough_len = rough.chars().count();
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[CLX] voice: STT error: {e}");
            }
        }
        stt.check_pending_upgrade();
    }

    // Phase 2: server transcription (polished). Send 16kHz WAV.
    let polished = send_chunk_to_server(&samples_16k, 16000, server_url);

    if !polished.is_empty() {
        // Delete the rough draft first.
        if rough_len > 0 {
            eprintln!(
                "[CLX] voice: replacing rough draft ({rough_len} chars) with polished text"
            );
            for _ in 0..rough_len {
                platform.key_tap(KeyCode::Backspace);
            }
        }
        if crate::modules::agent::is_agent_mode() {
            eprintln!("[CLX] voice: → agent (polished): {:?}", polished);
            crate::modules::agent::on_voice_transcript(&polished, platform.as_ref());
        } else {
            eprintln!("[CLX] voice: typing polished: {:?}", polished);
            platform.type_text(&polished);
        }
    } else if rough_len == 0 {
        eprintln!("[CLX] voice: no text from server either, skipping");
    }
    // If server returned empty but rough was already typed, keep the rough text.
}

/// Send a speech chunk to the server and return the polished text (or empty string).
#[allow(dead_code)]
fn send_chunk_to_server(samples: &[f32], sample_rate: u32, server_url: &str) -> String {
    let duration_s = samples.len() as f64 / sample_rate as f64;
    eprintln!(
        "[CLX] voice: sending chunk to server ({:.1}s, {} samples)",
        duration_s,
        samples.len()
    );

    let wav_data = encode_wav(samples, sample_rate);

    let boundary = "----CLXVoiceBoundary9f8e7d6c";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"audio\"; filename=\"audio.wav\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
    body.extend_from_slice(&wav_data);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let content_type = format!("multipart/form-data; boundary={boundary}");

    match ureq::post(server_url)
        .set("Content-Type", &content_type)
        .send_bytes(&body)
    {
        Ok(response) => match response.into_string() {
            Ok(body_text) => {
                let mut final_text = String::new();
                for line in body_text.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(text) = extract_json_text(line) {
                        final_text = text;
                    }
                }
                if final_text.is_empty() {
                    eprintln!("[CLX] voice: no text in server response: {body_text}");
                }
                final_text
            }
            Err(e) => {
                eprintln!("[CLX] voice: failed to read response body: {e}");
                String::new()
            }
        },
        Err(e) => {
            eprintln!("[CLX] voice: HTTP request failed: {e}");
            String::new()
        }
    }
}

/// Legacy wrapper kept for backward compatibility (unused after refactor).
/// Encode a speech chunk as WAV, POST to server, and type the transcribed text.
#[allow(dead_code)]
fn send_chunk_and_type(
    samples: &[f32],
    sample_rate: u32,
    server_url: &str,
    platform: &Arc<dyn Platform>,
) {
    let duration_s = samples.len() as f64 / sample_rate as f64;
    eprintln!(
        "[CLX] voice: sending chunk ({:.1}s, {} samples)",
        duration_s,
        samples.len()
    );

    let wav_data = encode_wav(samples, sample_rate);

    // Build multipart body manually since ureq doesn't have built-in multipart.
    let boundary = "----CLXVoiceBoundary9f8e7d6c";
    let mut body = Vec::new();

    // Audio part
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"audio\"; filename=\"audio.wav\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
    body.extend_from_slice(&wav_data);
    body.extend_from_slice(b"\r\n");

    // End boundary
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let content_type = format!("multipart/form-data; boundary={boundary}");

    match ureq::post(server_url)
        .set("Content-Type", &content_type)
        .send_bytes(&body)
    {
        Ok(response) => {
            match response.into_string() {
                Ok(body_text) => {
                    // Parse NDJSON response: each line is a JSON object.
                    // Look for the last "transcribed" or "polished" stage.
                    let mut final_text = String::new();
                    for line in body_text.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        // Simple JSON parsing without serde: look for "text" field.
                        if let Some(text) = extract_json_text(line) {
                            // Prefer "polished" stage, but use any text we find.
                            final_text = text;
                        }
                    }

                    if !final_text.is_empty() {
                        if crate::modules::agent::is_agent_mode() {
                            eprintln!("[CLX] voice: → agent: {:?}", final_text);
                            crate::modules::agent::on_voice_transcript(&final_text, platform.as_ref());
                        } else {
                            eprintln!("[CLX] voice: typing: {:?}", final_text);
                            platform.type_text(&final_text);
                        }
                    } else {
                        eprintln!("[CLX] voice: no text in response: {body_text}");
                    }
                }
                Err(e) => {
                    eprintln!("[CLX] voice: failed to read response body: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("[CLX] voice: HTTP request failed: {e}");
        }
    }
}

/// Extract the "text" field from a JSON line without pulling in serde.
/// Handles: {"stage":"transcribed","text":"hello world"}
fn extract_json_text(json: &str) -> Option<String> {
    // Find "text" key and extract its string value.
    let needle = "\"text\"";
    let idx = json.find(needle)?;
    let after_key = &json[idx + needle.len()..];

    // Skip whitespace and colon.
    let after_colon = after_key.trim_start();
    let after_colon = after_colon.strip_prefix(':')?;
    let after_colon = after_colon.trim_start();

    // Expect a quoted string.
    if !after_colon.starts_with('"') {
        return None;
    }

    // Parse the JSON string value (handling basic escapes).
    let mut result = String::new();
    let mut chars = after_colon[1..].chars();
    loop {
        match chars.next()? {
            '"' => break,
            '\\' => {
                match chars.next()? {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '/' => result.push('/'),
                    'u' => {
                        // Unicode escape: \uXXXX
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            hex.push(chars.next()?);
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            }
                        }
                    }
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            }
            ch => result.push(ch),
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
#[path = "voice_e2e_test.rs"]
mod voice_e2e_test;

#[cfg(test)]
mod tests {
    use super::*;

    // test_compute_rms removed — compute_rms replaced by TEN VAD neural network

    #[test]
    fn test_encode_wav() {
        let samples = vec![0.0f32; 16000]; // 1 second of silence
        let wav = encode_wav(&samples, 16000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        // Total size: 44 header + 32000 data = 32044
        assert_eq!(wav.len(), 44 + 16000 * 2);
    }

    #[test]
    fn test_vad_silence_no_chunks() {
        let mut vad = VadState::new();
        let silence = vec![0.0f32; 16000]; // 1 second
        let chunks = vad.feed(&silence);
        assert!(chunks.is_empty());
    }

    #[test]
    #[ignore = "synthetic 440Hz sine isn't recognized as speech by TEN VAD neural net — needs a real WAV"]
    fn test_vad_speech_then_silence() {
        let mut vad = VadState::new();

        // Generate 1 second of "speech" (loud sine wave).
        let speech: Vec<f32> = (0..16000)
            .map(|i| (i as f32 / 16000.0 * 440.0 * std::f32::consts::TAU).sin() * 0.5)
            .collect();
        let chunks = vad.feed(&speech);
        // Should not complete yet (no silence tail).
        assert!(chunks.is_empty());

        // Now feed 600ms of silence to trigger end.
        let silence = vec![0.0f32; 9600];
        let chunks = vad.feed(&silence);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].len() > 16000); // Should include the speech + some silence
    }

    #[test]
    fn test_extract_json_text() {
        let line = r#"{"stage":"transcribed","text":"hello world"}"#;
        assert_eq!(extract_json_text(line), Some("hello world".to_string()));

        let line = r#"{"stage":"polished","text":"Hello, world!"}"#;
        assert_eq!(extract_json_text(line), Some("Hello, world!".to_string()));

        let line = r#"{"text":"escaped \"quotes\""}"#;
        assert_eq!(extract_json_text(line), Some("escaped \"quotes\"".to_string()));

        assert_eq!(extract_json_text("{}"), None);
        assert_eq!(extract_json_text(r#"{"text":""}"#), None);
    }
}
