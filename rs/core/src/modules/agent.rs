/// CLX Agent — LLM-driven computer control via clx-agent subprocess.
///
/// Keybindings:
///   CLX+M       — Prompt for task, launch clx-agent
///   CLX+Shift+M — Kill running agent
///   ESC         — Kill running agent (when active)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

pub struct AgentModule {
    platform: Arc<dyn Platform>,
    running: AtomicBool,
    child: Mutex<Option<std::process::Child>>,
}

impl AgentModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self {
            platform,
            running: AtomicBool::new(false),
            child: Mutex::new(None),
        }
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

        // Capture selected text for context.
        let selected_text = self.platform.get_selected_text();

        let platform = Arc::clone(&self.platform);
        let running = &self.running as *const AtomicBool as usize;
        let child_ptr = &self.child as *const Mutex<Option<std::process::Child>> as usize;

        std::thread::Builder::new()
            .name("clx-agent-launcher".into())
            .spawn(move || {
                let running_ref = unsafe { &*(running as *const AtomicBool) };
                let child_ref = unsafe { &*(child_ptr as *const Mutex<Option<std::process::Child>>) };

                // Show prompt dialog (subprocess — keyboard works normally).
                let prefill = if selected_text.is_empty() {
                    String::new()
                } else {
                    selected_text
                };

                let prompt = match platform.show_prompt_input(
                    "CLX Agent",
                    "What should the agent do? It will control your keyboard and mouse.",
                    &prefill,
                ) {
                    Some(p) if !p.trim().is_empty() => p.trim().to_string(),
                    _ => {
                        eprintln!("[CLX] agent: cancelled");
                        return;
                    }
                };

                eprintln!("[CLX] agent: launching with prompt: {}", &prompt[..prompt.len().min(80)]);

                // Find clx-agent binary next to the main binary.
                let agent_bin = {
                    let exe = std::env::current_exe().unwrap_or_default();
                    let dir = exe.parent().unwrap_or(std::path::Path::new("."));
                    let bin = dir.join("clx-agent");
                    if bin.exists() { bin } else { std::path::PathBuf::from("clx-agent") }
                };

                // Spawn clx-agent --prompt "..."
                match std::process::Command::new(&agent_bin)
                    .arg("--prompt")
                    .arg(&prompt)
                    .stderr(std::process::Stdio::inherit()) // show logs
                    .stdout(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => {
                        let pid = child.id();
                        eprintln!("[CLX] agent: spawned clx-agent pid={}", pid);
                        running_ref.store(true, Ordering::Relaxed);
                        *child_ref.lock().unwrap() = Some(child);

                        // Wait for it to finish.
                        if let Some(ref mut c) = *child_ref.lock().unwrap() {
                            match c.wait() {
                                Ok(status) => eprintln!("[CLX] agent: exited with {}", status),
                                Err(e) => eprintln!("[CLX] agent: wait error: {}", e),
                            }
                        }
                        running_ref.store(false, Ordering::Relaxed);
                        *child_ref.lock().unwrap() = None;
                    }
                    Err(e) => {
                        eprintln!("[CLX] agent: failed to spawn clx-agent: {}", e);
                        eprintln!("[CLX] agent: looked for binary at {:?}", agent_bin);
                    }
                }
            })
            .ok();
    }

    fn kill_agent(&self) {
        if let Some(ref mut child) = *self.child.lock().unwrap() {
            eprintln!("[CLX] agent: killing clx-agent pid={}", child.id());
            let _ = child.kill();
        }
        self.running.store(false, Ordering::Relaxed);
    }
}
