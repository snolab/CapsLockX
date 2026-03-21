/// CapsLockX Agent — LLM with tool use (web search, URL fetch, shell).
///
/// Wraps `llm_client` with an agentic tool-use loop:
///   1. Send messages + tool definitions to LLM
///   2. LLM responds (may include tool calls)
///   3. Execute tools, append results
///   4. Repeat until LLM gives a final text response (max 10 iterations)

use crate::llm_client::{LlmConfig, LlmProvider, Message, stream_chat};

const MAX_TOOL_ITERATIONS: usize = 10;
/// Max chars for inline tool result. Larger outputs saved to file.
const INLINE_RESULT_LIMIT: usize = 2000;
/// Max total chars across all messages before triggering compaction.
const COMPACTION_THRESHOLD: usize = 60_000; // ~15k tokens

/// Tool definitions shared across providers.
struct ToolDef {
    name: &'static str,
    description: &'static str,
    params_json: &'static str, // JSON Schema for parameters
}

const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "web_search",
        description: "Search the web for information. Returns top results with titles and snippets.",
        params_json: r#"{"type":"object","properties":{"query":{"type":"string","description":"Search query"}},"required":["query"]}"#,
    },
    ToolDef {
        name: "fetch_url",
        description: "Fetch the text content of a URL (webpage, API endpoint, etc.). Truncated to 8000 chars.",
        params_json: r#"{"type":"object","properties":{"url":{"type":"string","description":"URL to fetch"}},"required":["url"]}"#,
    },
    ToolDef {
        name: "js_eval",
        description: "Execute JavaScript code in a sandboxed engine. No filesystem/network access. Use for calculations, data transformations, JSON parsing. If execution exceeds timeout, it moves to a background task — use task_status/task_output to check results.",
        params_json: r#"{"type":"object","properties":{"code":{"type":"string","description":"JavaScript code to execute"},"timeout":{"type":"integer","description":"Timeout in seconds (default 5)"}},"required":["code"]}"#,
    },
    ToolDef {
        name: "math_eval",
        description: "Evaluate a Wolfram Language expression (Woxi v0.1, limited). Supports: arithmetic, Prime[n], Sin/Cos/Tan. Does NOT support D[], Integrate[], Solve[]. Use js_eval for unsupported ops. Auto-moves to background if timeout exceeded.",
        params_json: r#"{"type":"object","properties":{"expr":{"type":"string","description":"Wolfram Language expression"},"timeout":{"type":"integer","description":"Timeout in seconds (default 5)"}},"required":["expr"]}"#,
    },
    ToolDef {
        name: "screenshot",
        description: "Take a screenshot of the current screen. Returns image metadata. Use read_screen for text.",
        params_json: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "read_file_range",
        description: "Read a portion of a file by line range. Use this to read large tool outputs that were saved to files. Returns the requested lines.",
        params_json: r#"{"type":"object","properties":{"path":{"type":"string","description":"File path"},"start_line":{"type":"integer","description":"Starting line (1-based, default 1)"},"num_lines":{"type":"integer","description":"Number of lines to read (default 50)"}},"required":["path"]}"#,
    },
    ToolDef {
        name: "read_screen",
        description: "Read the frontmost window's app name, title, and window count via accessibility API.",
        params_json: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "read_clipboard",
        description: "Read the current clipboard text content.",
        params_json: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "task_status",
        description: "Check the status of a background task by ID. Returns whether it's running, completed, killed, or failed.",
        params_json: r#"{"type":"object","properties":{"id":{"type":"integer","description":"Task ID"}},"required":["id"]}"#,
    },
    ToolDef {
        name: "task_output",
        description: "Read output from a background task. Specify start position and max chars to read.",
        params_json: r#"{"type":"object","properties":{"id":{"type":"integer","description":"Task ID"},"start":{"type":"integer","description":"Start char position (default 0)"},"max_chars":{"type":"integer","description":"Max chars to read (default 2000)"}},"required":["id"]}"#,
    },
    ToolDef {
        name: "task_kill",
        description: "Kill a running background task.",
        params_json: r#"{"type":"object","properties":{"id":{"type":"integer","description":"Task ID"}},"required":["id"]}"#,
    },
    ToolDef {
        name: "task_list",
        description: "List all background tasks with their status.",
        params_json: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "speak",
        description: "Speak text aloud (max 128 chars per call). Speeches are queued and play in order — no need to wait between calls. Only use when user explicitly asks to read aloud.",
        params_json: r#"{"type":"object","properties":{"text":{"type":"string","description":"Text to speak (max 128 chars)"},"lang":{"type":"string","description":"Language code: en, ja, zh, ko, etc."}},"required":["text","lang"]}"#,
    },
    ToolDef {
        name: "wait",
        description: "Wait for a specified number of seconds. Use between speak calls to avoid overlap.",
        params_json: r#"{"type":"object","properties":{"seconds":{"type":"number","description":"Seconds to wait"}},"required":["seconds"]}"#,
    },
];

