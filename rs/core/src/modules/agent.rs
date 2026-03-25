/// CLX Agent — LLM-driven computer control via voice input.
///
/// Keybindings:
///   CLX+M       — Listen to mic → STT → send to agent → execute → loop
///   CLX+Shift+M — Kill running agent
///   ESC         — Kill running agent (when active)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::io::{BufRead, BufReader};

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

/// Record a short voice prompt via macOS mic and run STT.
/// Returns the transcribed text, or None if no speech detected.
fn record_voice_prompt() -> Option<String> {
    // Use sox/rec to capture 5s of audio, then run whisper/sensevoice via clx.
    // Simplest approach: use `say` for feedback + `rec` for recording.

    // Check if sox is available for recording.
    let has_sox = std::process::Command::new("which").arg("rec")
        .output().map(|o| o.status.success()).unwrap_or(false);

    let audio_path = std::env::current_exe().ok()
        .and_then(|e| e.parent().map(|p| p.join("tmp").join("agent-voice.wav")))
        .unwrap_or_else(|| "tmp/agent-voice.wav".into());
    let _ = std::fs::create_dir_all(audio_path.parent().unwrap_or(std::path::Path::new(".")));

    if has_sox {
        eprintln!("[CLX] agent: recording 5s via sox...");
        // Record 5 seconds, mono, 16kHz (optimal for whisper/sensevoice).
        let status = std::process::Command::new("rec")
            .args([
                audio_path.to_str().unwrap_or("tmp/agent-voice.wav"),
                "rate", "16000", "channels", "1",
                "trim", "0", "5",  // 5 seconds max
            ])
            .stderr(std::process::Stdio::null())
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return run_stt_on_file(&audio_path);
        }
    }

    // Fallback: use osascript to ask for text input (old behavior).
    eprintln!("[CLX] agent: sox not available, falling back to text input");
    let output = std::process::Command::new("osascript")
        .args(["-e", r#"display dialog "Agent command:" default answer "" with title "CLX Agent""#])
        .output().ok()?;
    if !output.status.success() { return None; }
    let text = String::from_utf8_lossy(&output.stdout);
    // osascript returns "button returned:OK, text returned:THE_TEXT"
    text.split("text returned:").nth(1).map(|s| s.trim().to_string())
}

/// Run STT on an audio file using the clx binary's SenseVoice/Whisper engine.
fn run_stt_on_file(path: &std::path::Path) -> Option<String> {
    // Use the clx binary to run STT (it has the ONNX runtime loaded).
    let clx_bin = std::env::current_exe().ok()?;
    // For now, use a simpler approach: call whisper CLI if available.
    let has_whisper = std::process::Command::new("which").arg("whisper")
        .output().map(|o| o.status.success()).unwrap_or(false);

    if has_whisper {
        let output = std::process::Command::new("whisper")
            .args(["--model", "tiny", "--language", "auto", "--output_format", "txt",
                   path.to_str().unwrap_or("")])
            .output().ok()?;
        if output.status.success() {
            let txt_path = path.with_extension("txt");
            return std::fs::read_to_string(&txt_path).ok();
        }
    }

    // Fallback: read the file size — if > 1KB, something was recorded but we can't transcribe.
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > 1024 {
        eprintln!("[CLX] agent: audio recorded ({} bytes) but no STT engine available", meta.len());
        eprintln!("[CLX] agent: install sox (`brew install sox`) and whisper (`pip install openai-whisper`)");
    }
    None
}

pub struct AgentModule {
    platform: Arc<dyn Platform>,
    running: AtomicBool,
    child: Mutex<Option<std::process::Child>>,
}

/// Path to the live overlay file (shared with clx-agent).
/// Uses ./tmp/ relative to the binary location.
fn live_log_path() -> std::path::PathBuf {
    std::env::current_exe().ok()
        .and_then(|e| e.parent().map(|p| p.join("tmp").join("agent-live.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("tmp/agent-live.log"))
}

impl AgentModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        let s = Self {
            platform,
            running: AtomicBool::new(false),
            child: Mutex::new(None),
        };
        // Start file watcher for live overlay — shows agent log even
        // when agent is launched from CLI, not just CLX+M.
        s.start_live_watcher();
        s
    }

    /// Watch agent-live.log and show changes in the brainstorm overlay.
    fn start_live_watcher(&self) {
        let platform = Arc::clone(&self.platform);
        std::thread::Builder::new()
            .name("clx-agent-live-watcher".into())
            .spawn(move || {
                let path = live_log_path();
                let mut last_size: u64 = 0;
                let mut last_content = String::new();
                // Only show content written AFTER clx started.
                let start_time = std::time::SystemTime::now();

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    let meta = match std::fs::metadata(&path) {
                        Ok(m) => m,
                        Err(_) => { last_size = 0; continue; }
                    };
                    let size = meta.len();

                    // Skip stale files from previous sessions.
                    let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if modified < start_time && last_size == 0 {
                        continue; // file predates this clx session
                    }

                    // File was truncated (new session) or grown.
                    if size != last_size {
                        if size < last_size {
                            // New session — file was truncated.
                            last_content.clear();
                        }
                        last_size = size;

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content != last_content && !content.trim().is_empty() {
                                // Keep last ~2000 chars for overlay.
                                let display = if content.len() > 2000 {
                                    format!("...{}", &content[content.len() - 1500..])
                                } else {
                                    content.clone()
                                };
                                platform.show_brainstorm_overlay(&display);
                                last_content = content;
                            }
                        }
                    }

                    // If file hasn't changed for 10s and content is non-empty,
                    // the agent likely finished. Keep showing until dismissed.
                }
            })
            .ok();
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        match key {
            KeyCode::M => {
                if mods.shift {
                    self.kill_agent();
                } else {
                    self.launch_agent();
                }
                true
            }
            KeyCode::Escape => {
                if self.running.load(Ordering::Relaxed) {
                    self.kill_agent();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn on_key_up(&self, _key: KeyCode) -> bool { false }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::M)
    }

    fn launch_agent(&self) {
        // Kill any existing agent first.
        self.kill_agent();

        let platform = Arc::clone(&self.platform);
        let running = &self.running as *const AtomicBool as usize;
        let child_ptr = &self.child as *const Mutex<Option<std::process::Child>> as usize;

        std::thread::Builder::new()
            .name("clx-agent-launcher".into())
            .spawn(move || {
                let running_ref = unsafe { &*(running as *const AtomicBool) };
                let child_ref = unsafe { &*(child_ptr as *const Mutex<Option<std::process::Child>>) };

                // Voice input loop: listen → STT → agent → repeat.
                running_ref.store(true, Ordering::Relaxed);

                loop {
                    if !running_ref.load(Ordering::Relaxed) { break; }

                    platform.show_brainstorm_overlay("🎤 Agent listening... speak your command.\n\nESC to stop.");
                    eprintln!("[CLX] agent: listening...");

                    let prompt = match record_voice_prompt() {
                        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
                        _ => {
                            // No speech — keep listening.
                            eprintln!("[CLX] agent: no speech, retrying...");
                            continue;
                        }
                    };

                    if !running_ref.load(Ordering::Relaxed) { break; }

                    eprintln!("[CLX] agent: voice prompt: {}", &prompt[..prompt.len().min(80)]);
                    platform.show_brainstorm_overlay(&format!("🎤 \"{}\"\n\nRunning agent...", prompt));

                // Spawn `clx agent --prompt "..."` with stderr piped for overlay.
                let clx_bin = std::env::current_exe().unwrap_or_else(|_| "clx".into());

                match std::process::Command::new(&clx_bin)
                    .arg("agent")
                    .arg("--prompt")
                    .arg(&prompt)
                    .stderr(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(mut child) => {
                        let pid = child.id();
                        eprintln!("[CLX] agent: spawned pid={}", pid);
                        running_ref.store(true, Ordering::Relaxed);

                        // Take stderr before storing child (borrow checker).
                        let stderr = child.stderr.take();
                        *child_ref.lock().unwrap() = Some(child);

                        // Stream stderr lines to the overlay in real-time.
                        let mut log = format!("Agent: {}\n", prompt);
                        if let Some(stderr) = stderr {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                let line = match line {
                                    Ok(l) => l,
                                    Err(_) => break,
                                };
                                eprintln!("[CLX] agent: {}", line);

                                // Append to log, keep last ~2000 chars for overlay.
                                log.push_str(&line);
                                log.push('\n');
                                if log.len() > 2000 {
                                    let start = log.len() - 1500;
                                    log = format!("...{}", &log[start..]);
                                }
                                platform.show_brainstorm_overlay(&log);
                            }
                        }

                        // Wait for child to finish.
                        if let Some(ref mut c) = *child_ref.lock().unwrap() {
                            match c.wait() {
                                Ok(status) => {
                                    log.push_str(&format!("\n— Done ({}). ESC to dismiss.", status));
                                    platform.show_brainstorm_overlay(&log);
                                }
                                Err(e) => {
                                    log.push_str(&format!("\n— Error: {}", e));
                                    platform.show_brainstorm_overlay(&log);
                                }
                            }
                        }
                        *child_ref.lock().unwrap() = None;
                    }
                    Err(e) => {
                        eprintln!("[CLX] agent: failed to spawn: {}", e);
                        platform.show_brainstorm_overlay(&format!(
                            "Agent error: {}\n\n🎤 Listening for next command...", e
                        ));
                    }
                }

                } // end loop

                running_ref.store(false, Ordering::Relaxed);
                platform.hide_brainstorm_overlay();
            })
            .ok();
    }

    fn kill_agent(&self) {
        if let Some(ref mut child) = *self.child.lock().unwrap() {
            eprintln!("[CLX] agent: killing pid={}", child.id());
            let _ = child.kill();
        }
        self.running.store(false, Ordering::Relaxed);
        self.platform.hide_brainstorm_overlay();
    }
}
