/// Voice backend using `otoji listen --plain -` as an external subprocess.
///
/// CLX captures microphone audio via cpal (which has mic permission) and
/// streams a 16 kHz mono WAV to otoji's stdin.  Otoji runs SenseVoice in its
/// own process (~500 MB), keeping the CLX process lightweight.
///
/// JSON-line AsrEvents from otoji stdout are parsed and forwarded to the
/// platform overlay + cursor input.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::platform::Platform;

/// Parsed subset of otoji's AsrEvent (serde tag="type", rename_all="snake_case").
#[derive(Debug)]
enum AsrEvent {
    Partial { text: String },
    Final { text: String },
    Status { message: String },
    Error { message: String },
    PttPartial { text: String },
    PttFinal { text: String },
    PttUpgrade { text: String },
    PttTranslated { text: String, lang: String },
    Other,
}

fn parse_event(line: &str) -> AsrEvent {
    // Minimal JSON parsing without pulling in serde for core.
    let line = line.trim();
    let get = |key: &str| -> Option<String> {
        let needle = format!("\"{}\":\"", key);
        let start = line.find(&needle)? + needle.len();
        let end = line[start..].find('"')? + start;
        Some(line[start..end].replace("\\n", "\n").replace("\\\"", "\"").replace("\\\\", "\\"))
    };
    let ty = match get("type") {
        Some(t) => t,
        None => return AsrEvent::Other,
    };
    match ty.as_str() {
        "partial" => AsrEvent::Partial { text: get("text").unwrap_or_default() },
        "final"   => AsrEvent::Final   { text: get("text").unwrap_or_default() },
        "status"      => AsrEvent::Status      { message: get("message").unwrap_or_default() },
        "error"       => AsrEvent::Error       { message: get("message").unwrap_or_default() },
        "ptt_partial"    => AsrEvent::PttPartial    { text: get("text").unwrap_or_default() },
        "ptt_final"      => AsrEvent::PttFinal      { text: get("text").unwrap_or_default() },
        "ptt_upgrade"    => AsrEvent::PttUpgrade    { text: get("text").unwrap_or_default() },
        "ptt_translated" => AsrEvent::PttTranslated {
            text: get("text").unwrap_or_default(),
            lang: get("lang").unwrap_or_default(),
        },
        _ => AsrEvent::Other,
    }
}

