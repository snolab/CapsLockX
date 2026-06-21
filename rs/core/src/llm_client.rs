/// Lightweight multi-provider LLM client using ureq + SSE streaming.
///
/// Supports: OpenAI (GPT-4o), Anthropic (Claude), Google Gemini.
/// No async runtime needed — fully synchronous streaming via BufReader.
use std::io::{BufRead, BufReader};

/// Which LLM provider to use.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    Gemini,    // generativelanguage.googleapis.com
    OpenAI,    // api.openai.com
    Anthropic, // api.anthropic.com
    Ollama,    // localhost:11434 (OpenAI-compatible)
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
        Self {
            provider,
            api_key: api_key.to_string(),
            model,
            base_url,
        }
    }

    /// Build a quality-first fallback chain from available API keys.
    /// Order: Gemini → OpenAI → Anthropic → Ollama (local).
    pub fn fallback_chain(gemini_key: &str, openai_key: &str, anthropic_key: &str) -> Vec<Self> {
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
    let preference = [
        "gemini-3-flash",        // Best vision + speed, free tier (preview)
        "gemini-3.1-flash-lite", // Fastest TTFT, cheapest (preview)
        "gemini-2.5-flash",      // Stable fallback
        "gemini-3.1-pro",        // Best reasoning (preview)
        "gemini-2.5-pro",        // Stable pro fallback
    ];
    for pref in &preference {
        for m in models {
            let name = m["name"].as_str().unwrap_or("");
            // name is like "models/gemini-2.5-flash"
            let short = name.strip_prefix("models/").unwrap_or(name);
            if short == *pref || short.starts_with(*pref) {
                eprintln!("[CLX] llm: discovered Gemini model: {}", short);
                return Some(short.to_string());
            }
        }
    }
    // Fallback: return first model that supports generateContent.
    for m in models {
        let methods = m["supportedGenerationMethods"].as_array();
        if let Some(methods) = methods {
            if methods
                .iter()
                .any(|v| v.as_str() == Some("generateContent"))
            {
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
        .call()
        .ok()?;
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

    if models.is_empty() {
        return None;
    }

    let mut best: Option<(String, u64)> = None;
    for m in models {
        let name = m["name"].as_str().unwrap_or("");
        let size: u64 = name
            .split(':')
            .last()
            .unwrap_or("")
            .trim_end_matches('b')
            .trim_end_matches('B')
            .parse()
            .unwrap_or(0);
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
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

/// Stream a chat completion. Calls `on_token` for each streamed token.
/// Returns the full accumulated response.
#[cfg(all(not(target_arch = "wasm32"), feature = "ai"))]
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
    let msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect();

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
            if data == "[DONE]" {
                break;
            }
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
    let system = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let msgs: Vec<serde_json::Value> = messages
        .iter()
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
                if event["type"] == "message_stop" {
                    break;
                }
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
    let system_msg = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());

    let contents: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            let role = if m.role == "assistant" {
                "model"
            } else {
                "user"
            };
            serde_json::json!({"role": role, "parts": [{"text": m.content}]})
        })
        .collect();

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
    let base = config
        .base_url
        .as_deref()
        .unwrap_or("http://localhost:11434");
    let url = format!("{}/v1/chat/completions", base);

    let msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect();

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
            if data == "[DONE]" {
                break;
            }
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

#[cfg(any(target_arch = "wasm32", not(feature = "ai")))]
pub fn stream_chat(
    _config: &LlmConfig,
    _messages: &[Message],
    _on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    Err("LLM streaming disabled (build without `ai` feature, or running on WASM)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn provider_detected_from_anthropic_key_prefix() {
        let cfg = LlmConfig::from_key_and_model("sk-ant-abc", "some-model");
        assert_eq!(cfg.provider, LlmProvider::Anthropic);
    }

    #[test]
    fn provider_detected_from_claude_model_name() {
        let cfg = LlmConfig::from_key_and_model("xxx", "claude-opus-4");
        assert_eq!(cfg.provider, LlmProvider::Anthropic);
    }

    #[test]
    fn provider_detected_from_aiza_key_prefix() {
        let cfg = LlmConfig::from_key_and_model("AIzaSyFAKE", "anything");
        assert_eq!(cfg.provider, LlmProvider::Gemini);
    }

    #[test]
    fn provider_detected_from_gemini_model_name() {
        let cfg = LlmConfig::from_key_and_model("zzz", "gemini-2.5-flash");
        assert_eq!(cfg.provider, LlmProvider::Gemini);
    }

    #[test]
    fn provider_detected_as_ollama_when_key_empty() {
        let cfg = LlmConfig::from_key_and_model("", "qwen3:32b");
        assert_eq!(cfg.provider, LlmProvider::Ollama);
        assert!(cfg.base_url.is_some());
    }

    #[test]
    fn provider_detected_as_ollama_when_model_has_slash() {
        let cfg = LlmConfig::from_key_and_model("anykey", "mlx/qwen");
        assert_eq!(cfg.provider, LlmProvider::Ollama);
    }

    #[test]
    fn provider_detected_as_ollama_when_key_is_ollama() {
        let cfg = LlmConfig::from_key_and_model("ollama", "qwen3:32b");
        assert_eq!(cfg.provider, LlmProvider::Ollama);
    }

    #[test]
    fn provider_defaults_to_openai_for_generic_key() {
        let cfg = LlmConfig::from_key_and_model("sk-proj-randomkey", "gpt-4o");
        assert_eq!(cfg.provider, LlmProvider::OpenAI);
    }

    #[test]
    fn explicit_model_is_preserved() {
        let cfg = LlmConfig::from_key_and_model("sk-ant-x", "claude-3-5-sonnet");
        assert_eq!(cfg.model, "claude-3-5-sonnet");
    }

    #[test]
    fn base_url_only_set_for_ollama() {
        let cfg = LlmConfig::from_key_and_model("sk-ant-x", "claude-x");
        assert!(cfg.base_url.is_none());
        let cfg2 = LlmConfig::from_key_and_model("AIzaXYZ", "gemini-2.5-flash");
        assert!(cfg2.base_url.is_none());
    }

    #[test]
    fn fallback_chain_includes_ollama_last() {
        let chain = LlmConfig::fallback_chain("", "", "");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].provider, LlmProvider::Ollama);
    }

    #[test]
    fn fallback_chain_orders_providers_correctly() {
        let chain = LlmConfig::fallback_chain("AIzaKey", "sk-openaikey", "sk-ant-key");
        assert_eq!(chain.len(), 4);
        assert_eq!(chain[0].provider, LlmProvider::Gemini);
        assert_eq!(chain[1].provider, LlmProvider::OpenAI);
        assert_eq!(chain[2].provider, LlmProvider::Anthropic);
        assert_eq!(chain[3].provider, LlmProvider::Ollama);
    }

    #[test]
    fn fallback_chain_skips_empty_keys() {
        let chain = LlmConfig::fallback_chain("AIzaKey", "", "sk-ant-key");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].provider, LlmProvider::Gemini);
        assert_eq!(chain[1].provider, LlmProvider::Anthropic);
        assert_eq!(chain[2].provider, LlmProvider::Ollama);
    }

    #[test]
    fn message_clone_preserves_fields() {
        let m = msg("user", "hello");
        let c = m.clone();
        assert_eq!(c.role, "user");
        assert_eq!(c.content, "hello");
    }

    #[test]
    fn provider_debug_and_clone_work() {
        let p = LlmProvider::Gemini;
        let p2 = p.clone();
        assert_eq!(p, p2);
        let s = format!("{:?}", p);
        assert!(s.contains("Gemini"));
    }

    #[test]
    fn config_debug_and_clone_work() {
        let cfg = LlmConfig::from_key_and_model("AIzaX", "gemini-2.5-flash");
        let cfg2 = cfg.clone();
        assert_eq!(cfg.provider, cfg2.provider);
        let _ = format!("{:?}", cfg);
    }

    #[test]
    fn openai_sse_chunk_parses_delta_content() {
        let data = r#"{"choices":[{"delta":{"content":"hello"}}]}"#;
        let chunk: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(
            chunk["choices"][0]["delta"]["content"].as_str(),
            Some("hello")
        );
    }

    #[test]
    fn anthropic_sse_event_parses_content_block_delta() {
        let data = r#"{"type":"content_block_delta","delta":{"text":"world"}}"#;
        let event: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(event["type"], "content_block_delta");
        assert_eq!(event["delta"]["text"].as_str(), Some("world"));
    }

    #[test]
    fn gemini_sse_chunk_parses_candidates_text() {
        let data = r#"{"candidates":[{"content":{"parts":[{"text":"hi"}]}}]}"#;
        let chunk: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(
            chunk["candidates"][0]["content"]["parts"][0]["text"].as_str(),
            Some("hi"),
        );
    }

    #[test]
    fn openai_done_marker_recognized() {
        let line = "data: [DONE]";
        let stripped = line.strip_prefix("data: ").unwrap();
        assert_eq!(stripped, "[DONE]");
    }

    #[test]
    fn openai_request_body_shape() {
        let messages = vec![msg("system", "be brief"), msg("user", "hi")];
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let body = serde_json::json!({
            "model": "gpt-4o",
            "messages": msgs,
            "stream": true,
        });
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "hi");
    }

    #[test]
    fn anthropic_request_body_separates_system_field() {
        let messages = vec![
            msg("system", "you are helpful"),
            msg("user", "hello"),
            msg("assistant", "hi back"),
        ];
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let mut body = serde_json::json!({
            "model": "claude-x",
            "messages": msgs,
            "max_tokens": 4096,
            "stream": true,
        });
        if !system.is_empty() {
            body["system"] = serde_json::Value::String(system);
        }
        assert_eq!(body["system"], "you are helpful");
        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["max_tokens"], 4096);
    }

    #[test]
    fn anthropic_body_omits_system_when_absent() {
        let messages = vec![msg("user", "hello")];
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();
        let mut body = serde_json::json!({
            "model": "claude-x",
            "messages": msgs,
            "max_tokens": 4096,
            "stream": true,
        });
        if !system.is_empty() {
            body["system"] = serde_json::Value::String(system);
        }
        assert!(body.get("system").is_none());
    }

    #[test]
    fn gemini_request_body_uses_contents_and_role_mapping() {
        let messages = vec![
            msg("system", "be brief"),
            msg("user", "hi"),
            msg("assistant", "hello"),
        ];
        let system_msg = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());
        let contents: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                let role = if m.role == "assistant" {
                    "model"
                } else {
                    "user"
                };
                serde_json::json!({"role": role, "parts": [{"text": m.content}]})
            })
            .collect();
        let mut body = serde_json::json!({ "contents": contents });
        if let Some(sys) = system_msg {
            body["systemInstruction"] = serde_json::json!({"parts": [{"text": sys}]});
        }
        assert_eq!(body["contents"].as_array().unwrap().len(), 2);
        assert_eq!(body["contents"][0]["role"], "user");
        assert_eq!(body["contents"][1]["role"], "model");
        assert_eq!(body["contents"][0]["parts"][0]["text"], "hi");
        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "be brief");
    }

    #[test]
    fn ollama_url_uses_base_url_override() {
        let cfg = LlmConfig {
            provider: LlmProvider::Ollama,
            api_key: "ollama".to_string(),
            model: "qwen3:32b".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
        };
        let base = cfg.base_url.as_deref().unwrap_or("http://localhost:11434");
        let url = format!("{}/v1/chat/completions", base);
        assert_eq!(url, "http://localhost:11434/v1/chat/completions");
    }

    #[test]
    fn ollama_falls_back_to_default_base_when_none() {
        let cfg = LlmConfig {
            provider: LlmProvider::Ollama,
            api_key: String::new(),
            model: "x".to_string(),
            base_url: None,
        };
        let base = cfg.base_url.as_deref().unwrap_or("http://localhost:11434");
        assert_eq!(base, "http://localhost:11434");
    }

    #[test]
    fn gemini_url_construction_includes_model_and_key() {
        let model = "gemini-2.5-flash";
        let key = "AIzaTEST";
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            model, key
        );
        assert!(url.contains("gemini-2.5-flash:streamGenerateContent"));
        assert!(url.contains("alt=sse"));
        assert!(url.contains("key=AIzaTEST"));
    }

    #[test]
    fn stream_chat_returns_error_on_unreachable_openai() {
        let cfg = LlmConfig {
            provider: LlmProvider::Ollama,
            api_key: String::new(),
            model: "x".to_string(),
            base_url: Some("http://127.0.0.1:1".to_string()),
        };
        let messages = vec![msg("user", "hi")];
        let mut tokens = String::new();
        let res = stream_chat(&cfg, &messages, &mut |t| tokens.push_str(t));
        // Just verify it errors on an unreachable endpoint — don't pin the
        // exact message, which has changed shape between provider revamps.
        assert!(
            res.is_err(),
            "expected stream_chat to fail on unreachable endpoint"
        );
    }

    #[test]
    fn sse_line_without_data_prefix_is_skipped() {
        let line = "event: ping";
        assert!(line.strip_prefix("data: ").is_none());
    }

    #[test]
    fn sse_invalid_json_payload_does_not_panic() {
        let data = "not-json";
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(data);
        assert!(parsed.is_err());
    }

    #[test]
    fn anthropic_message_stop_event_recognized() {
        let data = r#"{"type":"message_stop"}"#;
        let event: serde_json::Value = serde_json::from_str(data).unwrap();
        assert_eq!(event["type"], "message_stop");
    }

    #[test]
    fn role_mapping_assistant_to_model_for_gemini() {
        let role_in = "assistant";
        let role_out = if role_in == "assistant" {
            "model"
        } else {
            "user"
        };
        assert_eq!(role_out, "model");
        let role_in = "user";
        let role_out = if role_in == "assistant" {
            "model"
        } else {
            "user"
        };
        assert_eq!(role_out, "user");
    }
}
