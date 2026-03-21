/// Lightweight multi-provider LLM client using ureq + SSE streaming.
///
/// Supports: OpenAI (GPT-4o), Anthropic (Claude), Google Gemini.
/// No async runtime needed — fully synchronous streaming via BufReader.

use std::io::{BufRead, BufReader};

/// Which LLM provider to use.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    Gemini,     // generativelanguage.googleapis.com
    OpenAI,     // api.openai.com
    Anthropic,  // api.anthropic.com
    Ollama,     // localhost:11434 (OpenAI-compatible)
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: String,
    pub model: String,
    /// Base URL override (for Ollama or custom endpoints).
    pub base_url: Option<String>,
}

impl LlmConfig {
    /// Auto-detect provider from API key prefix or explicit model name.
    pub fn from_key_and_model(api_key: &str, model: &str) -> Self {
        let provider = if model.starts_with("claude") || api_key.starts_with("sk-ant-") {
            LlmProvider::Anthropic
        } else if model.starts_with("gemini") || api_key.starts_with("AIza") {
            LlmProvider::Gemini
        } else if api_key.is_empty() || api_key == "ollama" || model.contains('/') {
            LlmProvider::Ollama
        } else {
            LlmProvider::OpenAI
        };
        let model = if model.is_empty() {
            // Try to discover the best available model from the provider.
            #[cfg(not(target_arch = "wasm32"))]
            let discovered = discover_best_model(&provider, api_key);
            #[cfg(target_arch = "wasm32")]
            let discovered = None;

            discovered.unwrap_or_else(|| match provider {
                LlmProvider::Gemini => "gemini-2.5-flash".to_string(),
                LlmProvider::OpenAI => "gpt-4o".to_string(),
                LlmProvider::Anthropic => "claude-opus-4-20250514".to_string(),
                LlmProvider::Ollama => "qwen3:32b".to_string(),
            })
        } else {
            model.to_string()
        };
        let base_url = if provider == LlmProvider::Ollama {
            // Try MLX server first (port 8321), fall back to Ollama (port 11434).
            if ureq::get("http://localhost:8321/v1/models").call().is_ok() {
                Some("http://localhost:8321".to_string())
            } else {
                Some("http://localhost:11434".to_string())
            }
        } else {
            None
        };
        Self { provider, api_key: api_key.to_string(), model, base_url }
    }

    /// Build a quality-first fallback chain from available API keys.
    /// Order: Gemini → OpenAI → Anthropic → Ollama (local).
    pub fn fallback_chain(
        gemini_key: &str,
        openai_key: &str,
        anthropic_key: &str,
    ) -> Vec<Self> {
        let mut chain = Vec::new();

        if !gemini_key.is_empty() {
            chain.push(Self::from_key_and_model(gemini_key, ""));
        }
        if !openai_key.is_empty() {
            chain.push(Self::from_key_and_model(openai_key, ""));
        }
        if !anthropic_key.is_empty() {
            chain.push(Self::from_key_and_model(anthropic_key, ""));
        }
        // Ollama as final fallback.
        chain.push(Self::from_key_and_model("ollama", ""));

        chain
    }
}

