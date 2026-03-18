/// CLX-Voice -- V key toggles voice listening (toggle / hold modes).
///
/// Pipeline: AudioCapture -> VAD (energy-based) -> two-phase transcription:
///   1. Local Whisper (instant rough draft typed at cursor)
///   2. Server Whisper + LLM (polished text replaces rough draft via Backspace)
///
/// State machine:
///   - on_key_down(V): Idle->Listening (start), Listening->Idle (toggle off)
///   - on_key_up(V):   if held >300ms -> stop (hold mode); else keep listening (toggle mode)
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::audio_capture::AudioCapture;
use crate::key_code::KeyCode;
use crate::local_whisper::LocalWhisper;
use crate::platform::Platform;

const STATE_IDLE: u8 = 0;
const STATE_LISTENING: u8 = 1;
#[allow(dead_code)]
const STATE_STOPPING: u8 = 2;

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
const SPEECH_START_PROB: f32 = 0.5;
/// Speech probability threshold below which silence is counted.
const SPEECH_END_PROB: f32 = 0.3;
/// Consecutive speech frames to trigger speech start.
const SPEECH_START_FRAMES: usize = 2;
/// Consecutive silence frames to end speech (~480ms at 32ms/frame).
const SILENCE_END_FRAMES: usize = 15;
/// Streaming interval: transcribe after this many new samples accumulate.
/// Whisper inference is ~constant time regardless of audio length (~450ms
/// for base model), so the real cadence is limited by inference speed,
/// not this threshold. Setting it low (1600 = 100ms) means "transcribe
/// as fast as the model can keep up" — back-to-back inference during speech.
const STREAMING_CHUNK_SAMPLES: usize = 1_600; // 100ms at 16kHz
/// Maximum chunk duration in samples at 16kHz (25 seconds).
const VAD_MAX_CHUNK_SAMPLES: usize = 400_000;

pub struct VoiceModule {
    state: Arc<AtomicU8>,
    press_time: Mutex<Option<Instant>>,
    platform: Arc<dyn Platform>,
    /// Signal to stop recording (but keep thread alive).
    bg_stop: Arc<AtomicBool>,
    /// Signal to fully terminate the background thread.
    bg_quit: Arc<AtomicBool>,
    /// Handle to the background thread.
    bg_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Signal to wake the bg thread when recording starts.
    bg_wake: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    /// Whether to capture system audio (Shift+V).
    with_system_audio: Arc<AtomicBool>,
}

