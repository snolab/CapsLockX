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

/// Tray icon state — one byte per state, sent over Unix datagram socket
/// to `otoji-tray` with sub-millisecond latency. Maps directly to a PNG
/// asset variant (`tray-icon-<name>.png`).
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrayState {
    Idle = 0,           // listen stopped → tray-icon-off
    Starting = 1,       // mic opening → tray-icon-starting (dimmed)
    ListenSilent = 2,   // ready, VAD=off → tray-icon (default)
    ListenVoice = 3,    // VAD=on (voice) → tray-icon-voice (dot UR)
    Decoding = 4,       // ASR running → tray-icon-processing (ring UR)
    Polishing = 5,      // polish LLM in flight → tray-icon-polish
    Saved = 6,          // segment just written → tray-icon-saved (✓ flash)
}

/// Fire-and-forget: send a single byte to otoji-tray's datagram socket so
/// the tray icon flips state with ~microsecond latency. Silently ignored
/// if the tray isn't running. Mirrors the path computation in
/// `otoji::notes::data_dir()`.
pub fn notify_tray(state: TrayState) {
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixDatagram;
        let path = otoji_data_dir().join(".tray.sock");
        let Ok(sock) = UnixDatagram::unbound() else { return; };
        let byte: [u8; 1] = [state as u8];
        let _ = sock.send_to(&byte, &path);
    }
    #[cfg(not(unix))]
    let _ = state;
}

fn otoji_data_dir() -> std::path::PathBuf {
    if let Ok(custom) = std::env::var("OTOJI_DATA_DIR") {
        return std::path::PathBuf::from(custom);
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return std::path::PathBuf::from(home).join("Library/Application Support/otoji");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return std::path::PathBuf::from(appdata).join("otoji");
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return std::path::PathBuf::from(xdg).join("otoji");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home).join(".local/share/otoji");
    }
    std::path::PathBuf::from(".")
}

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
    Vad { active: bool },
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
        "vad" => {
            // active is a bare bool, not a string — look for "active":true/false.
            let active = line.contains("\"active\":true");
            AsrEvent::Vad { active }
        }
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

/// Linear resample from `src_rate` to `dst_rate`. `carry` preserves the
/// fractional sample position across chunks so successive calls stitch
/// seamlessly. Mono in, mono out.
fn resample_linear(src: &[f32], src_rate: u32, dst_rate: u32, carry: &mut f64) -> Vec<f32> {
    if src.is_empty() { return Vec::new(); }
    if src_rate == dst_rate { return src.to_vec(); }
    let ratio = src_rate as f64 / dst_rate as f64;
    let mut out = Vec::with_capacity((src.len() as f64 / ratio) as usize + 1);
    let mut pos = *carry;
    while (pos as usize) + 1 < src.len() {
        let i = pos as usize;
        let frac = (pos - i as f64) as f32;
        let s0 = src[i];
        let s1 = src[i + 1];
        out.push(s0 + (s1 - s0) * frac);
        pos += ratio;
    }
    *carry = pos - src.len() as f64;
    out
}