/// Discover the best/latest model from a provider's API.
/// Returns None if discovery fails (falls back to hardcoded defaults).
#[cfg(not(target_arch = "wasm32"))]
fn discover_best_model(provider: &LlmProvider, api_key: &str) -> Option<String> {
    match provider {
        LlmProvider::Gemini => discover_gemini(api_key),
        LlmProvider::OpenAI => discover_openai(api_key),
        LlmProvider::Anthropic => {
            // Anthropic has no list-models endpoint.
            // "claude-opus-4-latest" is the alias for the latest Opus.
            Some("claude-opus-4-latest".to_string())
        }
        LlmProvider::Ollama => discover_ollama(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_gemini(api_key: &str) -> Option<String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );
    let resp = ureq::get(&url).call().ok()?;
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().ok()?).ok()?;
    let models = body["models"].as_array()?;

    // Preference order: flash (fast+cheap) > pro > ultra.
    // Pick the latest version of the best tier.
    let preference = ["gemini-2.5-flash", "gemini-2.5-pro", "gemini-2.0-flash"];
    for pref in &preference {
        for m in models {
            let name = m["name"].as_str().unwrap_or("");
            // name is like "models/gemini-2.5-flash"
            let short = name.strip_prefix("models/").unwrap_or(name);
            if short == *pref {
                eprintln!("[CLX] llm: discovered Gemini model: {}", short);
                return Some(short.to_string());
            }
        }
    }
    // Fallback: return first model that supports generateContent.
    for m in models {
        let methods = m["supportedGenerationMethods"].as_array();
        if let Some(methods) = methods {
            if methods.iter().any(|v| v.as_str() == Some("generateContent")) {
                let name = m["name"].as_str().unwrap_or("");
                let short = name.strip_prefix("models/").unwrap_or(name);
                eprintln!("[CLX] llm: discovered Gemini model (fallback): {}", short);
                return Some(short.to_string());
            }
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_openai(api_key: &str) -> Option<String> {
    let resp = ureq::get("https://api.openai.com/v1/models")
        .set("Authorization", &format!("Bearer {}", api_key))
        .call().ok()?;
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().ok()?).ok()?;
    let models = body["data"].as_array()?;

    // Preference: gpt-4o > gpt-4-turbo > gpt-4 > gpt-3.5-turbo.
    let preference = ["gpt-4o", "gpt-4-turbo", "gpt-4", "gpt-3.5-turbo"];
    for pref in &preference {
        for m in models {
            let id = m["id"].as_str().unwrap_or("");
            if id == *pref {
                eprintln!("[CLX] llm: discovered OpenAI model: {}", id);
                return Some(id.to_string());
            }
        }
    }
    // Fallback: first model containing "gpt".
    for m in models {
        let id = m["id"].as_str().unwrap_or("");
        if id.contains("gpt") {
            eprintln!("[CLX] llm: discovered OpenAI model (fallback): {}", id);
            return Some(id.to_string());
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_ollama() -> Option<String> {
    // Try MLX server first (port 8321).
    if let Ok(resp) = ureq::get("http://localhost:8321/v1/models").call() {
        if let Ok(body) = resp.into_string() {
            // MLX server uses the model name from startup.
            eprintln!("[CLX] llm: discovered MLX server at :8321");
            // MLX requires full HF model name.
            return Some("mlx-community/Qwen2.5-3B-Instruct-4bit".to_string());
        }
    }

    // Fall back to Ollama (port 11434).
    let resp = ureq::get("http://localhost:11434/api/tags").call().ok()?;
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().ok()?).ok()?;
    let models = body["models"].as_array()?;

    if models.is_empty() { return None; }

    let mut best: Option<(String, u64)> = None;
    for m in models {
        let name = m["name"].as_str().unwrap_or("");
        let size: u64 = name.split(':').last().unwrap_or("")
            .trim_end_matches('b').trim_end_matches('B')
            .parse().unwrap_or(0);
        if best.is_none() || size > best.as_ref().unwrap().1 {
            best = Some((name.to_string(), size));
        }
    }

    if let Some((name, _)) = best {
        eprintln!("[CLX] llm: discovered Ollama model: {}", name);
        Some(name)
    } else {
        let name = models[0]["name"].as_str().unwrap_or("qwen3:32b");
        eprintln!("[CLX] llm: discovered Ollama model (first): {}", name);
        Some(name.to_string())
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
        LlmProvider::Gemini => stream_gemini(config, messages, on_token),
        LlmProvider::OpenAI => stream_openai(config, messages, on_token),
        LlmProvider::Anthropic => stream_anthropic(config, messages, on_token),
        LlmProvider::Ollama => stream_ollama(config, messages, on_token),
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

/// Ollama: OpenAI-compatible API at localhost:11434.
#[cfg(not(target_arch = "wasm32"))]
fn stream_ollama(
    config: &LlmConfig,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    let base = config.base_url.as_deref().unwrap_or("http://localhost:11434");
    let url = format!("{}/v1/chat/completions", base);

    let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
        serde_json::json!({"role": m.role, "content": m.content})
    }).collect();

    let body = serde_json::json!({
        "model": config.model,
        "messages": msgs,
        "stream": true,
    });

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Ollama ({}): {}", base, e))?;

    // Same SSE format as OpenAI.
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

#[cfg(target_arch = "wasm32")]
pub fn stream_chat(
    _config: &LlmConfig,
    _messages: &[Message],
    _on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    Err("LLM streaming not yet supported on wasm".into())
}