/// Run the agent loop: send messages, handle tool calls, return final text.
/// Calls `on_token` for streaming text output, `on_status` for tool execution updates.
#[cfg(not(target_arch = "wasm32"))]
pub fn agent_chat(
    config: &LlmConfig,
    messages: &mut Vec<Message>,
    on_token: &mut dyn FnMut(&str),
    on_status: &mut dyn FnMut(&str),
) -> Result<String, String> {
    let mut file_counter: u32 = 0;

    for iteration in 0..MAX_TOOL_ITERATIONS {
        // Compact context if too long.
        maybe_compact(config, messages, on_status);

        if iteration > 0 {
            on_status("Thinking...");
        }
        let result = match config.provider {
            LlmProvider::Gemini => gemini_turn(config, messages, on_token)?,
            LlmProvider::OpenAI | LlmProvider::Ollama => openai_turn(config, messages, on_token)?,
            LlmProvider::Anthropic => anthropic_turn(config, messages, on_token)?,
        };

        match result {
            TurnResult::Text(text) => return Ok(text),
            TurnResult::ToolCalls(calls) => {
                // Deduplicate consecutive identical tool calls (LLM sometimes repeats).
                let mut prev_sig = String::new();
                for call in calls {
                    let sig = format!("{}:{}", call.name, call.args);
                    if sig == prev_sig {
                        eprintln!("[CLX] agent: skipping duplicate tool call: {}", call.name);
                        continue;
                    }
                    prev_sig = sig;

                    on_status(&format!("Running {}({})...", call.name,
                        call.args.chars().take(40).collect::<String>()));
                    eprintln!("[CLX] agent: tool call #{}: {}({})",
                        iteration + 1, call.name, call.args.chars().take(80).collect::<String>());

                    let mut result = execute_tool(&call.name, &call.args);
                    eprintln!("[CLX] agent: tool result: {} chars", result.len());
                    on_status(&format!("✓ {} done", call.name));

                    // Save large outputs to file, return path instead.
                    if result.len() > INLINE_RESULT_LIMIT {
                        file_counter += 1;
                        let path = format!("/tmp/clx-agent-output-{}.txt", file_counter);
                        let total_lines = result.lines().count();
                        let _ = std::fs::write(&path, &result);
                        // Return a summary + file path for the agent to read_file_range.
                        let preview: String = result.lines().take(20).collect::<Vec<_>>().join("\n");
                        result = format!(
                            "[Output too large ({} chars, {} lines). Saved to: {}\nFirst 20 lines:\n{}\n\n... use read_file_range(path=\"{}\", start_line=21) to read more]",
                            result.len(), total_lines, path, preview, path
                        );
                        eprintln!("[CLX] agent: saved large output to {}", path);
                    }

                    append_tool_result(config, messages, &call, &result);
                }
            }
        }
    }
    Err("max tool iterations reached".into())
}