/// Write a streaming WAV header (16 kHz, mono, 16-bit PCM, unknown length).
fn write_wav_header(w: &mut impl Write) -> std::io::Result<()> {
    let sample_rate: u32 = 16000;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * (channels as u32) * (bits_per_sample as u32) / 8;
    let block_align = channels * bits_per_sample / 8;

    w.write_all(b"RIFF")?;
    w.write_all(&0xFFFFFFFEu32.to_le_bytes())?; // streaming: large even size
    w.write_all(b"WAVE")?;

    // fmt chunk
    w.write_all(b"fmt ")?;
    w.write_all(&16u32.to_le_bytes())?;       // chunk size
    w.write_all(&1u16.to_le_bytes())?;        // PCM format
    w.write_all(&channels.to_le_bytes())?;
    w.write_all(&sample_rate.to_le_bytes())?;
    w.write_all(&byte_rate.to_le_bytes())?;
    w.write_all(&block_align.to_le_bytes())?;
    w.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk — use a large even value for streaming (must be multiple of
    // block_align=2 so WAV parsers don't reject it).
    w.write_all(b"data")?;
    w.write_all(&0xFFFFFFFEu32.to_le_bytes())?;
    w.flush()
}

/// Spawn `otoji-tray` once (detached) if not already running. Best-effort.
/// The tray is a separate binary that owns the macOS menu bar item and
/// reads `notes.jsonl` independently — its lifecycle is not tied to the
/// listen child, so a sensevoice crash here doesn't take it down.
fn ensure_tray_running() {
    // Detect existing instance by exact basename. `pgrep -x` matches the
    // process *name* (not the full command), avoiding false positives
    // from anything that happens to mention "otoji-tray".
    let already = Command::new("pgrep")
        .args(["-x", "otoji-tray"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if already {
        return;
    }
    let _ = Command::new("otoji-tray")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}


pub struct OtojiBackend {
    child: Mutex<Option<Child>>,
    reader_stop: Arc<AtomicBool>,
}

impl OtojiBackend {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            reader_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the PID of the running otoji subprocess (for signal sending).
    pub fn pid(&self) -> Option<u32> {
        self.child.lock().unwrap().as_ref().map(|c| c.id())
    }

    /// Check if `otoji` binary is available on PATH.
    pub fn is_available() -> bool {
        Command::new("otoji")
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Start otoji listen subprocess with stdin audio piping. Returns true if started.
    pub fn start(
        &self,
        platform: Arc<dyn Platform>,
        input_active: Arc<AtomicBool>,
        typed_text: Arc<Mutex<String>>,
        ptt: Option<Arc<super::voice_ptt::PttSession>>,
    ) -> bool {
        // Refuse to spawn real subprocesses under `cargo test` or when
        // explicitly disabled. Without this, tests that touch VoiceModule
        // would fork dozens of `otoji listen` (and the tray) processes
        // that survive the test binary, polluting the user's session.
        if cfg!(test) || std::env::var_os("CLX_DISABLE_OTOJI_SPAWN").is_some() {
            return false;
        }

        // Best-effort: launch the otoji menu-bar tray once so the user
        // gets a status icon + recent-notes menu without having to know
        // about a separate `otoji-tray` binary. Idempotent via pgrep, and
        // intentionally outside the early-return guard so a tray that
        // was killed externally gets respawned on the next start() call.
        ensure_tray_running();


        let mut guard = self.child.lock().unwrap();
        if guard.is_some() {
            return true; // already running
        }

        // Use `otoji listen --plain -` to read WAV from stdin instead of
        // opening the mic itself. CLX has mic permission; otoji may not.
        let mut cmd = Command::new("otoji");
        let ctx_path = super::voice_ptt::ptt_context_file_path();
        let mut args: Vec<String> = vec![
            "listen".into(), "--plain".into(), "-".into(),
            // "openai" route goes through OpenAiPolisher which honors the
            // OTOJI_POLISH_BASE_URL / _API_KEY / _MODEL env vars. Default
            // in .env.local points to Cloudflare Workers AI (edge inference,
            // ~200-500ms TTFB). Falls back to Gemini if those env vars are
            // unset thanks to `resolve_polisher`'s "auto" chain.
            "--ptt-polish".into(), "openai".into(),
            // Gemini handles multilingual (en/zh/ja) — "auto" would pick Piper
            // which is English-only and mangles CJK text.
            "--ptt-tts".into(), "gemini".into(),
            "--ptt-context-file".into(), ctx_path,
        ];
        // Translation (Phase 1: env-driven).
        // CLX_TRANSLATE_TO: target language BCP-47 code (e.g. "en"). Empty = off.
        // CLX_TRANSLATE_TTS_SOURCE: "original" or "translated" (default original).
        if let Ok(to) = std::env::var("CLX_TRANSLATE_TO") {
            if !to.is_empty() {
                args.push("--ptt-translate-to".into());
                args.push(to);
            }
        }
        if let Ok(src) = std::env::var("CLX_TRANSLATE_TTS_SOURCE") {
            if !src.is_empty() {
                args.push("--ptt-tts-source".into());
                args.push(src);
            }
        }

        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .env("OTOJI_RELAUNCHED", "1")
            .env("OTOJI_REBUILDING", "1"); // prevent auto-rebuild + exec which breaks pipes

        // NOTE: process_group(0) was disabled — it may interfere with signal
        // delivery from parent to child on macOS. We kill otoji explicitly
        // via its PID instead of the process group.

        let child = match cmd.spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[CLX] voice-otoji: failed to spawn otoji: {}", e);
                return false;
            }
        };

        eprintln!("[CLX] voice-otoji: started otoji pid={}", child.id());

        let mut child = child;
        let stdout = child.stdout.take().expect("otoji stdout");
        let stderr = child.stderr.take().expect("otoji stderr");
        let mut stdin = child.stdin.take().expect("otoji stdin");

        let stop = Arc::clone(&self.reader_stop);
        stop.store(false, Ordering::Relaxed);

        // Clone ptt for the reader thread before mic thread takes ownership.
        let ptt_for_reader = ptt.clone();

        // Stdin writer — capture mic via cpal and stream WAV to otoji
        std::thread::Builder::new()
            .name("otoji-mic".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || {
                    if let Err(e) = write_wav_header(&mut stdin) {
                        eprintln!("[CLX] voice-otoji: failed to write WAV header: {}", e);
                        return;
                    }

                    // Open mic via cpal — CLX binary has mic permission.
                    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
                    let host = cpal::default_host();
                    let device = match host.default_input_device() {
                        Some(d) => d,
                        None => {
                            eprintln!("[CLX] voice-otoji: no default input device");
                            return;
                        }
                    };

                    // Try 16kHz mono first; fall back to device default if unsupported.
                    let default_cfg = match device.default_input_config() {
                        Ok(c) => c,
                        Err(e) => { eprintln!("[CLX] voice-otoji: no default input config: {e}"); return; }
                    };
                    let supports_16k = device.supported_input_configs().map_or(false, |mut it| {
                        it.any(|c| c.channels() == 1
                            && c.min_sample_rate().0 <= 16000
                            && c.max_sample_rate().0 >= 16000
                            && c.sample_format() == cpal::SampleFormat::F32)
                    });
                    let (config, device_sr, device_ch) = if supports_16k {
                        (cpal::StreamConfig { channels: 1, sample_rate: cpal::SampleRate(16000), buffer_size: cpal::BufferSize::Default }, 16000u32, 1usize)
                    } else {
                        let sr = default_cfg.sample_rate().0;
                        let ch = default_cfg.channels() as usize;
                        let fmt = default_cfg.sample_format();
                        eprintln!("[CLX] voice-otoji: 16kHz mono not supported, falling back to {sr}Hz {ch}ch fmt={fmt:?}");
                        (cpal::StreamConfig { channels: default_cfg.channels(), sample_rate: cpal::SampleRate(sr), buffer_size: cpal::BufferSize::Default }, sr, ch)
                    };

                    let stdin_mutex = Arc::new(Mutex::new(stdin));
                    let stdin_for_cb = Arc::clone(&stdin_mutex);
                    let stop_for_cb = Arc::clone(&stop);
                    let ptt_for_cb = ptt.clone();

                    let stream = device.build_input_stream(
                        &config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            if stop_for_cb.load(Ordering::Relaxed) { return; }
                            // Down-mix to mono if needed.
                            let mono: Vec<f32> = if device_ch == 1 {
                                data.to_vec()
                            } else {
                                data.chunks(device_ch)
                                    .map(|c| c.iter().sum::<f32>() / device_ch as f32)
                                    .collect()
                            };
                            // Resample to 16kHz if needed.
                            let mono_16k: Vec<f32> = if device_sr == 16000 {
                                mono
                            } else {
                                let ratio = device_sr as f64 / 16000.0;
                                let out_len = (mono.len() as f64 / ratio) as usize;
                                let mut out = Vec::with_capacity(out_len);
                                for i in 0..out_len {
                                    let src = i as f64 * ratio;
                                    let i0 = src as usize;
                                    let frac = (src - i0 as f64) as f32;
                                    let s0 = mono.get(i0).copied().unwrap_or(0.0);
                                    let s1 = mono.get(i0 + 1).copied().unwrap_or(s0);
                                    out.push(s0 + (s1 - s0) * frac);
                                }
                                out
                            };
                            // Tee 16kHz mono into PTT ring buffer.
                            if let Some(ref p) = ptt_for_cb { p.feed(&mono_16k); }
                            // Convert to i16 PCM and write WAV payload to stdin.
                            let mut buf = Vec::with_capacity(mono_16k.len() * 2);
                            for &sample in &mono_16k {
                                let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                buf.extend_from_slice(&s.to_le_bytes());
                            }
                            if let Ok(mut w) = stdin_for_cb.lock() {
                                let _ = w.write_all(&buf);
                            }
                        },
                        move |err| {
                            eprintln!("[CLX] voice-otoji: cpal error: {}", err);
                        },
                        None,
                    );

                    let stream = match stream {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[CLX] voice-otoji: failed to build input stream: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = stream.play() {
                        eprintln!("[CLX] voice-otoji: failed to start stream: {}", e);
                        return;
                    }

                    eprintln!("[CLX] voice-otoji: mic capture started (16kHz mono → stdin)");

                    // Keep the stream alive until stop is signaled.
                    while !stop.load(Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }

                    drop(stream);
                    // Close stdin to signal EOF to otoji.
                    drop(stdin_mutex);
                    eprintln!("[CLX] voice-otoji: mic capture stopped");
                }
            })
            .ok();

        // Stderr reader — forward otoji logs to CLX stderr
        std::thread::Builder::new()
            .name("otoji-stderr".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if stop.load(Ordering::Relaxed) { break; }
                        if let Ok(line) = line {
                            eprintln!("[otoji] {}", line);
                        }
                    }
                }
            })
            .ok();

        // Stdout reader — parse AsrEvents
        std::thread::Builder::new()
            .name("otoji-reader".into())
            .spawn({
                let stop = Arc::clone(&stop);
                let ptt = ptt_for_reader;
                move || {
                    let reader = BufReader::new(stdout);
                    let mut partial_text = String::new();

                    for line in reader.lines() {
                        if stop.load(Ordering::Relaxed) { break; }
                        let line = match line {
                            Ok(l) => l,
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                            Err(e) => {
                                eprintln!("[CLX] voice-otoji: reader error: {e}");
                                break;
                            }
                        };

                        match parse_event(&line) {
                            AsrEvent::Partial { text } => {
                                partial_text = text.clone();
                                platform.update_voice_subtitle(&text);
                            }
                            AsrEvent::Final { text } => {
                                platform.update_voice_subtitle(&text);

                                if input_active.load(Ordering::Relaxed) {
                                    let prev = typed_text.lock().unwrap().clone();
                                    if !prev.is_empty() {
                                        for _ in prev.chars() {
                                            platform.key_tap(crate::key_code::KeyCode::Backspace);
                                        }
                                    }
                                    platform.type_text(&text);
                                    *typed_text.lock().unwrap() = text.clone();
                                }

                                partial_text.clear();
                            }
                            AsrEvent::PttPartial { text } => {
                                if let Some(ref p) = ptt {
                                    p.on_ptt_partial(&text);
                                }
                            }
                            AsrEvent::PttFinal { text } => {
                                if let Some(ref p) = ptt {
                                    p.on_ptt_final(&text);
                                }
                            }
                            AsrEvent::PttUpgrade { text } => {
                                if let Some(ref p) = ptt {
                                    p.on_ptt_upgrade(&text);
                                }
                            }
                            AsrEvent::PttTranslated { text, lang } => {
                                if let Some(ref p) = ptt {
                                    p.on_ptt_translated(&text, &lang);
                                }
                            }
                            AsrEvent::Status { message } => {
                                platform.update_voice_subtitle(&format!("[{}]", message));
                            }
                            AsrEvent::Error { message } => {
                                eprintln!("[CLX] voice-otoji: error: {}", message);
                                platform.update_voice_subtitle(&format!("ERR: {}", message));
                            }
                            AsrEvent::Other => {}
                        }
                    }
                    eprintln!("[CLX] voice-otoji: reader thread exited");
                }
            })
            .ok();

        *guard = Some(child);
        true
    }

    /// Stop the otoji subprocess and all its children.
    pub fn stop(&self) {
        self.reader_stop.store(true, Ordering::Relaxed);
        let mut guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *guard {
            let pid = child.id();
            eprintln!("[CLX] voice-otoji: stopping otoji pid={}", pid);
            #[cfg(unix)]
            {
                extern "C" { fn kill(pid: i32, sig: i32) -> i32; }
                // Kill specific PID (no longer use process group since we removed
                // cmd.process_group(0) — a negative pid would now target a wrong group).
                unsafe {
                    kill(pid as i32, 15); // SIGTERM
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
                unsafe {
                    kill(pid as i32, 9); // SIGKILL
                }
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        *guard = None;
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}

impl Drop for OtojiBackend {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_partial() {
        let line = r#"{"type":"partial","seg_id":0,"text":"hello"}"#;
        match parse_event(line) {
            AsrEvent::Partial { text } => assert_eq!(text, "hello"),
            other => panic!("expected Partial, got {:?}", other),
        }
    }

    #[test]
    fn parse_final() {
        let line = r#"{"type":"final","seg_id":1,"text":"hello world","words":[]}"#;
        match parse_event(line) {
            AsrEvent::Final { text } => assert_eq!(text, "hello world"),
            other => panic!("expected Final, got {:?}", other),
        }
    }

    #[test]
    fn parse_status() {
        let line = r#"{"type":"status","message":"loading model..."}"#;
        match parse_event(line) {
            AsrEvent::Status { message } => assert_eq!(message, "loading model..."),
            other => panic!("expected Status, got {:?}", other),
        }
    }

    #[test]
    fn wav_header_size() {
        let mut buf = Vec::new();
        write_wav_header(&mut buf).unwrap();
        assert_eq!(buf.len(), 44); // standard WAV header
    }
}
