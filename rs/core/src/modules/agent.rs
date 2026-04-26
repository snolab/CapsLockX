/// CLX Agent — LLM-driven computer control via voice.
///
/// Keybindings:
///   CLX+M       — Toggle agent mode (voice → STT → LLM agent → execute)
///   CLX+Shift+M — Force stop agent
///   ESC         — Stop agent

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::io::{BufRead, BufReader};

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

/// Global flag: when true, voice transcripts are sent to the agent
/// instead of being typed at cursor.
static AGENT_MODE: AtomicBool = AtomicBool::new(false);

/// Check if agent mode is active (called by voice module).
pub fn is_agent_mode() -> bool {
    AGENT_MODE.load(Ordering::Relaxed)
}

/// Enable agent mode programmatically (called by wake-word listener).
/// Idempotent — safe to call when already on.
pub fn enable_agent_mode() {
    if !AGENT_MODE.swap(true, Ordering::Relaxed) {
        eprintln!("[CLX] agent: MODE ON (via wake-word)");
    }
}

/// Called by the voice module when a transcript is ready and agent mode is on.
/// Spawns clx agent --prompt with the transcript.
pub fn on_voice_transcript(text: &str, platform: &dyn Platform) {
    if text.trim().is_empty() { return; }
    let prompt = text.trim().to_string();
    eprintln!("[CLX] agent: voice transcript → \"{}\"", &prompt[..prompt.len().min(80)]);
    platform.show_brainstorm_overlay(&format!("🎤 \"{}\"\n\nRunning agent...", prompt));

    // Spawn clx agent subprocess.
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
            let stderr = child.stderr.take();
            let mut log = format!("🎤 \"{}\"\n", prompt);

            if let Some(stderr) = stderr {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    let line = match line {
                        Ok(l) => l,
                        Err(_) => break,
                    };
                    log.push_str(&line);
                    log.push('\n');
                    if log.len() > 2000 {
                        let start = log.len() - 1500;
                        log = format!("...{}", &log[start..]);
                    }
                    platform.show_brainstorm_overlay(&log);
                }
            }

            let _ = child.wait();
            log.push_str("\n— Done. 🎤 Listening...");
            platform.show_brainstorm_overlay(&log);
        }
        Err(e) => {
            eprintln!("[CLX] agent: spawn error: {}", e);
            platform.show_brainstorm_overlay(&format!("Agent error: {}\n\n🎤 Listening...", e));
        }
    }
}

pub struct AgentModule {
    platform: Arc<dyn Platform>,
    running: AtomicBool,
    child: Mutex<Option<std::process::Child>>,
}

/// Path to the live overlay file (shared with clx-agent).
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
        s.start_live_watcher();
        s
    }

    fn start_live_watcher(&self) {
        let platform = Arc::clone(&self.platform);
        std::thread::Builder::new()
            .name("clx-agent-live-watcher".into())
            .spawn(move || {
                let path = live_log_path();
                let mut last_size: u64 = 0;
                let mut last_content = String::new();
                let start_time = std::time::SystemTime::now();

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    let meta = match std::fs::metadata(&path) {
                        Ok(m) => m,
                        Err(_) => { last_size = 0; continue; }
                    };
                    let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if modified < start_time && last_size == 0 {
                        continue;
                    }

                    let size = meta.len();
                    if size != last_size {
                        if size < last_size { last_content.clear(); }
                        last_size = size;

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content != last_content && !content.trim().is_empty() {
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
                }
            })
            .ok();
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        match key {
            KeyCode::M => {
                if mods.shift {
                    self.stop_agent_mode();
                } else {
                    self.toggle_agent_mode();
                }
                true
            }
            KeyCode::Escape => {
                if AGENT_MODE.load(Ordering::Relaxed) {
                    self.stop_agent_mode();
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

    fn toggle_agent_mode(&self) {
        let was_on = AGENT_MODE.load(Ordering::Relaxed);
        if was_on {
            self.stop_agent_mode();
        } else {
            AGENT_MODE.store(true, Ordering::Relaxed);
            self.running.store(true, Ordering::Relaxed);
            eprintln!("[CLX] agent: MODE ON — voice transcripts → agent");
            self.platform.show_brainstorm_overlay("🤖 Agent Mode ON\n\n🎤 Speak commands. Space+V to dictate.\nSpace+M or ESC to stop.");

            // Also start voice listening if not already active.
            self.platform.show_voice_overlay();
        }
    }

    fn stop_agent_mode(&self) {
        AGENT_MODE.store(false, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
        // Kill any running agent subprocess.
        if let Some(ref mut child) = *self.child.lock().unwrap() {
            eprintln!("[CLX] agent: killing pid={}", child.id());
            let _ = child.kill();
        }
        *self.child.lock().unwrap() = None;
        eprintln!("[CLX] agent: MODE OFF");
        self.platform.show_brainstorm_overlay("🤖 Agent Mode OFF");
        std::thread::spawn({
            let p = Arc::clone(&self.platform);
            move || {
                std::thread::sleep(std::time::Duration::from_secs(2));
                p.hide_brainstorm_overlay();
            }
        });
    }
}