impl VoiceModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STATE_IDLE)),
            press_time: Mutex::new(None),
            platform,
            bg_stop: Arc::new(AtomicBool::new(false)),
            bg_quit: Arc::new(AtomicBool::new(false)),
            bg_thread: Mutex::new(None),
            bg_wake: Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new())),
            with_system_audio: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        let current = self.state.load(Ordering::Relaxed);
        match current {
            STATE_IDLE => {
                self.state.store(STATE_LISTENING, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = Some(Instant::now());
                // Shift+V = capture system audio too.
                let sys_audio = self.platform.is_key_physically_down(KeyCode::LShift)
                    || self.platform.is_key_physically_down(KeyCode::RShift);
                self.with_system_audio.store(sys_audio, Ordering::Relaxed);
                if sys_audio { eprintln!("[CLX] voice: Shift+V → mic + system audio"); }
                self.start_listening();
                true
            }
            STATE_LISTENING => {
                self.state.store(STATE_IDLE, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = None;
                self.stop_listening();
                true
            }
            _ => true,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        let current = self.state.load(Ordering::Relaxed);
        if current == STATE_LISTENING {
            let held_long = self
                .press_time
                .lock()
                .unwrap()
                .map(|t| t.elapsed().as_millis() >= HOLD_THRESHOLD_MS)
                .unwrap_or(false);

            if held_long {
                // Hold mode: release stops listening.
                self.state.store(STATE_IDLE, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = None;
                self.stop_listening();
            } else {
                // Toggle mode (<300ms): clear press_time to signal toggle mode.
                // This tells stop() (from CLX deactivate) to NOT stop listening.
                *self.press_time.lock().unwrap() = None;
            }
        }
        true
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    /// Called when CLX mode deactivates.
    /// In toggle mode (V was tapped <300ms), keep listening — don't stop.
    /// Only stop if we're in hold mode (press_time is set and recent).
    pub fn stop(&self) {
        // If press_time is None, it was a toggle (already cleared on key_up).
        // Keep listening — user will tap V again to stop.
        let press_time = *self.press_time.lock().unwrap();
        if press_time.is_some() {
            // Hold mode — Space released while V was held. Stop.
            self.state.store(STATE_IDLE, Ordering::Relaxed);
            *self.press_time.lock().unwrap() = None;
            self.stop_listening();
        }
        // Toggle mode: do nothing, keep listening.
    }

    pub fn is_listening(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STATE_LISTENING
    }

    // ── Internal: audio lifecycle ─────────────────────────────────────────────

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
        let platform = Arc::clone(&self.platform);

        let server_url = resolve_server_url();

        let handle = std::thread::Builder::new()
            .name("clx-voice-bg".into())
            .spawn(move || {
                voice_bg_persistent(bg_stop, bg_quit, bg_wake, with_sys, platform, &server_url);
            })
            .expect("failed to spawn voice bg thread");

        *bg = Some(handle);
    }

    /// Ensure the persistent bg thread is running, then wake it to start recording.
    fn start_listening(&self) {
        eprintln!("[CLX] voice: start listening");

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

    fn stop_listening(&self) {
        eprintln!("[CLX] voice: stop listening");
        self.bg_stop.store(true, Ordering::Relaxed);
        // Don't join — thread stays alive for next session.
    }
}

// ── Persistent background thread ──────────────────────────────────────────────

/// Background thread that loads the Whisper model once, then loops:
/// wait for wake signal → record + VAD + transcribe → wait again.
fn voice_bg_persistent(
    bg_stop: Arc<AtomicBool>,
    bg_quit: Arc<AtomicBool>,
    bg_wake: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    with_system_audio: Arc<AtomicBool>,
    platform: Arc<dyn Platform>,
    server_url: &str,
) {
    // Load Whisper model once (the slow part).
    let mut local_whisper = match LocalWhisper::new() {
        Ok(w) => {
            eprintln!("[CLX] voice: local Whisper ready");
            Some(w)
        }
        Err(e) => {
            eprintln!("[CLX] voice: local Whisper unavailable ({e}), server-only");
            None
        }
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
        let ac = match AudioCapture::new() {
            Ok(ac) => ac,
            Err(e) => {
                eprintln!("[CLX] voice: audio capture failed: {e}");
                continue;
            }
        };
        if let Err(e) = ac.start() {
            eprintln!("[CLX] voice: audio start failed: {e}");
            continue;
        }
        let sample_rate = ac.sample_rate();
        let mut vad = VadState::new();

        // Start system audio capture if Shift+V was pressed.
        let sys_capture = if with_system_audio.load(Ordering::Relaxed) {
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

        // Committed text: confirmed transcriptions that won't change.
        let mut committed_text = String::new();
        // Pending audio: only the uncommitted tail gets re-transcribed.
        let mut pending_buf: Vec<f32> = Vec::new();
        let mut pending_text = String::new();
        let mut pending_audio_since_last: usize = 0;

        // Session log.
        let voice_log = std::path::PathBuf::from("/tmp/clx-voice.log");
        let _ = std::fs::write(&voice_log, "");

        // Record loop until bg_stop is set.
        while !bg_stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let samples = ac.take_samples();
            if samples.is_empty() {
                platform.update_voice_overlay(&[], vad.in_speech);
                continue;
            }

            // Mix in system audio if available.
            let samples = if let Some(ref sys) = sys_capture {
                let sys_samples = sys.take_samples();
                if sys_samples.is_empty() {
                    samples
                } else {
                    let max_len = samples.len().max(sys_samples.len());
                    (0..max_len)
                        .map(|i| {
                            let m = samples.get(i).copied().unwrap_or(0.0);
                            let s = sys_samples.get(i).copied().unwrap_or(0.0);
                            (m + s).clamp(-1.0, 1.0)
                        })
                        .collect()
                }
            } else {
                samples
            };

            // Resample to 16kHz BEFORE VAD.
            let samples_16k = resample(&samples, sample_rate, 16000);

            // Compute RMS levels for the overlay.
            let rms_levels: Vec<f32> = samples_16k
                .chunks(TEN_VAD_FRAME_SIZE.max(1))
                .map(|frame| {
                    let sum_sq: f32 = frame.iter().map(|s| s * s).sum();
                    (sum_sq / frame.len() as f32).sqrt()
                })
                .collect();
            platform.update_voice_overlay(&rms_levels, vad.in_speech);

            // Feed to VAD for speech detection.
            let was_in_speech = vad.in_speech;
            let _chunks = vad.feed(&samples_16k);

            if vad.in_speech {
                pending_buf.extend_from_slice(&samples_16k);
                pending_audio_since_last += samples_16k.len();

                // Transcribe the pending (uncommitted) buffer only.
                if pending_audio_since_last >= STREAMING_CHUNK_SAMPLES {
                    let new_text = transcribe_local(&pending_buf, &mut local_whisper);
                    if !new_text.is_empty() {
                        // Diff only against pending_text (not committed).
                        type_diff(&pending_text, &new_text, &platform);
                        pending_text = new_text;

                        // Log full text (committed + pending).
                        use std::io::Write;
                        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
                            let _ = writeln!(f, "{}{}", committed_text, pending_text);
                        }
                    }
                    pending_audio_since_last = 0;
                }

                // Commit pending text when buffer exceeds 3s — freeze it.
                // Shorter window = less instability from Whisper changing its mind.
                if pending_buf.len() > 48_000 { // 3s at 16kHz
                    committed_text.push_str(&pending_text);
                    eprintln!("[CLX] voice: committed {:?}", pending_text);
                    pending_text.clear();
                    pending_buf.clear();
                    pending_audio_since_last = 0;
                }
            } else if was_in_speech {
                // Speech ended — commit remaining pending text.
                if pending_buf.len() > 4800 {
                    let final_text = transcribe_local(&pending_buf, &mut local_whisper);
                    if !final_text.is_empty() {
                        type_diff(&pending_text, &final_text, &platform);
                        pending_text = final_text;
                    }
                }

                committed_text.push_str(&pending_text);

                // Log final text.
                {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
                        let _ = writeln!(f, "{}", committed_text);
                    }
                }

                // Send committed text to server for polishing.
                if !committed_text.is_empty() {
                    // TODO: send full audio to server in background
                    // For now, just log the committed text.
                    eprintln!("[CLX] voice: utterance done: {:?}", committed_text);
                }

                pending_text.clear();
                pending_buf.clear();
                pending_audio_since_last = 0;
                committed_text.clear();
            }
        }

        // Flush remaining.
        if pending_buf.len() > 4800 {
            let new_text = transcribe_local(&pending_buf, &mut local_whisper);
            if !new_text.is_empty() {
                type_diff(&pending_text, &new_text, &platform);
            }
        }

        platform.hide_voice_overlay();

        if let Some(ref sys) = sys_capture { sys.stop(); }
        ac.stop();
        eprintln!("[CLX] voice: session ended, waiting for next activation");
    }

    eprintln!("[CLX] voice: bg thread exiting");
}

// ── Streaming diff helpers ────────────────────────────────────────────────────

/// Transcribe audio locally (Whisper only, no server). Returns text or empty.
fn transcribe_local(samples: &[f32], local_whisper: &mut Option<LocalWhisper>) -> String {
    let samples = if samples.len() < 16000 {
        let mut padded = samples.to_vec();
        padded.resize(16000, 0.0);
        padded
    } else {
        samples.to_vec()
    };

    if let Some(ref mut whisper) = local_whisper {
        match whisper.transcribe(&samples) {
            Ok(text) => {
                // Auto-scale check.
                let mut swap: Option<LocalWhisper> = None;
                whisper.check_pending_upgrade(&mut swap);
                if let Some(new_model) = swap {
                    *local_whisper = Some(new_model);
                }
                text
            }
            Err(e) => {
                eprintln!("[CLX] voice: local Whisper error: {e}");
                String::new()
            }
        }
    } else {
        String::new()
    }
}

/// Type only the difference between old and new text.
/// Finds the longest common prefix, deletes the diverging suffix of old,
/// then types the new suffix. Logs the resulting text to /tmp/clx-voice.log.
fn type_diff(old: &str, new: &str, platform: &Arc<dyn Platform>) {
    // Find common prefix length (in chars).
    let common_chars = old.chars().zip(new.chars()).take_while(|(a, b)| a == b).count();
    let old_chars = old.chars().count();
    let new_chars: Vec<char> = new.chars().collect();

    // Delete the diverging tail of old text.
    let to_delete = old_chars - common_chars;
    if to_delete > 0 {
        for _ in 0..to_delete {
            platform.key_tap(KeyCode::Backspace);
        }
    }

    // Type the new suffix.
    let suffix: String = new_chars[common_chars..].iter().collect();
    if !suffix.is_empty() || to_delete > 0 {
        eprintln!("[CLX] voice: +{:?} (-{} chars)", suffix, to_delete);
        if !suffix.is_empty() {
            platform.type_text(&suffix);
        }
    }

    // Log current state of what's in the input box to /tmp/clx-voice.log
    if !new.is_empty() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("/tmp/clx-voice.log") {
            let _ = writeln!(f, "{}", new);
        }
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
fn transcribe_and_type(
    samples: &[f32],
    sample_rate: u32,
    server_url: &str,
    platform: &Arc<dyn Platform>,
    local_whisper: &mut Option<LocalWhisper>,
) {
    // Resample to 16kHz for Whisper (most mics capture at 44.1/48kHz).
    let samples_16k = resample(samples, sample_rate, 16000);

    // Skip very short chunks. Pad to 1s if between 0.3-1s (Whisper needs ≥1s).
    if samples_16k.len() < 4800 { // < 0.3s — too short, skip
        return;
    }
    let samples_16k = if samples_16k.len() < 16000 {
        // Pad with silence to reach 1s minimum for Whisper.
        let mut padded = samples_16k;
        padded.resize(16000, 0.0);
        padded
    } else {
        samples_16k
    };

    let mut rough_len: usize = 0;

    // Phase 1: instant local transcription.
    if let Some(ref mut whisper) = local_whisper {
        match whisper.transcribe(&samples_16k) {
            Ok(rough) if !rough.is_empty() => {
                eprintln!("[CLX] voice: local rough: {:?}", rough);
                platform.type_text(&rough);
                rough_len = rough.chars().count();
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[CLX] voice: local Whisper error: {e}");
            }
        }
    }
    // Auto-scale: check if a new model is ready from background loading.
    // Done outside the borrow of local_whisper.
    if let Some(ref mut whisper) = local_whisper {
        let mut swap: Option<LocalWhisper> = None;
        whisper.check_pending_upgrade(&mut swap);
        if let Some(new_model) = swap {
            *local_whisper = Some(new_model);
        }
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

    #[test]
    fn test_compute_rms() {
        let silence = vec![0.0f32; 320];
        assert!(compute_rms(&silence) < VAD_RMS_THRESHOLD);

        let loud: Vec<f32> = (0..320).map(|i| (i as f32 / 320.0 * std::f32::consts::TAU).sin() * 0.5).collect();
        assert!(compute_rms(&loud) > VAD_RMS_THRESHOLD);
    }

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
