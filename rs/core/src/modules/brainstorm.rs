/// CLX-Brainstorm — Agent loop with persistent chat history.
///
/// Keybindings:
///   CLX+B       — Open prompt, send to LLM, stream response (continues chat)
///   CLX+Shift+B — Clear chat history, start fresh
///   CLX+Ctrl+B  — Open brainstorm web UI in browser
///   ESC         — Dismiss overlay
///
/// The agent maintains conversation history across multiple CLX+B presses.
/// Each interaction appends to the history, giving the LLM full context.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;

use crate::key_code::{KeyCode, Modifiers};
use crate::llm_client::{LlmConfig, Message};
use crate::agent::agent_chat;
use crate::platform::Platform;

const STATE_IDLE: u8 = 0;
const STATE_STREAMING: u8 = 1;
const STATE_DONE: u8 = 2;

const SYSTEM_PROMPT: &str = "\
You are CapsLockX Brainstorm, a helpful assistant embedded in a desktop productivity tool. \
Answer concisely and helpfully. Use the same language as the user's input. \
If given code or text, help with it. You can see the conversation history for context.";

const DEFAULT_ORIGIN: &str = "https://brainstorm.snomiao.com";

pub struct BrainstormModule {
    platform: Arc<dyn Platform>,
    state: AtomicU8,
    cancel: Arc<AtomicBool>,
    /// Persistent chat history — survives across CLX+B presses.
    history: Mutex<Vec<Message>>,
    /// Last response for re-show.
    last_response: Mutex<String>,
    /// LLM config.
    llm_config: Option<LlmConfig>,
    /// Web UI origin.
    origin: String,
}

