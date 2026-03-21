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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
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
If given code or text, help with it. You can see the conversation history for context. \
IMPORTANT: When translating text or teaching language, ALWAYS use the speak tool to read the result aloud. \
Speak each phrase separately (max 32 chars), using wait() between calls to avoid overlap.";

const HISTORY_FILENAME: &str = "brainstorm_history.json";

const DEFAULT_ORIGIN: &str = "https://brainstorm.snomiao.com";

pub struct BrainstormModule {
    platform: Arc<dyn Platform>,
    state: AtomicU8,
    cancel: Arc<AtomicBool>,
    /// Generation counter — incremented on each new turn, so old turns stop.
    generation: AtomicU32,
    /// In-memory chat history for the current session.
    history: Mutex<Vec<Message>>,
    /// Last response for re-show.
    last_response: Mutex<String>,
    /// LLM config (behind Mutex for hot-reload from prefs).
    llm_config: Mutex<Option<LlmConfig>>,
    /// Web UI origin.
    origin: String,
    /// Whether "keep history" is checked (persists across restarts).
    keep_history: AtomicBool,
}

fn history_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join(HISTORY_FILENAME)
}

fn load_persistent_history() -> Vec<Message> {
    let path = history_path();
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let arr: Vec<serde_json::Value> = match serde_json::from_str(&data) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };
    arr.iter().filter_map(|v| {
        Some(Message {
            role: v["role"].as_str()?.to_string(),
            content: v["content"].as_str()?.to_string(),
        })
    }).collect()
}

fn save_persistent_history(messages: &[Message]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Skip system prompt (index 0) when saving.
    let arr: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m.role != "system")
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect();
    let json = serde_json::to_string_pretty(&arr).unwrap_or_default();
    let _ = std::fs::write(path, json);
}

fn count_persistent_history() -> usize {
    load_persistent_history().len()
}

fn clear_persistent_history() {
    let _ = std::fs::remove_file(history_path());
}

fn load_keep_history_pref() -> bool {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("brainstorm_keep_history");
    path.exists()
}

