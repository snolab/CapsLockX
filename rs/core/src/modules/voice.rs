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

/// Default server URL for voice transcription.
const DEFAULT_SERVER_URL: &str = "https://brainstorm.snomiao.com/api/voice-transcribe";

/// Read voice_server URL from config file, then env var, then default.
fn resolve_server_url() -> String {
    // 1. Try config file
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("CapsLockX").join("config.json");
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Some(url) = extract_json_text_for_key(&data, "voice_server") {
                if !url.is_empty() {
                    return url;
                }
            }
        }
    }
    // 2. Try env var
    if let Ok(url) = std::env::var("CLX_VOICE_SERVER") {
        return url;
    }
    // 3. Default
    DEFAULT_SERVER_URL.to_string()
}

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
const SPEECH_START_PROB: f32 = 0.7;
/// Speech probability threshold below which silence is counted.
const SPEECH_END_PROB: f32 = 0.4;
/// Consecutive speech frames to trigger speech start (more = less sensitive).
const SPEECH_START_FRAMES: usize = 8;   // 128ms — filters ambient noise / fan hum
/// Consecutive silence frames to end speech (~480ms at 32ms/frame).
const SILENCE_END_FRAMES: usize = 25;  // 400ms — prevents rapid re-trigger on pauses
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
    /// STT engine preference: "sherpa" or "whisper".
    stt_engine_pref: String,
    /// LLM config for STT correction.
    llm_api_key: String,
    llm_model: String,
    stt_correction: bool,
}