impl BrainstormModule {
    pub fn new(
        platform: Arc<dyn Platform>,
        origin: String,
        llm_api_key: String,
        llm_model: String,
    ) -> Self {
        let llm_config = if llm_api_key.is_empty() {
            None
        } else {
            Some(LlmConfig::from_key_and_model(&llm_api_key, &llm_model))
        };

        let history = vec![
            Message { role: "system".into(), content: SYSTEM_PROMPT.into() },
        ];

        Self {
            platform,
            state: AtomicU8::new(STATE_IDLE),
            cancel: Arc::new(AtomicBool::new(false)),
            history: Mutex::new(history),
            last_response: Mutex::new(String::new()),
            llm_config,
            origin: if origin.is_empty() { DEFAULT_ORIGIN.to_string() } else { origin },
        }
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        match key {
            KeyCode::B => {
                if mods.ctrl {
                    self.open_web_ui();
                } else if mods.shift {
                    self.clear_history();
                } else {
                    self.start_turn();
                }
                true
            }
            KeyCode::Escape => {
                if self.state.load(Ordering::Relaxed) != STATE_IDLE {
                    self.dismiss();
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
        matches!(key, KeyCode::B)
    }

    fn open_web_ui(&self) {
        let url = format!("{}/", self.origin);
        #[cfg(target_os = "macos")]
        { let _ = std::process::Command::new("open").arg(&url).spawn(); }
        #[cfg(target_os = "windows")]
        { let _ = std::process::Command::new("cmd").args(["/c", "start", &url]).spawn(); }
        #[cfg(target_os = "linux")]
        { let _ = std::process::Command::new("xdg-open").arg(&url).spawn(); }
    }

    fn clear_history(&self) {
        let mut hist = self.history.lock().unwrap();
        hist.truncate(1); // keep system prompt
        *self.last_response.lock().unwrap() = String::new();
        self.platform.show_brainstorm_overlay("Chat history cleared.");
        self.state.store(STATE_DONE, Ordering::Relaxed);
        eprintln!("[CLX] brainstorm: history cleared");
    }

    fn start_turn(&self) {
        if self.llm_config.is_none() {
            self.platform.show_brainstorm_overlay(
                "No LLM API key configured.\nSet one in Preferences → LLM → API Key."
            );
            self.state.store(STATE_DONE, Ordering::Relaxed);
            return;
        }

        // Cancel any ongoing request.
        self.cancel.store(true, Ordering::Relaxed);

        // Everything on background thread.
        let platform = Arc::clone(&self.platform);
        let cancel = Arc::clone(&self.cancel);
        let config = self.llm_config.clone().unwrap();
        let history_ptr = &self.history as *const Mutex<Vec<Message>> as usize;
        let state_ptr = &self.state as *const AtomicU8 as usize;
        let last_resp_ptr = &self.last_response as *const Mutex<String> as usize;

        std::thread::Builder::new()
            .name("clx-brainstorm".into())
            .spawn(move || {
                let history_ref = unsafe { &*(history_ptr as *const Mutex<Vec<Message>>) };
                let state_ref = unsafe { &*(state_ptr as *const AtomicU8) };
                let last_resp_ref = unsafe { &*(last_resp_ptr as *const Mutex<String>) };

                agent_turn(&platform, &config, &cancel, history_ref, state_ref, last_resp_ref);
            })
            .ok();
    }

    fn dismiss(&self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.state.store(STATE_IDLE, Ordering::Relaxed);
        self.platform.hide_brainstorm_overlay();
    }
}

/// One turn of the agent loop: prompt → LLM → response → update history.
fn agent_turn(
    platform: &Arc<dyn Platform>,
    config: &LlmConfig,
    cancel: &AtomicBool,
    history: &Mutex<Vec<Message>>,
    state: &AtomicU8,
    last_response: &Mutex<String>,
) {
    // 1. Copy selection.
    platform.key_tap_ctrl(KeyCode::C);
    std::thread::sleep(std::time::Duration::from_millis(150));
    let clipboard = platform.get_clipboard_text();

    // 2. Format prefill: clipboard at top, then prompt area.
    let prefill = if clipboard.trim().is_empty() {
        String::new()
    } else {
        clipboard
    };

    // 3. Show prompt panel (non-modal, blocks this thread only).
    let question = match platform.show_prompt_input(
        "CapsLockX Brainstorm",
        "Chat with AI. History is preserved across turns.",
        &prefill,
    ) {
        Some(q) if !q.trim().is_empty() => q,
        _ => {
            eprintln!("[CLX] brainstorm: cancelled");
            return;
        }
    };

    // 4. Append user message to history.
    {
        let mut hist = history.lock().unwrap();
        hist.push(Message { role: "user".into(), content: question.clone() });

        // Show turn count in log.
        let turns = hist.iter().filter(|m| m.role == "user").count();
        eprintln!("[CLX] brainstorm: turn {} ({} chars, {} history msgs)",
            turns, question.len(), hist.len());
    }

    // 5. Stream LLM response.
    cancel.store(false, Ordering::Relaxed);
    state.store(STATE_STREAMING, Ordering::Relaxed);
    platform.show_brainstorm_overlay("Thinking...");

    let mut messages = history.lock().unwrap().clone();
    let mut accumulated = String::new();
    let platform_clone = Arc::clone(platform);

    match agent_chat(config, &mut messages, &mut |token| {
        if cancel.load(Ordering::Relaxed) { return; }
        accumulated.push_str(token);

        let display = if accumulated.chars().count() > 2000 {
            let chars: Vec<char> = accumulated.chars().collect();
            let start = chars.len().saturating_sub(1500);
            format!("...{}", chars[start..].iter().collect::<String>())
        } else {
            accumulated.clone()
        };
        platform.show_brainstorm_overlay(&display);
    }, &mut |status| {
        // Show tool execution status.
        platform_clone.show_brainstorm_overlay(&format!("🔧 {}", status));
    }) {
        Ok(final_text) => {
            let response = if final_text.is_empty() { accumulated } else { final_text };
            if !response.is_empty() {
                // Update history with full conversation (includes tool calls/results).
                *history.lock().unwrap() = messages;

                // Trim if too long.
                {
                    let mut hist = history.lock().unwrap();
                    if hist.len() > 42 {
                        let system = hist[0].clone();
                        let tail: Vec<Message> = hist[hist.len()-40..].to_vec();
                        *hist = vec![system];
                        hist.extend(tail);
                    }
                }

                platform.set_clipboard_text(&response);
                *last_response.lock().unwrap() = response.clone();
                platform.show_brainstorm_overlay(&format!(
                    "{}\n\n— Copied. Space+B to continue, ESC to dismiss.", response
                ));
                eprintln!("[CLX] brainstorm: response {} chars", response.len());
            }
        }
        Err(e) => {
            eprintln!("[CLX] brainstorm: error: {}", e);
            history.lock().unwrap().pop();

            let msg = if e.contains("401") || e.contains("403") {
                "Error: Invalid API key. Check Preferences → LLM → API Key.".to_string()
            } else if e.contains("429") {
                "Error: Rate limited. Please wait.".to_string()
            } else {
                format!("Error: {}", e)
            };
            platform.show_brainstorm_overlay(&msg);
        }
    }

    state.store(STATE_DONE, Ordering::Relaxed);
}