enum TurnResult {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

struct ToolCall {
    id: String,
    name: String,
    args: String,
}

// ── Tool execution ───────────────────────────────────────────────────────────

fn execute_tool(name: &str, args_json: &str) -> String {
    let args: serde_json::Value = serde_json::from_str(args_json).unwrap_or_default();

    match name {
        "web_search" => {
            let query = args["query"].as_str().unwrap_or("");
            web_search(query)
        }
        "fetch_url" => {
            let url = args["url"].as_str().unwrap_or("");
            fetch_url(url)
        }
        "js_eval" => {
            let code = args["code"].as_str().unwrap_or("").to_string();
            let timeout = args["timeout"].as_u64().unwrap_or(5);
            run_with_timeout_tool("js_eval", timeout, move || js_eval(&code))
        }
        "math_eval" => {
            let expr = args["expr"].as_str().unwrap_or("").to_string();
            let timeout = args["timeout"].as_u64().unwrap_or(5);
            run_with_timeout_tool("math_eval", timeout, move || math_eval(&expr))
        }
        "read_file_range" => {
            let path = args["path"].as_str().unwrap_or("");
            let start = args["start_line"].as_u64().unwrap_or(1) as usize;
            let num = args["num_lines"].as_u64().unwrap_or(50) as usize;
            read_file_range(path, start, num)
        }
        "screenshot" => take_screenshot_and_describe(),
        "read_screen" => read_screen_text(),
        "read_clipboard" => read_clipboard(),
        "task_status" => {
            let id = args["id"].as_u64().unwrap_or(0) as u32;
            crate::task_manager::task_status(id)
        }
        "task_output" => {
            let id = args["id"].as_u64().unwrap_or(0) as u32;
            let start = args["start"].as_u64().unwrap_or(0) as usize;
            let max = args["max_chars"].as_u64().unwrap_or(2000) as usize;
            crate::task_manager::task_output(id, start, max)
        }
        "task_kill" => {
            let id = args["id"].as_u64().unwrap_or(0) as u32;
            crate::task_manager::task_kill(id)
        }
        "task_list" => crate::task_manager::task_list(),
        "speak" => {
            let text = args["text"].as_str().unwrap_or("");
            let lang = args["lang"].as_str().unwrap_or("en");
            let text: String = text.chars().take(128).collect();
            // Queue speech — plays serially in a background thread.
            // Agent doesn't need to wait; speeches queue up and play in order.
            speech_queue(&text, lang);
            "Queued. Speeches play in order automatically, no need to wait.".to_string()
        }
        "wait" => {
            let secs = args["seconds"].as_f64().unwrap_or(1.0).min(30.0).max(0.1);
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            format!("Waited {:.1}s.", secs)
        }
        _ => format!("Unknown tool: {}", name),
    }
}

/// Execute JavaScript in a sandboxed engine. No I/O, no network, no filesystem.
/// Native: rquickjs (8x faster, 97% ES conformance).
/// WASM: boa_engine (pure Rust, compiles to wasm32).
#[cfg(not(target_arch = "wasm32"))]
fn js_eval(code: &str) -> String {
    use rquickjs::{Runtime, Context};

    eprintln!("[CLX] agent: js_eval({} chars) [rquickjs]", code.len());

    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => return format!("JS runtime error: {:?}", e),
    };

