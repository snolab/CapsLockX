/// Lightweight multi-provider LLM client using ureq + SSE streaming.
///
/// Supports: OpenAI (GPT-4o), Anthropic (Claude), Google Gemini.
/// No async runtime needed — fully synchronous streaming via BufReader.

use std::io::{BufRead, BufReader};

/// Which LLM provider to use.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    OpenAI,     // api.openai.com
    Anthropic,  // api.anthropic.com
    Gemini,     // generativelanguage.googleapis.com
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: String,
    pub model: String,
}

impl LlmConfig {
    /// Auto-detect provider from API key prefix or explicit model name.
    pub fn from_key_and_model(api_key: &str, model: &str) -> Self {
        let provider = if model.starts_with("claude") || api_key.starts_with("sk-ant-") {
            LlmProvider::Anthropic
        } else if model.starts_with("gemini") || api_key.starts_with("AIza") {
            LlmProvider::Gemini
        } else {
            LlmProvider::OpenAI
        };
        let model = if model.is_empty() {
            match provider {
                LlmProvider::OpenAI => "gpt-4o-mini".to_string(),
                LlmProvider::Anthropic => "claude-sonnet-4-5-20250514".to_string(),
                LlmProvider::Gemini => "gemini-2.5-flash".to_string(),
            }
        } else {
            model.to_string()
        };
        Self { provider, api_key: api_key.to_string(), model }
    }
}

#[derive(Clone)]
pub struct Message {
    pub role: String,   // "system", "user", "assistant"
    pub content: String,
}

/// Stream a chat completion. Calls `on_token` for each streamed token.
/// Returns the full accumulated response.
#[cfg(not(target_arch = "wasm32"))]
pub fn stream_chat(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    match config.provider {
        LlmProvider::OpenAI => stream_openai(config, messages, on_token),
        LlmProvider::Anthropic => stream_anthropic(config, messages, on_token),
        LlmProvider::Gemini => stream_gemini(config, messages, on_token),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn stream_openai(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
        serde_json::json!({"role": m.role, "content": m.content})
    }).collect();

    let body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "stream": true,
    });

    let resp = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("OpenAI request: {}", e))?;

    let reader = BufReader::new(resp.into_reader());
    let mut full = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read: {}", e))?;
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" { break; }
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = chunk["choices"][0]["delta"]["content"].as_str() {
                    full.push_str(content);
                    on_token(content);
                }
            }
        }
    }
    Ok(full)
}

#[cfg(not(target_arch = "wasm32"))]
fn stream_anthropic(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    // Anthropic separates system from messages.
    let system = messages.iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let msgs: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m.role != "system")
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect();

    let mut body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "max_tokens": 4096,
        "stream": true,
    });
    if !system.is_empty() {
        body["system"] = serde_json::Value::String(system);
    }

    let resp = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &config.api_key)
        .set("anthropic-version", "2023-06-01")
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Anthropic request: {}", e))?;

    let reader = BufReader::new(resp.into_reader());
    let mut full = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read: {}", e))?;
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                if event["type"] == "content_block_delta" {
                    if let Some(text) = event["delta"]["text"].as_str() {
                        full.push_str(text);
                        on_token(text);
                    }
                }
                if event["type"] == "message_stop" { break; }
            }
        }
    }
    Ok(full)
}

#[cfg(not(target_arch = "wasm32"))]
fn stream_gemini(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    // Gemini uses a different format: contents[] with parts[].
    let system_msg = messages.iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());

    let contents: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            let role = if m.role == "assistant" { "model" } else { "user" };
            serde_json::json!({"role": role, "parts": [{"text": m.content}]})
        }).collect();

    let mut body = serde_json::json!({ "contents": contents });
    if let Some(sys) = system_msg {
        body["systemInstruction"] = serde_json::json!({"parts": [{"text": sys}]});
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        config.model, config.api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Gemini request: {}", e))?;

    let reader = BufReader::new(resp.into_reader());
    let mut full = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read: {}", e))?;
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(text) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    full.push_str(text);
                    on_token(text);
                }
            }
        }
    }
    Ok(full)
}

#[cfg(target_arch = "wasm32")]
pub fn stream_chat(
    _config: &LlmConfig,
    _messages: &[Message],
    _on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    Err("LLM streaming not yet supported on wasm".into())
}