impl VoiceModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self::with_stt_engine(platform, "sherpa".to_string())
    }

    pub fn with_stt_engine(platform: Arc<dyn Platform>, stt_engine: String) -> Self {
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
            stt_engine_pref: stt_engine,
            llm_api_key: String::new(),
            llm_model: String::new(),
            stt_correction: false,
        }
    }

    pub fn with_llm_config(mut self, api_key: String, model: String, correction: bool) -> Self {
        self.llm_api_key = api_key;
        self.llm_model = model;
        self.stt_correction = correction;
        self
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        *self.press_time.lock().unwrap() = Some(Instant::now());

        // Default = mic only. Shift+V = dual capture (mic+system).
        let shift_held = self.platform.is_key_physically_down(KeyCode::LShift)
            || self.platform.is_key_physically_down(KeyCode::RShift);
        self.with_system_audio.store(shift_held, Ordering::Relaxed);

        // Audio pipeline starts immediately so we capture from the beginning.
        self.ensure_pipeline_running();

        // After 300ms, if V is still held, activate input mode (typing at cursor).
        // If released before 300ms, it's a click → note mode (no typing).
        self.v_held.store(true, Ordering::Relaxed);
        let input_flag = Arc::clone(&self.input_active);
        let flush_flag = Arc::clone(&self.flush_pending);
        let v_held = Arc::clone(&self.v_held);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(HOLD_THRESHOLD_MS as u64));
            if v_held.load(Ordering::Relaxed) {
                // Flush old pending audio so input starts fresh from this moment.
                flush_flag.store(true, Ordering::Relaxed);
                input_flag.store(true, Ordering::Relaxed);
                eprintln!("[CLX] voice: held >300ms → input mode activated (buffer flushed)");
            }
        });
        true
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        self.v_held.store(false, Ordering::Relaxed);

        let held_long = self
            .press_time
            .lock()
            .unwrap()
            .map(|t| t.elapsed().as_millis() >= HOLD_THRESHOLD_MS)
            .unwrap_or(false);

        if held_long {
            // Hold release (>300ms): this was voice INPUT mode.
            // Input was active during the hold — now stop it.
            self.input_active.store(false, Ordering::Relaxed);
            eprintln!("[CLX] voice: hold release → input done");

            // If note mode isn't active, stop the pipeline entirely.
            if !self.note_active.load(Ordering::Relaxed) {
                self.stop_pipeline();
            }
        } else {
            // Click (<300ms): toggle voice NOTE mode.
            if self.note_active.load(Ordering::Relaxed) {
                // Note was running → stop it.
                self.note_active.store(false, Ordering::Relaxed);
                eprintln!("[CLX] voice: click → note stopped");
                self.stop_pipeline();
            } else {
                // Note wasn't running → start it.
                self.note_active.store(true, Ordering::Relaxed);
                eprintln!("[CLX] voice: click → note started (recording to file)");
                // Pipeline already started on key_down.
            }
        }

        *self.press_time.lock().unwrap() = None;
        true
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    /// Called when CLX mode deactivates.
    /// Stop voice input (hold mode), but keep voice note running.
    pub fn stop(&self) {
        let was_input = self.input_active.swap(false, Ordering::Relaxed);
        // Only stop pipeline if input was active AND note is not running.
        if was_input && !self.note_active.load(Ordering::Relaxed) {
            self.stop_pipeline();
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
        self.ensure_pipeline_running();
    }

    /// Spawn the background thread eagerly so the Whisper model loads at startup.
    pub fn preload(&self) {
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
        let platform = Arc::clone(&self.platform);

        let server_url = resolve_server_url();
        let stt_engine = self.stt_engine_pref.clone();
        let llm_key = self.llm_api_key.clone();
        let llm_model = self.llm_model.clone();
        let stt_correction = self.stt_correction;

        let handle = std::thread::Builder::new()
            .name("clx-voice-bg".into())
            .spawn(move || {
                voice_bg_persistent(bg_stop, bg_quit, bg_wake, with_sys, note_active, input_active, flush_pending, platform, &server_url, &stt_engine, &llm_key, &llm_model, stt_correction);
            })
            .expect("failed to spawn voice bg thread");

        *bg = Some(handle);
    }

    /// Ensure the audio pipeline bg thread is running and capturing.
    fn ensure_pipeline_running(&self) {
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
    platform: Arc<dyn Platform>,
    _server_url: &str,
    stt_engine_pref: &str,
    llm_api_key: &str,
    llm_model: &str,
    stt_correction: bool,
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
        let mut vad = VadState::new();

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
                )
            })
            .expect("failed to spawn STT worker thread");

        // NLMS adaptive echo canceller: 100ms filter at 16kHz = 1600 taps.
        let mut nlms = NlmsEchoCancel::new(1600, 0.3);

        // ── System audio VAD (audio loop only runs VAD, worker handles transcription) ──
        let mut sys_vad = if sys_capture.is_some() { Some(VadState::new()) } else { None };

        // Pre-send accumulators: batch audio before sending to the STT worker.
        // Mic: 200ms batches for low latency.  Sys: 500ms batches — background audio
        // doesn't need to be as responsive, and this keeps mic getting priority.
        const MIC_SEND_THRESHOLD: usize = 3_200;  // 200ms at 16kHz
        const SYS_SEND_THRESHOLD: usize = 8_000;  // 500ms at 16kHz
        let mut mic_send_buf: Vec<f32> = Vec::new();
        let mut sys_send_buf: Vec<f32> = Vec::new();

        let note_start_time = std::time::Instant::now();

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
        while !bg_stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
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
                vad = VadState::new();
                // Tell the worker to flush its transcription state and buffers.
                let _ = stt_tx.try_send(SttCommand::Flush);
                eprintln!("[CLX] voice: flushed audio buffers for fresh input session");
            }

            // Resample both streams to 16kHz.
            let mic_16k_raw = resample(&samples, sample_rate, 16000);
            let sys_16k = if !sys_samples.is_empty() {
                resample(&sys_samples, 48000, 16000)
            } else {
                Vec::new()
            };

            // NLMS adaptive echo cancellation on raw (pre-gain) signal.
            let mic_cancelled = if !sys_16k.is_empty() {
                nlms.process_buf(&mic_16k_raw, &sys_16k)
            } else {
                mic_16k_raw
            };

            // Apply gain + noise gate AFTER NLMS (so NLMS works on clean signal).
            let mic_16k: Vec<f32> = if use_aec {
                const AEC_GAIN: f32 = 40.0;
                const NOISE_GATE: f32 = 0.002;
                mic_cancelled.iter()
                    .map(|&s| if s.abs() < NOISE_GATE { 0.0 } else { (s * AEC_GAIN).clamp(-1.0, 1.0) })
                    .collect()
            } else {
                mic_cancelled
            };

            let samples_16k = &mic_16k;

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

        // ── Session ending: tell worker to flush remaining ────────────────
        // The worker's accumulated buffer will be finalized when it receives Quit.

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
) -> SttWorkerResult {
    // Take ownership of engine/corrector from the shared locks for the session.
    // This avoids holding the mutex during transcription.
    let mut engine = engine_lock.lock().unwrap().take();
    let mut corrector = corrector_lock.lock().unwrap().take();
    // ── Mic track state ──
    let mut mic_pending_buf: Vec<f32> = Vec::new();
    let mut mic_pending_since: usize = 0;
    let mut mic_committed = String::new();
    let mut mic_whisper_pending = String::new();
    let mut mic_typed_pending = String::new();
    let mut mic_prev_whisper = String::new();
    let mut mic_stable: usize = 0;

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
            };
            match source {
                Some(SttSource::Mic) => mic_cmds.push(cmd),
                Some(SttSource::Sys) => sys_cmds.push(cmd),
                None => {} // Flush/Quit already handled
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
                            &mut mic_stable,
                            &sys_committed, &sys_whisper_pending,
                            &mut note_srt, &mut srt_index,
                        );
                        mic_pending_buf.clear();
                        mic_pending_since = 0;
                    }

                    // Transcribe when enough new samples have accumulated.
                    if mic_pending_since >= STREAMING_CHUNK_SAMPLES {
                        let result = process_mic_streaming(
                            &mic_pending_buf, timestamp_secs,
                            &mut engine, &mut corrector, platform,
                            note_active, input_active, has_sys,
                            &mut mic_committed, &mut mic_whisper_pending,
                            &mut mic_typed_pending, &mut mic_prev_whisper,
                            &mut mic_stable,
                            &sys_committed, &sys_whisper_pending,
                            &mut note_srt, &mut srt_index,
                        );
                        mic_pending_since = 0;
                        match result {
                            Some(true) => {
                                // Committed — start fresh.
                                mic_pending_buf.clear();
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
                    process_mic_speech_end(
                        &mic_pending_buf, timestamp_secs,
                        &mut engine, &mut corrector, platform,
                        note_active, input_active, has_sys,
                        &mut mic_committed, &mut mic_whisper_pending,
                        &mut mic_typed_pending, &mut mic_prev_whisper,
                        &mut mic_stable,
                        &sys_committed, &sys_whisper_pending,
                        &mut note_srt, &mut srt_index,
                    );
                    mic_pending_buf.clear();
                    mic_pending_since = 0;
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
                    );
                    sys_pending_buf.clear();
                    sys_pending_since = 0;
                    sys_subtitle_dirty = true;
                }
                _ => {}
            }
        }
        // Push subtitle update when sys track changed but mic didn't trigger one.
        if sys_subtitle_dirty {
            let mic_raw = format!("{}{}", mic_committed, mic_whisper_pending);
            let sys_raw = format!("{}{}", sys_committed, sys_whisper_pending);
            let mic_display = last_n_chars(&mic_raw, 200);
            let sys_display = last_n_chars(&sys_raw, 200);
            let mut parts = Vec::new();
            if !mic_display.is_empty() { parts.push(format!("\u{1F3A4} {}", mic_display)); }
            if !sys_display.is_empty() { parts.push(format!("\u{1F50A} {}", sys_display)); }
            if !parts.is_empty() {
                let subtitle = parts.join("\n");
                let preview: String = subtitle.chars().take(60).collect();
                eprintln!("[CLX] stt-worker: pushing sys subtitle ({} chars): {:?}", subtitle.chars().count(), preview);
                platform.update_voice_subtitle(&subtitle);
            }
        }

        if quit {
            // Flush remaining mic audio on session end.
            if mic_pending_buf.len() > 4800 {
                let mut final_text = transcribe_local(&mic_pending_buf, &mut engine);
                if !final_text.is_empty() {
                    if let Some(ref mut c) = corrector {
                        final_text = c.correct(&final_text);
                    }
                    if input_active.load(Ordering::Relaxed) && mic_typed_pending != final_text {
                        type_replace(&mic_typed_pending, &final_text, platform);
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
    sys_committed: &str,
    sys_whisper_pending: &str,
    note_srt: &mut String,
    srt_index: &mut usize,
) -> Option<bool> {
    let new_text = transcribe_local(pending_buf, engine);
    if new_text.is_empty() {
        return None; // nospeech — caller should trim context window
    }
    *mic_whisper_pending = new_text.clone();

    // Voice Input: type at cursor only if input mode is active.
    let is_input = input_active.load(Ordering::Relaxed);
    if is_input {
        type_diff(mic_typed_pending, &new_text, platform);
        if new_text.starts_with(mic_typed_pending.as_str()) {
            *mic_typed_pending = new_text.clone();
        }
    }

    // Update overlay subtitle with speaker labels.
    let display_pending = if is_input { mic_typed_pending.as_str() } else { mic_whisper_pending.as_str() };
    let mic_raw = format!("{}{}", mic_committed, display_pending);
    let subtitle = if has_sys {
        let sys_raw = format!("{}{}", sys_committed, sys_whisper_pending);
        let mic_text = last_n_chars(&mic_raw, 200);
        let sys_text = last_n_chars(&sys_raw, 200);
        let mut parts = Vec::new();
        if !mic_text.is_empty() { parts.push(format!("\u{1F3A4} {}", mic_text)); }
        if !sys_text.is_empty() { parts.push(format!("\u{1F50A} {}", sys_text)); }
        parts.join("\n")
    } else {
        last_n_chars(&mic_raw, 200).to_string()
    };
    platform.update_voice_subtitle(&subtitle);

    // Log.
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
            let _ = writeln!(f, "{}", last_n_chars(&mic_raw, 200));
        }
    }

    // Stability tracking.
    if *mic_whisper_pending == *mic_prev_whisper && !mic_whisper_pending.is_empty() {
        *mic_stable += 1;
    } else {
        *mic_stable = 0;
    }
    *mic_prev_whisper = mic_whisper_pending.clone();

    // Commit when stable or forced at 5s.
    // Lower threshold (1s) vs sys (2s) since mic context window is only 500ms.
    let should_commit = (*mic_stable >= 2 && pending_buf.len() > 16_000)
        || pending_buf.len() > 80_000;
    if should_commit {
        // Apply LLM correction if enabled.
        if let Some(ref mut c) = corrector {
            let corrected = c.correct(mic_whisper_pending);
            if corrected != *mic_whisper_pending {
                if input_active.load(Ordering::Relaxed) {
                    type_replace(mic_typed_pending, &corrected, platform);
                }
                *mic_whisper_pending = corrected;
            }
        }
        // At commit: replace typed text with stable version (input mode only).
        if input_active.load(Ordering::Relaxed) && *mic_typed_pending != *mic_whisper_pending {
            type_replace(mic_typed_pending, mic_whisper_pending, platform);
        }
        if !mic_committed.is_empty() && !mic_committed.ends_with(' ') && !mic_committed.ends_with('\n') {
            mic_committed.push(' ');
        }
        mic_committed.push_str(mic_whisper_pending);
        // Add SRT entry for voice note.
        if note_active.load(Ordering::Relaxed) && !mic_whisper_pending.is_empty() {
            *srt_index += 1;
            let start_srt = format_srt_time(timestamp_secs - (pending_buf.len() as f64 / 16000.0).max(0.0));
            let end_srt = format_srt_time(timestamp_secs);
            let label = if has_sys { "\u{1F3A4} " } else { "" };
            note_srt.push_str(&format!("{}\n{} --> {}\n{}{}\n\n", srt_index, start_srt, end_srt, label, mic_whisper_pending));
        }
        eprintln!("[CLX] stt-worker: mic committed {:?} (stable={})", mic_whisper_pending, mic_stable);
        mic_whisper_pending.clear();
        mic_typed_pending.clear();
        mic_prev_whisper.clear();
        *mic_stable = 0;
        return Some(true);
    }
    Some(false) // speech detected but not yet stable
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
    _sys_committed: &str,
    _sys_whisper_pending: &str,
    note_srt: &mut String,
    srt_index: &mut usize,
) {
    // Final transcription.
    if pending_buf.len() > 4800 {
        let mut final_text = transcribe_local(pending_buf, engine);
        if !final_text.is_empty() {
            // Apply LLM correction at utterance end.
            if let Some(ref mut c) = corrector {
                final_text = c.correct(&final_text);
            }
            // At speech end, accept rewrites (input mode only).
            if input_active.load(Ordering::Relaxed) && *mic_typed_pending != final_text {
                type_replace(mic_typed_pending, &final_text, platform);
            }
            *mic_whisper_pending = final_text;
        }
    }

    if !mic_committed.is_empty() && !mic_committed.ends_with(' ') && !mic_committed.ends_with('\n') {
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
) -> bool {
    let text = transcribe_local(pending_buf, engine);
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
) {
    if pending_buf.len() > 4800 {
        let text = transcribe_local(pending_buf, engine);
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
    // Simple UTC timestamp without chrono crate.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Approximate date (good enough for filenames).
    let y = 1970 + days / 365;
    let doy = days % 365;
    let mon = doy / 30 + 1;
    let day = doy % 30 + 1;
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

/// Transcribe audio locally (sherpa or whisper). Returns text or empty.
fn transcribe_local(samples: &[f32], engine: &mut Option<SttEngine>) -> String {
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
        match stt.transcribe(samples) {
            Ok(text) => {
                stt.check_pending_upgrade();
                text
            }
            Err(e) => {
                eprintln!("[CLX] voice: STT error: {e}");
                String::new()
            }
        }
    } else {
        String::new()
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

// ── NLMS Adaptive Echo Canceller ─────────────────────────────────────────────
/// Cross-platform echo cancellation using Normalized Least Mean Squares filter.
/// Given the system audio (reference) and mic audio, adaptively learns the
/// acoustic path and subtracts the predicted echo from the mic signal.
struct NlmsEchoCancel {
    w: Vec<f32>,       // adaptive filter coefficients
    x_buf: Vec<f32>,   // reference signal ring buffer
    mu: f32,           // step size (learning rate)
    pos: usize,        // ring buffer position
}

impl NlmsEchoCancel {
    fn new(filter_len: usize, mu: f32) -> Self {
        Self {
            w: vec![0.0; filter_len],
            x_buf: vec![0.0; filter_len],
            mu,
            pos: 0,
        }
    }

    /// Process one sample: returns echo-cancelled mic sample.
    /// `mic` = microphone input, `ref_sample` = system audio (what speakers play).
    fn process(&mut self, mic: f32, ref_sample: f32) -> f32 {
        let n = self.w.len();

        // Insert reference sample into ring buffer.
        self.x_buf[self.pos] = ref_sample;

        // Predict echo: y_hat = sum(w[i] * x_buf[(pos-i) mod n])
        let mut y_hat: f32 = 0.0;
        for i in 0..n {
            let idx = (self.pos + n - i) % n;
            y_hat += self.w[i] * self.x_buf[idx];
        }

        // Error = mic - predicted echo (this is the cleaned signal).
        let error = mic - y_hat;

        // Compute reference signal power for normalization.
        let power: f32 = self.x_buf.iter().map(|x| x * x).sum::<f32>() + 1e-8;

        // Update filter coefficients: w += mu * error * x / power
        let step = self.mu * error / power;
        for i in 0..n {
            let idx = (self.pos + n - i) % n;
            self.w[i] += step * self.x_buf[idx];
        }

        self.pos = (self.pos + 1) % n;
        error
    }

    /// Process a buffer of samples.
    fn process_buf(&mut self, mic: &[f32], reference: &[f32]) -> Vec<f32> {
        let len = mic.len().min(reference.len());
        (0..len).map(|i| self.process(mic[i], reference[i])).collect()
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
}

/// TEN VAD frame size: 256 samples at 16kHz (16ms).
const TEN_VAD_FRAME_SIZE: usize = 256;

impl VadState {
    fn new() -> Self {
        // Embed the 308KB TEN VAD ONNX model in the binary.
        // Embed the 308KB TEN VAD ONNX model directly in the binary.
        let model_bytes = include_bytes!(concat!(env!("CARGO_HOME"), "/registry/src/index.crates.io-1949cf8c6b5b557f/ten-vad-rs-0.1.6/onnx/ten-vad.onnx"));
        let vad = ten_vad_rs::TenVad::new_from_bytes(model_bytes, 16000)
            .expect("failed to create TEN VAD");
        eprintln!("[CLX] vad: TEN VAD neural network initialized");
        Self {
            vad,
            chunk: Vec::new(),
            remainder: Vec::new(),
            in_speech: false,
            speech_frames: 0,
            silence_frames: 0,
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
            let is_speech = prob > SPEECH_START_PROB;

            if !self.in_speech {
                if is_speech {
                    self.speech_frames += 1;
                    // Buffer pre-speech frames so we don't cut the start.
                    self.chunk.extend_from_slice(frame);
                    if self.speech_frames >= SPEECH_START_FRAMES {
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

                if prob > SPEECH_END_PROB {
                    self.silence_frames = 0;
                } else {
                    self.silence_frames += 1;
                    if self.silence_frames >= SILENCE_END_FRAMES {
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
                platform.type_text(&rough);
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
        eprintln!("[CLX] voice: typing polished: {:?}", polished);
        platform.type_text(&polished);
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
                        eprintln!("[CLX] voice: typing: {:?}", final_text);
                        platform.type_text(&final_text);
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
