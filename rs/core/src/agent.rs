/// CapsLockX Agent — LLM with tool use (web search, URL fetch, shell).
///
/// Wraps `llm_client` with an agentic tool-use loop:
///   1. Send messages + tool definitions to LLM
///   2. LLM responds (may include tool calls)
///   3. Execute tools, append results
///   4. Repeat until LLM gives a final text response (max 10 iterations)

use crate::llm_client::{LlmConfig, LlmProvider, Message};

const MAX_TOOL_ITERATIONS: usize = 10;

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
        description: "Execute JavaScript code in a sandboxed engine (Boa). No filesystem/network access. Use for calculations, data transformations, JSON parsing, string manipulation. Returns the result of the last expression.",
        params_json: r#"{"type":"object","properties":{"code":{"type":"string","description":"JavaScript code to execute"}},"required":["code"]}"#,
    },
    ToolDef {
        name: "math_eval",
        description: "Evaluate a Wolfram Language / Mathematica expression (via Woxi). Use for symbolic math, calculus, algebra, number theory, equation solving. Examples: 'Integrate[x^2, x]', 'Solve[x^2 - 4 == 0, x]', 'Factor[x^4 - 1]', 'N[Pi, 50]'.",
        params_json: r#"{"type":"object","properties":{"expr":{"type":"string","description":"Wolfram Language expression"}},"required":["expr"]}"#,
    },
    ToolDef {
        name: "screenshot",
        description: "Take a screenshot of the current screen. Returns image metadata. Use read_screen for text.",
        params_json: r#"{"type":"object","properties":{}}"#,
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
    for iteration in 0..MAX_TOOL_ITERATIONS {
        let result = match config.provider {
            LlmProvider::Gemini => gemini_turn(config, messages, on_token)?,
            LlmProvider::OpenAI => openai_turn(config, messages, on_token)?,
            LlmProvider::Anthropic => anthropic_turn(config, messages, on_token)?,
        };

        match result {
            TurnResult::Text(text) => return Ok(text),
            TurnResult::ToolCalls(calls) => {
                for call in calls {
                    on_status(&format!("Running {}({})...", call.name,
                        call.args.chars().take(40).collect::<String>()));
                    eprintln!("[CLX] agent: tool call #{}: {}({})",
                        iteration + 1, call.name, call.args.chars().take(80).collect::<String>());

                    let result = execute_tool(&call.name, &call.args);
                    eprintln!("[CLX] agent: tool result: {} chars", result.len());

                    // Append tool result to messages (format varies by provider).
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
            let code = args["code"].as_str().unwrap_or("");
            js_eval(code)
        }
        "math_eval" => {
            let expr = args["expr"].as_str().unwrap_or("");
            math_eval(expr)
        }
        "screenshot" => take_screenshot_and_describe(),
        "read_screen" => read_screen_text(),
        "read_clipboard" => read_clipboard(),
        _ => format!("Unknown tool: {}", name),
    }
}

/// Execute JavaScript in sandboxed Boa engine. No I/O, no network, no filesystem.
fn js_eval(code: &str) -> String {
    use boa_engine::{Context, Source};

    eprintln!("[CLX] agent: js_eval({} chars)", code.len());
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

    let resp = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("OpenAI: {}", e))?;

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
        LlmProvider::OpenAI => {
            // OpenAI: assistant message with tool_calls, then tool role.
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
