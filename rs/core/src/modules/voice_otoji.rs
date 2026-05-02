/// Voice backend using `otoji listen --plain` as an external subprocess.
///
/// Otoji opens the microphone itself (and therefore holds the mic permission).
/// CLX only reads JSON-line AsrEvents from otoji's stdout and forwards them
/// to the platform overlay + cursor input.

use std::io::{BufRead, BufReader};
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

/// Spawn `otoji-tray` once (detached) if not already running. Best-effort.
/// The tray is a separate binary that owns the macOS menu bar item and
/// reads `notes.jsonl` independently — its lifecycle is not tied to the
/// listen child, so a sensevoice crash here doesn't take it down.
fn ensure_tray_running() {
    // Detect a running tray *specifically* — the bare process name `otoji`
    // is also used by `otoji listen`, `otoji kws`, etc., so a `pgrep -x
    // otoji` match would mean "any otoji subprocess is alive" and skip
    // launching the tray when wake-word is enabled. Match either:
    //   - the legacy `otoji-tray` binary (basename), or
    //   - the .app bundle whose command line contains
    //     `.app/Contents/MacOS/otoji`.
    let patterns = [
        // -x: full basename match (catches the standalone binary).
        ("-x", "otoji-tray"),
        // -f + regex: full command-line match for the .app bundle path.
        ("-f", r"\.app/Contents/MacOS/otoji"),
    ];
    for (flag, pat) in patterns {
        let running = Command::new("pgrep")
            .args([flag, pat])
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
    /// TCP control socket address (used on Windows instead of Unix signals).
    control_addr: Mutex<Option<String>>,
}

impl OtojiBackend {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            reader_stop: Arc::new(AtomicBool::new(false)),
            control_addr: Mutex::new(None),
        }
    }

    /// Get the PID of the running otoji subprocess (for signal sending).
    pub fn pid(&self) -> Option<u32> {
        self.child.lock().unwrap().as_ref().map(|c| c.id())
    }

    /// Get the control socket address (for TCP-based PTT control on Windows).
    pub fn control_addr(&self) -> Option<String> {
        self.control_addr.lock().unwrap().clone()
    }

    /// Send a control command via TCP to the otoji control socket.
    pub fn send_control(&self, cmd: &str) -> bool {
        if let Some(addr) = self.control_addr() {
            let socket_addr: std::net::SocketAddr = match addr.parse() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("[CLX] voice-otoji: invalid control address '{}': {}", addr, e);
                    return false;
                }
            };
            match std::net::TcpStream::connect_timeout(
                &socket_addr,
                std::time::Duration::from_millis(500),
            ) {
                Ok(mut stream) => {
                    use std::io::Write;
                    let msg = format!("{}\n", cmd);
                    if stream.write_all(msg.as_bytes()).is_ok() {
                        return true;
                    }
                    eprintln!("[CLX] voice-otoji: control write failed");
                    false
                }
                Err(e) => {
                    eprintln!("[CLX] voice-otoji: control connect failed: {}", e);
                    false
                }
            }
        } else {
            false
        }
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

    /// Start otoji listen subprocess. Returns true if started.
    /// Otoji opens the microphone itself — CLX only reads its stdout JSON events.
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

        let mut cmd = Command::new("otoji");
        let ctx_path = super::voice_ptt::ptt_context_file_path();
        let mut args: Vec<String> = vec![
            "listen".into(), "--plain".into(),
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

        // On Windows, use TCP control socket instead of Unix signals for PTT.
        #[cfg(target_os = "windows")]
        let control_port = {
            // Pick a random ephemeral port.
            let listener = std::net::TcpListener::bind("127.0.0.1:0").ok();
            let port = listener.as_ref().map(|l| l.local_addr().unwrap().port()).unwrap_or(18080);
            drop(listener);
            let addr = format!("127.0.0.1:{}", port);
            args.push("--ptt-control-socket".into());
            args.push(addr.clone());
            addr
        };

        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .env("OTOJI_RELAUNCHED", "1")
            .env("OTOJI_REBUILDING", "1"); // prevent auto-rebuild + exec which breaks pipes

        // On Windows, prevent a visible CMD window from flashing.
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

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

        // Store the control socket address on Windows.
        #[cfg(target_os = "windows")]
        {
            *self.control_addr.lock().unwrap() = Some(control_port);
            eprintln!("[CLX] voice-otoji: control socket ready");
        }

        let mut child = child;
        let stdout = child.stdout.take().expect("otoji stdout");
        let stderr = child.stderr.take().expect("otoji stderr");

        let stop = Arc::clone(&self.reader_stop);
        stop.store(false, Ordering::Relaxed);

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

}