    // Set interrupt handler — checks a deadline to abort long-running scripts.
    // Use 4s internal deadline (slightly less than the 5s task_manager timeout)
    // so QuickJS cleanly returns an error before the task manager forces background.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    rt.set_interrupt_handler(Some(Box::new(move || {
        std::time::Instant::now() > deadline
    })));

    let ctx = match Context::full(&rt) {
        Ok(c) => c,
        Err(e) => return format!("JS context error: {:?}", e),
    };

    ctx.with(|ctx| {
        let wrapped = format!("String(eval({}))", serde_json::json!(code));
        match ctx.eval::<String, _>(wrapped.as_bytes()) {
            Ok(s) => s,
            Err(e) => {
                let err = format!("{:?}", e);
                if err.contains("interrupted") || err.contains("InternalError") {
                    "Execution timed out after 4s. The code ran too long. Try a smaller computation, or break it into smaller steps.".to_string()
                } else {
                    format!("JS error: {}", err)
                }
            }
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn js_eval(code: &str) -> String {
    use boa_engine::{Context, Source};

    eprintln!("[CLX] agent: js_eval({} chars) [boa]", code.len());
    let mut context = Context::default();

    match context.eval(Source::from_bytes(code)) {
        Ok(result) => {
            let output = result.to_string(&mut context);
            match output {
                Ok(s) => s.to_std_string_escaped(),
                Err(e) => format!("toString error: {:?}", e),
            }
        }
        Err(e) => format!("JS error: {:?}", e),
    }
}

/// Evaluate Wolfram Language expression via Woxi.
fn math_eval(expr: &str) -> String {
    eprintln!("[CLX] agent: math_eval({:?})", expr);

    match std::panic::catch_unwind(|| {
        woxi::interpret(expr)
    }) {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => format!("Wolfram error: {:?}", e),
        Err(_) => "Woxi panicked (unsupported expression).".to_string(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn take_screenshot_and_describe() -> String {
    eprintln!("[CLX] agent: taking screenshot...");
    let path = "/tmp/clx-agent-screenshot.png";

    // macOS screencapture
    let status = std::process::Command::new("/usr/sbin/screencapture")
        .args(["-x", "-C", path])
        .status();

    match status {
        Ok(s) if s.success() => {
            // Read the image and encode to base64 for Gemini vision.
            match std::fs::read(path) {
                Ok(bytes) => {
                    use std::io::Write;
                    // Base64 encode.
                    let b64 = base64_encode(&bytes);
                    let size_kb = bytes.len() / 1024;
                    eprintln!("[CLX] agent: screenshot {}KB, sending to vision...", size_kb);

                    // Use Gemini to describe the screenshot.
                    // Return the base64 for the caller to handle, or describe it inline.
                    // For simplicity, return a placeholder with image metadata.
                    // The actual vision analysis happens when the LLM sees the result.
                    format!("[Screenshot taken: {}KB image at {}. Image content (base64, first 100 chars): {}... \
                        Note: Full image analysis requires vision API. Describing based on screen state: \
                        Use read_screen tool for text content, or describe what you need to see.]",
                        size_kb, path, &b64[..b64.len().min(100)])
                }
                Err(e) => format!("Failed to read screenshot: {}", e),
            }
        }
        Ok(s) => format!("screencapture failed with exit code: {}", s),
        Err(e) => format!("screencapture error: {}", e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_screen_text() -> String {
    eprintln!("[CLX] agent: reading screen via accessibility...");

    // Use osascript to get frontmost app info and window text.
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appName to name of frontApp
            set winTitle to ""
            try
                set winTitle to title of first window of frontApp
            end try
            set winCount to count of windows of frontApp
            return "App: " & appName & "\nWindow: " & winTitle & "\nWindows: " & winCount
        end tell
    "#;

    let output = std::process::Command::new("osascript")
        .args(["-e", script])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            if text.trim().is_empty() {
                "Could not read screen text (accessibility permissions may be needed).".to_string()
            } else {
                text
            }
        }
        Ok(o) => format!("osascript error: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => format!("Failed to run osascript: {}", e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_shell(command: &str) -> String {
    eprintln!("[CLX] agent: run_shell({:?})", command);

    // Safety: refuse obviously destructive commands.
    let lower = command.to_lowercase();
    let dangerous = ["rm -rf", "rm -r /", "mkfs", "dd if=", ":(){ ", "fork bomb",
                     "> /dev/sd", "shutdown", "reboot", "halt"];
    for d in &dangerous {
        if lower.contains(d) {
            return format!("Refused: '{}' looks destructive. Will not execute.", command);
        }
    }

    let output = std::process::Command::new("sh")
        .args(["-c", command])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let mut result = String::new();
            if !stdout.is_empty() {
                // Truncate long output.
                let s: String = stdout.chars().take(8000).collect();
                result.push_str(&s);
            }
            if !stderr.is_empty() && !o.status.success() {
                result.push_str("\nSTDERR: ");
                let s: String = stderr.chars().take(2000).collect();
                result.push_str(&s);
            }
            if !o.status.success() {
                result.push_str(&format!("\nExit code: {}", o.status));
            }
            if result.is_empty() { "(no output)".to_string() } else { result }
        }
        Err(e) => format!("Failed to execute: {}", e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_clipboard() -> String {
    eprintln!("[CLX] agent: reading clipboard...");
    let output = std::process::Command::new("pbpaste").output();
    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            let short: String = text.chars().take(4000).collect();
            if short.is_empty() { "(clipboard is empty)".to_string() } else { short }
        }
        Err(e) => format!("Failed to read clipboard: {}", e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn list_files(path: &str) -> String {
    eprintln!("[CLX] agent: list_files({:?})", path);
    let output = std::process::Command::new("ls")
        .args(["-la", path])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let short: String = text.chars().take(4000).collect();
            short
        }
        Ok(o) => format!("ls error: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => format!("Failed to list files: {}", e),
    }
}

/// Serial speech queue — speaks one at a time, no overlap.
fn speech_queue(text: &str, lang: &str) {
    use std::sync::mpsc;
    static SPEECH_TX: std::sync::Mutex<Option<mpsc::Sender<(String, String)>>> = std::sync::Mutex::new(None);

    let mut tx_guard = SPEECH_TX.lock().unwrap();
    if tx_guard.is_none() {
        let (tx, rx) = mpsc::channel::<(String, String)>();
        *tx_guard = Some(tx);
        std::thread::Builder::new()
            .name("clx-speech-queue".into())
            .spawn(move || {
                for (text, lang) in rx {
                    let _ = speak_text(&text, &lang);
                }
            })
            .ok();
    }
    if let Some(ref tx) = *tx_guard {
        let _ = tx.send((text.to_string(), lang.to_string()));
    }
}

/// Run a tool function with timeout. Returns inline result or background task message.
fn run_with_timeout_tool<F>(name: &str, timeout_secs: u64, func: F) -> String
where
    F: FnOnce() -> String + Send + 'static,
{
    use crate::task_manager::{run_with_timeout, ToolResult};
    match run_with_timeout(name, timeout_secs, func) {
        ToolResult::Inline(result) => result,
        ToolResult::Background { message, .. } => message,
    }
}

/// Speak text aloud via the TTS fallback chain.
fn speak_text(text: &str, lang: &str) -> String {
    eprintln!("[CLX] agent: speak({:?}, lang={})", text.chars().take(40).collect::<String>(), lang);
    let el_key = std::env::var("ELEVENLABS_API_KEY").unwrap_or_default();
    let gemini_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    match crate::tts::speak(text, lang, &el_key, &gemini_key, &openai_key) {
        Ok(()) => format!("Spoke aloud: {:?} (lang={})", text.chars().take(50).collect::<String>(), lang),
        Err(e) => format!("TTS error: {}", e),
    }
}

/// Read a range of lines from a file.
fn read_file_range(path: &str, start_line: usize, num_lines: usize) -> String {
    eprintln!("[CLX] agent: read_file_range({:?}, start={}, n={})", path, start_line, num_lines);
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total = lines.len();
            let start = (start_line.max(1) - 1).min(total);
            let end = (start + num_lines).min(total);
            let selected: Vec<String> = lines[start..end].iter()
                .enumerate()
                .map(|(i, l)| format!("{:>5}: {}", start + i + 1, l))
                .collect();
            format!("Lines {}-{} of {} total:\n{}", start + 1, end, total, selected.join("\n"))
        }
        Err(e) => format!("Failed to read {}: {}", path, e),
    }
}

// ── Context compaction ───────────────────────────────────────────────────────

/// Compact conversation history when it exceeds the threshold.
/// Preserves: system prompt, last 4 messages (verbatim), summary of the rest.
#[cfg(not(target_arch = "wasm32"))]
fn maybe_compact(
    config: &LlmConfig,
    messages: &mut Vec<Message>,
    on_status: &mut dyn FnMut(&str),
) {
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    if total_chars < COMPACTION_THRESHOLD || messages.len() < 8 {
        return;
    }

    eprintln!("[CLX] agent: compacting context ({} chars, {} messages)", total_chars, messages.len());
    on_status("Compacting conversation history...");

    // Keep system prompt (index 0) and last 4 messages.
    let system = messages[0].clone();
    let keep_tail = 4.min(messages.len() - 1);
    let tail: Vec<Message> = messages[messages.len() - keep_tail..].to_vec();
    let to_summarize: Vec<&Message> = messages[1..messages.len() - keep_tail].iter().collect();

    if to_summarize.is_empty() { return; }

    // Build summary request.
    let summary_text: String = to_summarize.iter().map(|m| {
        let role = &m.role;
        let content: String = m.content.chars().take(500).collect();
        format!("[{}]: {}", role, content)
    }).collect::<Vec<_>>().join("\n");

    let summary_prompt = vec![
        Message {
            role: "system".into(),
            content: "Summarize this conversation excerpt concisely. Preserve: key decisions, current task, important facts, file paths, tool results. Omit: greetings, repetition, verbose tool output. Return only the summary, 200 words max.".into(),
        },
        Message { role: "user".into(), content: summary_text },
    ];

    let mut summary = String::new();
    match stream_chat(config, &summary_prompt, &mut |token| {
        summary.push_str(token);
    }) {
        Ok(_) => {
            eprintln!("[CLX] agent: compacted {} messages → {} char summary", to_summarize.len(), summary.len());
            // Rebuild: system + summary + tail
            *messages = vec![system];
            messages.push(Message {
                role: "assistant".into(),
                content: format!("[Conversation summary]\n{}", summary),
            });
            messages.extend(tail);
        }
        Err(e) => {
            eprintln!("[CLX] agent: compaction failed: {}, falling back to truncation", e);
            // Fallback: just keep system + last 6 messages.
            let keep = 6.min(messages.len() - 1);
            let tail: Vec<Message> = messages[messages.len() - keep..].to_vec();
            *messages = vec![system];
            messages.extend(tail);
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn web_search(query: &str) -> String {
    eprintln!("[CLX] agent: web_search({:?})", query);
    // Use DuckDuckGo HTML lite — no API key needed.
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoded(query));
    let resp = match ureq::get(&url)
        .set("User-Agent", "CapsLockX/2.0")
        .call() {
        Ok(r) => r,
        Err(e) => return format!("Search error: {}", e),
    };
    let body = resp.into_string().unwrap_or_default();

    // Extract results from DDG HTML: <a class="result__a" href="...">title</a> and <a class="result__snippet">...</a>
    let mut results = Vec::new();
    for (i, chunk) in body.split("result__a").enumerate() {
        if i == 0 || i > 8 { continue; } // skip first (header), limit to 8
        // Extract href
        let href = chunk.split("href=\"").nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("");
        // Extract title text (between > and </a>)
        let title = chunk.split('>').nth(1)
            .and_then(|s| s.split("</").next())
            .map(|s| strip_html_tags(s))
            .unwrap_or_default();
        if !title.is_empty() && !href.is_empty() {
            results.push(format!("{}. {} — {}", results.len() + 1, title, href));
        }
    }

    // Also extract snippets.
    let mut snippets = Vec::new();
    for chunk in body.split("result__snippet") {
        if let Some(text) = chunk.split('>').nth(1) {
            if let Some(t) = text.split("</").next() {
                let clean = strip_html_tags(t).trim().to_string();
                if !clean.is_empty() && clean.len() > 20 {
                    snippets.push(clean);
                }
            }
        }
    }

    if results.is_empty() {
        "No results found.".to_string()
    } else {
        let mut out = format!("Search results for '{}':\n\n", query);
        for (i, r) in results.iter().enumerate() {
            out.push_str(r);
            out.push('\n');
            if let Some(s) = snippets.get(i) {
                out.push_str("   ");
                // Truncate snippet
                let short: String = s.chars().take(200).collect();
                out.push_str(&short);
                out.push('\n');
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn fetch_url(url: &str) -> String {
    eprintln!("[CLX] agent: fetch_url({:?})", url);
    let resp = match ureq::get(url)
        .set("User-Agent", "CapsLockX/2.0")
        .call() {
        Ok(r) => r,
        Err(e) => return format!("Fetch error: {}", e),
    };
    let body = resp.into_string().unwrap_or_default();

    // Strip HTML tags for readability, truncate to 8000 chars.
    let text = strip_html_tags(&body);
    let text: String = text.chars().take(8000).collect();
    if text.is_empty() { "Empty response.".to_string() } else { text }
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { out.push(c); }
    }
    // Collapse whitespace.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn urlencoded(s: &str) -> String {
    s.chars().map(|c| {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c.to_string() }
        else if c == ' ' { "+".to_string() }
        else { format!("%{:02X}", c as u32) }
    }).collect()
}

// ── Provider-specific tool call handling ──────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn gemini_turn(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<TurnResult, String> {
    use std::io::{BufRead, BufReader};

    let system_msg = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
    let contents: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            let role = match m.role.as_str() {
                "assistant" => "model",
                "tool" => "function", // Gemini uses "function" role for tool results
                _ => "user",
            };
            // Check if this is a tool result message (contains JSON with functionResponse).
            if m.role == "tool" {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&m.content) {
                    return serde_json::json!({"role": "function", "parts": [v]});
                }
            }
            serde_json::json!({"role": role, "parts": [{"text": m.content}]})
        }).collect();

    let tool_defs: Vec<serde_json::Value> = TOOLS.iter().map(|t| {
        let params: serde_json::Value = serde_json::from_str(t.params_json).unwrap();
        serde_json::json!({
            "name": t.name,
            "description": t.description,
            "parameters": params,
        })
    }).collect();

    let mut body = serde_json::json!({
        "contents": contents,
        "tools": [{"functionDeclarations": tool_defs}],
    });
    if let Some(sys) = system_msg {
        body["systemInstruction"] = serde_json::json!({"parts": [{"text": sys}]});
    }

    // Non-streaming for tool calls (simpler parsing).
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        config.model, config.api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Gemini: {}", e))?;

    let result: serde_json::Value = serde_json::from_str(&resp.into_string().map_err(|e| format!("read: {}", e))?).map_err(|e| format!("parse: {}", e))?;

    // Check for function calls in response.
    if let Some(parts) = result["candidates"][0]["content"]["parts"].as_array() {
        let mut tool_calls = Vec::new();
        let mut text = String::new();

        for part in parts {
            if let Some(fc) = part.get("functionCall") {
                let name = fc["name"].as_str().unwrap_or("").to_string();
                let args = fc["args"].to_string();
                tool_calls.push(ToolCall { id: name.clone(), name, args });
            }
            if let Some(t) = part["text"].as_str() {
                text.push_str(t);
                on_token(t);
            }
        }

        if !tool_calls.is_empty() {
            // Append model's response to messages (for context).
            return Ok(TurnResult::ToolCalls(tool_calls));
        }
        return Ok(TurnResult::Text(text));
    }

    Ok(TurnResult::Text(String::new()))
}

#[cfg(not(target_arch = "wasm32"))]
fn openai_turn(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<TurnResult, String> {
    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
        if m.role == "tool" {
            // Parse tool result message: "tool_call_id\n\ncontent"
            let (id, content) = m.content.split_once("\n\n").unwrap_or(("", &m.content));
            serde_json::json!({"role": "tool", "tool_call_id": id, "content": content})
        } else {
            serde_json::json!({"role": m.role, "content": m.content})
        }
    }).collect();

    let tool_defs: Vec<serde_json::Value> = TOOLS.iter().map(|t| {
        let params: serde_json::Value = serde_json::from_str(t.params_json).unwrap();
        serde_json::json!({
            "type": "function",
            "function": {"name": t.name, "description": t.description, "parameters": params}
        })
    }).collect();

    let body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "tools": tool_defs,
    });

    let base = config.base_url.as_deref().unwrap_or("https://api.openai.com");
    let url = format!("{}/v1/chat/completions", base);
    let mut req = ureq::post(&url).set("Content-Type", "application/json");
    if !config.api_key.is_empty() {
        req = req.set("Authorization", &format!("Bearer {}", config.api_key));
    }
    let resp = req.send_string(&body.to_string())
        .map_err(|e| format!("OpenAI/Ollama: {}", e))?;

    let result: serde_json::Value = serde_json::from_str(&resp.into_string().map_err(|e| format!("read: {}", e))?).map_err(|e| format!("parse: {}", e))?;
    let choice = &result["choices"][0];

    if let Some(tool_calls) = choice["message"]["tool_calls"].as_array() {
        let calls: Vec<ToolCall> = tool_calls.iter().map(|tc| {
            ToolCall {
                id: tc["id"].as_str().unwrap_or("").to_string(),
                name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                args: tc["function"]["arguments"].as_str().unwrap_or("{}").to_string(),
            }
        }).collect();
        if !calls.is_empty() { return Ok(TurnResult::ToolCalls(calls)); }
    }

    let text = choice["message"]["content"].as_str().unwrap_or("").to_string();
    on_token(&text);
    Ok(TurnResult::Text(text))
}

#[cfg(not(target_arch = "wasm32"))]
fn anthropic_turn(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<TurnResult, String> {
    let system = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone()).unwrap_or_default();
    let msgs: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            if m.role == "tool" {
                // Parse: "tool_use_id\n\ncontent"
                let (id, content) = m.content.split_once("\n\n").unwrap_or(("", &m.content));
                serde_json::json!({"role": "user", "content": [{"type": "tool_result", "tool_use_id": id, "content": content}]})
            } else {
                serde_json::json!({"role": m.role, "content": m.content})
            }
        }).collect();

    let tool_defs: Vec<serde_json::Value> = TOOLS.iter().map(|t| {
        let params: serde_json::Value = serde_json::from_str(t.params_json).unwrap();
        serde_json::json!({"name": t.name, "description": t.description, "input_schema": params})
    }).collect();

    let mut body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "tools": tool_defs,
        "max_tokens": 4096,
    });
    if !system.is_empty() { body["system"] = serde_json::Value::String(system); }

    let resp = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &config.api_key)
        .set("anthropic-version", "2023-06-01")
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Anthropic: {}", e))?;

    let result: serde_json::Value = serde_json::from_str(&resp.into_string().map_err(|e| format!("read: {}", e))?).map_err(|e| format!("parse: {}", e))?;

    let mut tool_calls = Vec::new();
    let mut text = String::new();

    if let Some(content) = result["content"].as_array() {
        for block in content {
            match block["type"].as_str() {
                Some("text") => {
                    let t = block["text"].as_str().unwrap_or("");
                    text.push_str(t);
                    on_token(t);
                }
                Some("tool_use") => {
                    tool_calls.push(ToolCall {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        args: block["input"].to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    if !tool_calls.is_empty() { return Ok(TurnResult::ToolCalls(tool_calls)); }
    Ok(TurnResult::Text(text))
}

fn append_tool_result(config: &LlmConfig, messages: &mut Vec<Message>, call: &ToolCall, result: &str) {
    match config.provider {
        LlmProvider::Gemini => {
            // Gemini: model response with functionCall, then function role with functionResponse.
            messages.push(Message {
                role: "assistant".into(),
                content: format!("[called {}]", call.name),
            });
            let response_json = serde_json::json!({
                "functionResponse": {
                    "name": call.name,
                    "response": {"content": result}
                }
            });
            messages.push(Message {
                role: "tool".into(),
                content: response_json.to_string(),
            });
        }
        LlmProvider::OpenAI | LlmProvider::Ollama => {
            // OpenAI/Ollama: assistant message with tool_calls, then tool role.
            messages.push(Message {
                role: "assistant".into(),
                content: format!("[called {}]", call.name),
            });
            messages.push(Message {
                role: "tool".into(),
                content: format!("{}\n\n{}", call.id, result),
            });
        }
        LlmProvider::Anthropic => {
            // Anthropic: assistant with tool_use block, then user with tool_result.
            messages.push(Message {
                role: "assistant".into(),
                content: format!("[called {}]", call.name),
            });
            messages.push(Message {
                role: "tool".into(),
                content: format!("{}\n\n{}", call.id, result),
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn agent_chat(
    _config: &LlmConfig,
    _messages: &mut Vec<Message>,
    _on_token: &mut dyn FnMut(&str),
    _on_status: &mut dyn FnMut(&str),
) -> Result<String, String> {
    Err("agent not supported on wasm".into())
}