/// Spawn `otoji-tray` once (detached) if not already running. Best-effort.
/// The tray is a separate binary that owns the macOS menu bar item and
/// reads `notes.jsonl` independently — its lifecycle is not tied to the
/// listen child, so a sensevoice crash here doesn't take it down.
fn ensure_tray_running() {
    // Detect any running tray (both the bundled .app and legacy
    // standalone `otoji-tray` binaries appear in `ps` with the same
    // executable name `otoji` or `otoji-tray`).
    let names = ["otoji-tray", "otoji"];
    for n in names {
        let running = Command::new("pgrep")
            .args(["-x", n])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if running {
            return;
        }
    }
    // Prefer launching the .app bundle — only that has Contents/Resources
    // with the calligraphy tray-icon PNGs next to the binary. The legacy
    // `otoji-tray` on PATH falls back to the mic.fill SF Symbol because
    // NSBundle.mainBundle.resourcePath doesn't reach the bundle assets.
    #[cfg(target_os = "macos")]
    {
        let app_paths = [
            "/Applications/otoji.app",
            &format!(
                "{}/Applications/otoji.app",
                std::env::var("HOME").unwrap_or_default()
            ),
        ];
        for app in app_paths {
            if std::path::Path::new(app).exists() {
                let _ = Command::new("open")
                    .args(["-g", app])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                return;
            }
        }
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
        aec_enabled: bool,
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

        // Stdin writer — capture mic via VPIO (with AEC) or cpal (raw),
        // stream 16kHz mono WAV to otoji.
        let platform_for_mic = Arc::clone(&platform);
        std::thread::Builder::new()
            .name("otoji-mic".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || {
                    if let Err(e) = write_wav_header(&mut stdin) {
                        eprintln!("[CLX] voice-otoji: failed to write WAV header: {}", e);
                        return;
                    }

                    // ── VPIO path (aec_enabled = "always") ──
                    // Use macOS VoiceProcessingIO so speaker bleed (YouTube,
                    // music) is canceled before reaching otoji. Falls back to
                    // cpal if VPIO is unavailable.
                    if aec_enabled {
                        if let Some(aec) = platform_for_mic.start_aec_mic() {
                            let device_sr = aec.sample_rate();
                            eprintln!("[CLX] voice-otoji: using VPIO mic with AEC (native {}Hz → 16kHz)", device_sr);
                            let stdin_mutex = Arc::new(Mutex::new(stdin));
                            let mut leftover = 0.0f64; // resample fractional carry
                            // VPIO post-AEC output is very quiet; same factor
                            // tuned in test-vpio. Clamp before quantising.
                            const VPIO_GAIN: f32 = 30.0;
                            while !stop.load(Ordering::Relaxed) {
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                let chunk = aec.take_samples();
                                if chunk.is_empty() { continue; }
                                if let Some(ref p) = ptt {
                                    let mono_16k = resample_linear(&chunk, device_sr, 16000, &mut leftover);
                                    let amplified: Vec<f32> = mono_16k.iter()
                                        .map(|&s| (s * VPIO_GAIN).clamp(-1.0, 1.0))
                                        .collect();
                                    p.feed(&amplified);
                                    let mut buf = Vec::with_capacity(amplified.len() * 2);
                                    for &s in &amplified {
                                        let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                        buf.extend_from_slice(&v.to_le_bytes());
                                    }
                                    if let Ok(mut w) = stdin_mutex.lock() {
                                        if w.write_all(&buf).is_err() { break; }
                                    }
                                }
                            }
                            aec.stop();
                            return;
                        } else {
                            eprintln!("[CLX] voice-otoji: VPIO unavailable, falling back to cpal (no AEC)");
                        }
                    }

                    // ── cpal path (aec disabled or VPIO unavailable) ──
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
                    // Track the most recent PTT original text so that when a
                    // `ptt_translated` event arrives we can render both lines
                    // in the voice overlay. Updated on ptt_final/ptt_upgrade.
                    let mut last_ptt_original = String::new();

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
                                // Note-mode tray: ASR is mid-decode.
                                if ptt.is_none() {
                                    notify_tray(TrayState::Decoding);
                                }
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
                                // Note-mode tray: segment landed in notes —
                                // flash "saved" then return to listening.
                                // (Final events come from continuous-listen
                                // mode by definition; PTT segments arrive as
                                // PttFinal in a different branch.)
                                if ptt.is_none() {
                                    notify_tray(TrayState::Saved);
                                    std::thread::Builder::new()
                                        .name("tray-saved-flash".into())
                                        .spawn(|| {
                                            std::thread::sleep(std::time::Duration::from_millis(300));
                                            notify_tray(TrayState::ListenSilent);
                                        })
                                        .ok();
                                }

                                // Note-mode translation (independent from PTT).
                                // Runs whenever Final arrives — Final is the
                                // continuous-listen event by construction, so
                                // we don't need to check for ptt presence
                                // (PttSession is always Some in the current
                                // architecture; the historic is_none() guard
                                // suppressed this branch entirely).
                                if let Ok(target) = std::env::var("CLX_NOTE_TRANSLATE_TO") {
                                    if !target.is_empty() && !text.trim().is_empty() {
                                        eprintln!("[CLX] note-translate: spawning for {} chars → {}", text.chars().count(), target);
                                        let platform_tr = std::sync::Arc::clone(&platform);
                                        let original = text.clone();
                                        std::thread::Builder::new()
                                            .name("note-translate".into())
                                            .spawn(move || {
                                                if let Some(tr) = translate_simple(&original, &target) {
                                                    // Skip the bottom-lane update when the
                                                    // translation collapses to the source
                                                    // (already in target language) — adding
                                                    // a duplicate row carries no info and
                                                    // wastes space.
                                                    if tr != original {
                                                        eprintln!("[CLX] note-translate: → {:?}", &tr[..tr.len().min(80)]);
                                                        platform_tr.update_voice_subtitle_translation(&tr);
                                                    } else {
                                                        eprintln!("[CLX] note-translate: same as source, not updating bottom lane");
                                                    }
                                                }
                                            })
                                            .ok();
                                    }
                                }
                            }
                            AsrEvent::PttPartial { text } => {
                                if let Some(ref p) = ptt {
                                    p.on_ptt_partial(&text);
                                }
                            }
                            AsrEvent::PttFinal { text } => {
                                last_ptt_original = text.clone();
                                platform.update_voice_subtitle(&text);
                                // Force VAD off in the overlay — otoji stops
                                // receiving PCM after PTT release, so it can
                                // never observe the silence transition itself
                                // and would leave VAD stuck on.
                                super::wake_word::note_vad(false);
                                platform.update_voice_overlay(&[], false, &[], false);
                                if let Some(ref p) = ptt {
                                    p.on_ptt_final(&text);
                                }
                            }
                            AsrEvent::PttUpgrade { text } => {
                                last_ptt_original = text.clone();
                                platform.update_voice_subtitle(&text);
                                if let Some(ref p) = ptt {
                                    p.on_ptt_upgrade(&text);
                                }
                            }
                            AsrEvent::PttTranslated { text, lang } => {
                                // Translation goes into its own sticky lane
                                // — partial/final updates don't clear it,
                                // so the user can keep reading while the
                                // next utterance starts streaming on top.
                                if !text.trim().is_empty() && text != last_ptt_original {
                                    platform.update_voice_subtitle_translation(&text);
                                }
                                if let Some(ref p) = ptt {
                                    p.on_ptt_translated(&text, &lang);
                                }
                            }
                            AsrEvent::Vad { active } => {
                                eprintln!("[CLX] otoji-reader: vad active={active}");
                                // Forward to the wake-word release watchdog
                                // regardless of which mode is active —
                                // it polls these globals to decide when to
                                // auto-release a wake-triggered PTT segment.
                                super::wake_word::note_vad(active);
                                // Mirror VAD into the voice overlay so the
                                // mic-row indicator reflects speech state.
                                // Without this, the overlay's mic_vad sticks
                                // at whatever value was last pushed — which
                                // looked like "VAD always on" to the user.
                                platform.update_voice_overlay(&[], active, &[], false);
                                if let Some(ref p) = ptt {
                                    p.on_vad(active);
                                } else {
                                    // Note-mode tray: VAD on/off swaps icon
                                    // between voice (●) and silent.
                                    notify_tray(if active {
                                        TrayState::ListenVoice
                                    } else {
                                        TrayState::ListenSilent
                                    });
                                }
                            }
                            AsrEvent::Status { message } => {
                                // Don't push every Status into the visible
                                // subtitle — they're diagnostic noise (model
                                // loading, calibration, noise_floor) that
                                // erases real transcript / translation text
                                // the user is reading. Log to stderr only.
                                eprintln!("[CLX] otoji-status: {}", message);
                                // First Status event = otoji is up and
                                // streaming — flip from Starting to Listen.
                                if ptt.is_none() {
                                    notify_tray(TrayState::ListenSilent);
                                }
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

/// One-shot blocking translation. Walks a fallback chain so a single
/// provider failing (Ollama OOM, cloud rate-limit, network blip, etc.)
/// degrades to the next provider instead of dropping the translation.
/// Returns `None` only when EVERY provider in the chain fails — at that
/// point the overlay keeps the previous sticky translation rather than
/// surfacing an error.
fn translate_simple(text: &str, target_lang: &str) -> Option<String> {
    use crate::llm_client::{stream_chat, LlmConfig, Message};
    let chain = build_translate_chain();
    if chain.is_empty() {
        eprintln!("[CLX] note-translate: chain empty (no keys, no local LLM)");
        return None;
    }
    let sys = format!(
        "You are a translation function. Translate the user's text into {target_lang}. \
         Output ONLY the translated sentence, no explanation, no quotes, no prefix. \
         If the input is already in {target_lang}, output it unchanged."
    );
    let msgs = vec![
        Message { role: "system".into(), content: sys },
        Message { role: "user".into(), content: text.to_string() },
    ];
    for (label, cfg) in chain {
        // Provider may be cooling off after a recent failure — skip silently.
        if provider_in_cooldown(&label) {
            eprintln!("[CLX] note-translate: skip {label} (cooldown)");
            continue;
        }
        let mut buf = String::new();
        match stream_chat(&cfg, &msgs, &mut |t| buf.push_str(t)) {
            Ok(_) => {
                let tr = buf.trim().to_string();
                if !tr.is_empty() {
                    eprintln!("[CLX] note-translate: {label} OK ({} chars)", tr.chars().count());
                    return Some(tr);
                }
                eprintln!("[CLX] note-translate: {label} returned empty, trying next");
            }
            Err(e) => {
                eprintln!("[CLX] note-translate: {label} error: {e}");
                mark_provider_failed(&label);
            }
        }
    }
    None
}

/// Build the (label, LlmConfig) chain to try in order. Honors the user's
/// override (CLX_NOTE_TRANSLATE_PROVIDER) but ALWAYS appends remaining
/// providers as fallback so memory-pressure / rate-limit failures are
/// transparently recovered.
fn build_translate_chain() -> Vec<(String, crate::llm_client::LlmConfig)> {
    use crate::llm_client::LlmConfig;
    let model_pref = std::env::var("CLX_NOTE_TRANSLATE_MODEL").unwrap_or_default();
    let preferred = std::env::var("CLX_NOTE_TRANSLATE_PROVIDER").unwrap_or_default();

    // Source list: every provider with a usable credential / endpoint.
    let mut sources: Vec<(&'static str, String, String)> = Vec::new();
    if let Ok(k) = std::env::var("GEMINI_API_KEY") {
        if !k.is_empty() { sources.push(("gemini", k, model_pref.clone())); }
    }
    if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        if !k.is_empty() { sources.push(("openai", k, model_pref.clone())); }
    }
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.is_empty() { sources.push(("anthropic", k, model_pref.clone())); }
    }
    // Local always available as the last-resort try (llm_client probes
    // localhost:8321 then localhost:11434; failure is just an Err).
    sources.push(("ollama", "ollama".into(), model_pref.clone()));

    // Reorder: user-preferred provider first.
    if !preferred.is_empty() {
        if let Some(idx) = sources.iter().position(|(name, _, _)| name.eq_ignore_ascii_case(&preferred)) {
            let pick = sources.remove(idx);
            sources.insert(0, pick);
        }
    }

    sources
        .into_iter()
        .map(|(name, key, model)| (name.to_string(), LlmConfig::from_key_and_model(&key, &model)))
        .collect()
}

// ── Per-provider cooldown ───────────────────────────────────────────────
//
// When a provider returns an error (Ollama OOM is the common case here)
// stop hitting it for COOLDOWN_SECS so each subsequent translate doesn't
// pay the same load-failure latency. After cooldown expires we try again.

const COOLDOWN_SECS: u64 = 60;

static FAILED_AT: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>>
    = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn mark_provider_failed(label: &str) {
    let mut m = FAILED_AT.lock().unwrap();
    m.insert(label.to_string(), std::time::Instant::now());
}

fn provider_in_cooldown(label: &str) -> bool {
    let m = FAILED_AT.lock().unwrap();
    match m.get(label) {
        Some(t) => t.elapsed().as_secs() < COOLDOWN_SECS,
        None => false,
    }
}

/// Legacy single-pick (kept for callers outside translate_simple). The
/// active translate path now uses `build_translate_chain` for fallback.
#[allow(dead_code)]
/// Pick LLM (key, model) for note-mode translation.
///
/// Priority:
///   1. `CLX_NOTE_TRANSLATE_PROVIDER=ollama` → force local (key="ollama")
///   2. `CLX_NOTE_TRANSLATE_MODEL=<name>` → use that model with auto-detect
///   3. First cloud key in env (Gemini > OpenAI > Anthropic)
///   4. Fallback to local Ollama / MLX (key="ollama" → llm_client probes
///      localhost:8321 then localhost:11434)
///
/// Returning ("ollama", "") trips `LlmConfig::from_key_and_model`'s Ollama
/// branch, which discovers an installed model automatically.
fn pick_best_key() -> (String, String) {
    let model_pref = std::env::var("CLX_NOTE_TRANSLATE_MODEL").unwrap_or_default();
    let provider = std::env::var("CLX_NOTE_TRANSLATE_PROVIDER").unwrap_or_default();
    if provider.eq_ignore_ascii_case("ollama") || provider.eq_ignore_ascii_case("local") {
        return ("ollama".into(), model_pref);
    }
    for var in &["GEMINI_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY"] {
        if let Ok(k) = std::env::var(var) {
            if !k.is_empty() { return (k, model_pref.clone()); }
        }
    }
    // No cloud key — try local. llm_client's Ollama branch returns Err if
    // neither MLX nor Ollama is reachable, surfacing as a clean translate
    // failure (overlay falls back to original-only).
    ("ollama".into(), model_pref)
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