fn save_keep_history_pref(keep: bool) {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("brainstorm_keep_history");
    if keep {
        let _ = std::fs::write(&path, "1");
    } else {
        let _ = std::fs::remove_file(&path);
    }
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

        let keep = load_keep_history_pref();
        let mut history = vec![
            Message { role: "system".into(), content: SYSTEM_PROMPT.into() },
        ];
        // Load persistent history if preference is set.
        if keep {
            let saved = load_persistent_history();
            if !saved.is_empty() {
                history.extend(saved);
                eprintln!("[CLX] brainstorm: loaded {} saved history messages", history.len() - 1);
            }
        }

        Self {
            platform,
            state: AtomicU8::new(STATE_IDLE),
            cancel: Arc::new(AtomicBool::new(false)),
            generation: AtomicU32::new(0),
            history: Mutex::new(history),
            last_response: Mutex::new(String::new()),
            llm_config: Mutex::new(llm_config),
            origin: if origin.is_empty() { DEFAULT_ORIGIN.to_string() } else { origin },
            keep_history: AtomicBool::new(keep),
        }
    }

    /// Hot-reload LLM config from preferences.
    pub fn update_llm_config(&self, api_key: &str, model: &str) {
        let new_config = if api_key.is_empty() {
            None
        } else {
            Some(LlmConfig::from_key_and_model(api_key, model))
        };
        *self.llm_config.lock().unwrap() = new_config;
        eprintln!("[CLX] brainstorm: LLM config hot-reloaded");
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
        clear_persistent_history();
        self.platform.show_brainstorm_overlay("Chat history cleared.");
        self.state.store(STATE_DONE, Ordering::Relaxed);
        eprintln!("[CLX] brainstorm: history cleared (memory + disk)");
    }

    fn start_turn(&self) {
        let config = match self.llm_config.lock().unwrap().clone() {
            Some(c) => c,
            None => {
                self.platform.show_brainstorm_overlay(
                    "No LLM API key configured.\nSet one in Preferences → LLM → API Key."
                );
                self.state.store(STATE_DONE, Ordering::Relaxed);
                return;
            }
        };

        // Cancel any ongoing request and bump generation so old threads stop.
        self.cancel.store(true, Ordering::Relaxed);
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;

        // Capture selected text NOW (on event tap thread) before focus changes.
        let selected_text = self.platform.get_selected_text();
        eprintln!("[CLX] brainstorm: selected text via AX: {} chars", selected_text.len());

        // Everything else on background thread.
        let platform = Arc::clone(&self.platform);
        let cancel = Arc::clone(&self.cancel);
        let history_ptr = &self.history as *const Mutex<Vec<Message>> as usize;
        let state_ptr = &self.state as *const AtomicU8 as usize;
        let last_resp_ptr = &self.last_response as *const Mutex<String> as usize;
        let keep_ptr = &self.keep_history as *const AtomicBool as usize;
        let gen_ptr = &self.generation as *const AtomicU32 as usize;

        std::thread::Builder::new()
            .name("clx-brainstorm".into())
            .spawn(move || {
                // Wait briefly for previous thread to observe cancel.
                std::thread::sleep(std::time::Duration::from_millis(50));

                let gen_ref = unsafe { &*(gen_ptr as *const AtomicU32) };
                // If generation advanced again, a newer turn superseded us.
                if gen_ref.load(Ordering::SeqCst) != gen {
                    return;
                }
                // Reset cancel for this turn.
                cancel.store(false, Ordering::SeqCst);

                let history_ref = unsafe { &*(history_ptr as *const Mutex<Vec<Message>>) };
                let state_ref = unsafe { &*(state_ptr as *const AtomicU8) };
                let last_resp_ref = unsafe { &*(last_resp_ptr as *const Mutex<String>) };
                let keep_ref = unsafe { &*(keep_ptr as *const AtomicBool) };

                agent_turn(&platform, &config, &cancel, history_ref, state_ref, last_resp_ref, keep_ref, &selected_text);
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
    keep_history: &AtomicBool,
    pre_selected: &str,
) {
    // 1. Use pre-captured selected text (from event tap thread).
    //    Fall back to Cmd+C with clipboard save/restore if AX returned empty.
    let prefill = if !pre_selected.is_empty() {
        pre_selected.to_string()
    } else {
        // Save current clipboard, copy selection, read it, restore clipboard.
        let old_clipboard = platform.get_clipboard_text();
        platform.key_tap_cmd_or_ctrl(KeyCode::C);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let selected = platform.get_clipboard_text();
        // Restore old clipboard if we actually got something new.
        if selected != old_clipboard && !old_clipboard.is_empty() {
            platform.set_clipboard_text(&old_clipboard);
        }
        selected
    };

    let hist_count = count_persistent_history();
    let keep_checked = keep_history.load(Ordering::Relaxed);

    // 2. Show prompt panel. The [KEEP:0/1] prefix carries checkbox state.
    let raw_input = match platform.show_prompt_input(
        "CapsLockX Brainstorm",
        &format!("Chat with AI. Check 'Keep histories' to accumulate context. ({} saved)", hist_count),
        &prefill,
    ) {
        Some(q) if !q.trim().is_empty() => q,
        _ => {
            eprintln!("[CLX] brainstorm: cancelled");
            return;
        }
    };

    // Parse [KEEP] prefix from checkbox.
    let (wants_keep, question) = if let Some(rest) = raw_input.strip_prefix("[KEEP]\n") {
        (true, rest.to_string())
    } else {
        (false, raw_input)
    };

    if question.trim().is_empty() {
        eprintln!("[CLX] brainstorm: empty question");
        return;
    }

    // Persist the checkbox preference.
    if wants_keep != keep_checked {
        keep_history.store(wants_keep, Ordering::Relaxed);
        save_keep_history_pref(wants_keep);
        eprintln!("[CLX] brainstorm: keep_history changed to {}", wants_keep);
    }

    // 3. Manage history based on checkbox.
    {
        let mut hist = history.lock().unwrap();
        if !wants_keep {
            // Not keeping → start fresh every time, clear saved file.
            hist.truncate(1);
            clear_persistent_history();
        }
        hist.push(Message { role: "user".into(), content: question.clone() });

        let turns = hist.iter().filter(|m| m.role == "user").count();
        eprintln!("[CLX] brainstorm: turn {} ({} chars, {} msgs, keep={})",
            turns, question.len(), hist.len(), wants_keep);
    }

    // 5. Stream LLM response.
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

                // Save to persistent file if keep_history is on.
                if keep_history.load(Ordering::Relaxed) {
                    let hist = history.lock().unwrap();
                    save_persistent_history(&hist);
                    eprintln!("[CLX] brainstorm: saved {} messages to history file", hist.len() - 1);
                }

                platform.set_clipboard_text(&response);
                *last_response.lock().unwrap() = response.clone();
                let keep = keep_history.load(Ordering::Relaxed);
                let saved = if keep { count_persistent_history() } else { 0 };
                platform.show_brainstorm_overlay(&format!(
                    "{}\n\n— Copied. Space+B to continue. {}",
                    response,
                    if keep { format!("({} history records saved)", saved) } else { "ESC to dismiss.".to_string() }
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
