use std::sync::atomic::{AtomicBool, Ordering};
/// CLX Agent — LLM-driven computer control.
///
/// **Temporarily disabled (2026-11).** Agent mode was driven by voice: CLX+M
/// routed CLX's own speech-to-text into `clx agent --prompt`. Voice input has
/// moved out of CapsLockX into otoji (CLX+V now just triggers otoji), so there
/// is no transcript left to feed the agent. Until otoji can hand transcripts
/// back, the hotkeys below are unbound: `Modules` skips this module while
/// [`ENABLED`] is false. The code is kept compiling (and unit-tested) so it can
/// be switched back on; the `clx agent` subcommand is unaffected.
///
/// Keybindings (when enabled):
///   CLX+M       — Toggle agent mode
///   CLX+Shift+M — Force stop agent
///   ESC         — Stop agent
use std::sync::Arc;
use std::sync::Mutex;

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

/// Whether CLX+M is bound. See the module docs for why this is off.
pub const ENABLED: bool = false;

/// Global flag: agent mode on/off.
static AGENT_MODE: AtomicBool = AtomicBool::new(false);

/// Check if agent mode is active.
pub fn is_agent_mode() -> bool {
    AGENT_MODE.load(Ordering::Relaxed)
}

pub struct AgentModule {
    platform: Arc<dyn Platform>,
    running: AtomicBool,
    child: Mutex<Option<std::process::Child>>,
}

/// Path to the live overlay file (shared with clx-agent).
fn live_log_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
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
                        Err(_) => {
                            last_size = 0;
                            continue;
                        }
                    };
                    let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if modified < start_time && last_size == 0 {
                        continue;
                    }

                    let size = meta.len();
                    if size != last_size {
                        if size < last_size {
                            last_content.clear();
                        }
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

    pub fn on_key_up(&self, _key: KeyCode) -> bool {
        false
    }

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
            eprintln!("[CLX] agent: MODE ON");
            self.platform
                .show_brainstorm_overlay("🤖 Agent Mode ON\n\nSpace+M or ESC to stop.");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn make_module() -> (AgentModule, Arc<MockPlatform>) {
        let mock = Arc::new(MockPlatform::new());
        let m = AgentModule::new(mock.clone() as Arc<dyn Platform>);
        (m, mock)
    }

    #[test]
    fn is_mapped_key_only_m() {
        let _g = TEST_LOCK.lock().unwrap();
        let (m, _mock) = make_module();
        assert!(m.is_mapped_key(KeyCode::M));
        assert!(!m.is_mapped_key(KeyCode::A));
        assert!(!m.is_mapped_key(KeyCode::B));
        assert!(!m.is_mapped_key(KeyCode::Escape));
        assert!(!m.is_mapped_key(KeyCode::Space));
    }

    #[test]
    fn on_key_up_always_false() {
        let _g = TEST_LOCK.lock().unwrap();
        let (m, _mock) = make_module();
        assert!(!m.on_key_up(KeyCode::M));
        assert!(!m.on_key_up(KeyCode::Escape));
    }

    #[test]
    fn on_key_down_unmapped_returns_false() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(false, Ordering::Relaxed);
        let (m, mock) = make_module();
        mock.clear();
        let mods = Modifiers::default();
        assert!(!m.on_key_down(KeyCode::A, &mods));
        assert!(!m.on_key_down(KeyCode::B, &mods));
        assert!(!m.on_key_down(KeyCode::Space, &mods));
    }

    #[test]
    fn toggle_agent_mode_on_then_off() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(false, Ordering::Relaxed);
        let (m, mock) = make_module();
        mock.clear();
        let mods = Modifiers::default();

        assert!(m.on_key_down(KeyCode::M, &mods));
        assert!(AGENT_MODE.load(Ordering::Relaxed));
        assert!(is_agent_mode());
        let calls = mock.calls();
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains("Agent Mode ON"))));

        mock.clear();
        assert!(m.on_key_down(KeyCode::M, &mods));
        assert!(!AGENT_MODE.load(Ordering::Relaxed));
        let calls = mock.calls();
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains("Agent Mode OFF"))));

        AGENT_MODE.store(false, Ordering::Relaxed);
    }

    #[test]
    fn shift_m_force_stops() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(true, Ordering::Relaxed);
        let (m, mock) = make_module();
        mock.clear();
        let mods = Modifiers {
            shift: true,
            ..Default::default()
        };
        assert!(m.on_key_down(KeyCode::M, &mods));
        assert!(!AGENT_MODE.load(Ordering::Relaxed));
        let calls = mock.calls();
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains("OFF"))));
    }

    #[test]
    fn escape_when_agent_off_returns_false() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(false, Ordering::Relaxed);
        let (m, mock) = make_module();
        mock.clear();
        let mods = Modifiers::default();
        assert!(!m.on_key_down(KeyCode::Escape, &mods));
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn escape_when_agent_on_stops() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(true, Ordering::Relaxed);
        let (m, mock) = make_module();
        mock.clear();
        let mods = Modifiers::default();
        assert!(m.on_key_down(KeyCode::Escape, &mods));
        assert!(!AGENT_MODE.load(Ordering::Relaxed));
        let calls = mock.calls();
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains("OFF"))));
    }

    #[test]
    fn is_agent_mode_reflects_global() {
        let _g = TEST_LOCK.lock().unwrap();
        AGENT_MODE.store(false, Ordering::Relaxed);
        assert!(!is_agent_mode());
        AGENT_MODE.store(true, Ordering::Relaxed);
        assert!(is_agent_mode());
        AGENT_MODE.store(false, Ordering::Relaxed);
    }

    #[test]
    fn live_log_path_is_deterministic() {
        let p = live_log_path();
        assert!(p.to_string_lossy().contains("agent-live.log"));
    }
}
