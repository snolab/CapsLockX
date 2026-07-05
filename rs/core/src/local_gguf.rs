//! In-process GGUF text generation via llama.cpp (`llama-cpp-2`).
//!
//! Prototype backend for local-first `clx+B` that runs **entirely in-process**
//! with no Ollama install and no background server — the same "embed the
//! runtime, download only the weights" pattern the voice module already uses
//! with `sherpa-onnx`. See `docs/dev/local-brainstorm-backends.md`.
//!
//! Layering mirrors `local_llm.rs`:
//!   * **pure** — `format_chatml()` builds the Qwen2.5 ChatML prompt. No I/O,
//!     unit-tested, always compiled.
//!   * **impure** — `stream_gguf()` loads a GGUF and streams tokens. Gated
//!     behind the `local-llm` cargo feature (which pulls in `llama-cpp-2` +
//!     a C++ toolchain). When the feature is off, a stub returns a clear error
//!     so callers still link.
//!
//! Enable with: `cargo build -p capslockx-core --features local-llm`
//! (GPU offload is a follow-up sub-feature; this prototype is CPU-only.)

use crate::llm_client::Message;

/// Render chat history into Qwen2.5 ChatML — the prompt format the Instruct
/// GGUFs are trained on. A trailing `<|im_start|>assistant\n` primes generation.
///
/// Kept separate from the llama.cpp call so it can be tested without the
/// `local-llm` feature (and reused if we swap runtimes to candle/mistral.rs).
pub fn format_chatml(messages: &[Message]) -> String {
    let mut s = String::new();
    for m in messages {
        // Roles map 1:1 to ChatML; unknown roles fall back to "user".
        let role = match m.role.as_str() {
            "system" | "user" | "assistant" => m.role.as_str(),
            _ => "user",
        };
        s.push_str("<|im_start|>");
        s.push_str(role);
        s.push('\n');
        s.push_str(m.content.trim_end());
        s.push_str("<|im_end|>\n");
    }
    // Prime the model to answer.
    s.push_str("<|im_start|>assistant\n");
    s
}

/// Upper bound on generated tokens per turn (brainstorm answers are short).
const MAX_NEW_TOKENS: usize = 1024;
/// Context window to allocate. 4k covers brainstorm history comfortably.
const N_CTX: u32 = 4096;

// ── Real implementation (feature = "local-llm") ──────────────────────────────

/// The one process-global llama.cpp backend (init is only valid once).
#[cfg(all(feature = "local-llm", not(target_arch = "wasm32")))]
fn backend() -> Result<&'static llama_cpp_2::llama_backend::LlamaBackend, String> {
    use llama_cpp_2::llama_backend::LlamaBackend;
    use once_cell::sync::OnceCell;
    static BACKEND: OnceCell<LlamaBackend> = OnceCell::new();
    BACKEND
        .get_or_try_init(LlamaBackend::init)
        .map_err(|e| format!("llama backend init: {e}"))
}

/// Loaded-model cache, process-global. Keeps each multi-GB `LlamaModel`
/// resident across turns keyed by path, so only the first `clx+B` after launch
/// pays the load cost (later turns just build a fresh context). Serialised
/// behind a `Mutex` — brainstorm is single-user.
#[cfg(all(feature = "local-llm", not(target_arch = "wasm32")))]
fn get_or_load_model(path: &str) -> Result<std::sync::Arc<llama_cpp_2::model::LlamaModel>, String> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use llama_cpp_2::model::params::LlamaModelParams;
    use llama_cpp_2::model::LlamaModel;
    use once_cell::sync::OnceCell;

    static MODELS: OnceCell<Mutex<HashMap<String, Arc<LlamaModel>>>> = OnceCell::new();

    let backend = backend()?;

    let cache = MODELS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache
        .lock()
        .map_err(|_| "model cache poisoned".to_string())?;
    if let Some(m) = guard.get(path) {
        return Ok(Arc::clone(m));
    }

    if !std::path::Path::new(path).exists() {
        return Err(format!("GGUF model not found: {path}"));
    }

    // GPU offload when built with a GPU sub-feature; CPU-only otherwise. A large
    // layer count offloads the whole model (llama.cpp clamps to what fits).
    #[allow(unused_mut)]
    let mut model_params = LlamaModelParams::default();
    #[cfg(any(
        feature = "local-llm-cuda",
        feature = "local-llm-vulkan",
        feature = "local-llm-metal"
    ))]
    {
        model_params = model_params.with_n_gpu_layers(1_000_000);
    }

    let model = LlamaModel::load_from_file(backend, path, &model_params)
        .map_err(|e| format!("load {path}: {e}"))?;
    let arc = Arc::new(model);
    guard.insert(path.to_string(), Arc::clone(&arc));
    Ok(arc)
}

#[cfg(all(feature = "local-llm", not(target_arch = "wasm32")))]
pub fn stream_gguf(
    model_path: &str,
    messages: &[Message],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    use std::num::NonZeroU32;

    use llama_cpp_2::context::params::LlamaContextParams;
    use llama_cpp_2::llama_batch::LlamaBatch;
    use llama_cpp_2::model::AddBos;
    #[allow(deprecated)]
    use llama_cpp_2::model::Special;
    use llama_cpp_2::sampling::LlamaSampler;

    // Cached model load (see `get_or_load_model`) — cheap after the first turn.
    let model = get_or_load_model(model_path)?;
    let backend = backend()?;

    let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(N_CTX));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("create context: {e}"))?;

    // Tokenize the ChatML prompt.
    let prompt = format_chatml(messages);
    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| format!("tokenize: {e}"))?;

    // Feed the prompt in one batch; only the last token requests logits.
    let mut batch = LlamaBatch::new(512.max(tokens.len()), 1);
    let last = tokens.len().saturating_sub(1);
    for (i, tok) in tokens.iter().enumerate() {
        batch
            .add(*tok, i as i32, &[0], i == last)
            .map_err(|e| format!("batch add: {e}"))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| format!("decode prompt: {e}"))?;

    // Greedy decode keeps the prototype deterministic; swap for a
    // temp/top_p/dist chain for more varied brainstorm output.
    let mut sampler = LlamaSampler::greedy();

    let mut n_cur = batch.n_tokens();
    let mut out = String::new();

    for _ in 0..MAX_NEW_TOKENS {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        // `token_to_str` is deprecated in favour of an incremental
        // `token_to_piece(&mut Decoder, …)` that preserves multi-byte UTF-8
        // across token boundaries — a TODO for clean CJK streaming. The simple
        // form is fine for the prototype.
        #[allow(deprecated)]
        let piece = model
            .token_to_str(token, Special::Tokenize)
            .map_err(|e| format!("detokenize: {e}"))?;
        out.push_str(&piece);
        on_token(&piece);

        // Feed the sampled token back in for the next step.
        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| format!("batch add: {e}"))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| format!("decode step: {e}"))?;
    }

    Ok(out)
}

// ── Stub (feature off / wasm) ────────────────────────────────────────────────

#[cfg(not(all(feature = "local-llm", not(target_arch = "wasm32"))))]
pub fn stream_gguf(
    _model_path: &str,
    _messages: &[Message],
    _on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    Err("local GGUF backend not built — rebuild with `--features local-llm`".into())
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
    fn chatml_wraps_each_role_and_primes_assistant() {
        let out = format_chatml(&[msg("system", "be brief"), msg("user", "hi")]);
        assert_eq!(
            out,
            "<|im_start|>system\nbe brief<|im_end|>\n\
             <|im_start|>user\nhi<|im_end|>\n\
             <|im_start|>assistant\n"
        );
    }

    #[test]
    fn chatml_ends_by_priming_the_assistant_turn() {
        let out = format_chatml(&[msg("user", "hello")]);
        assert!(out.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn chatml_unknown_role_falls_back_to_user() {
        let out = format_chatml(&[msg("tool", "result")]);
        assert!(out.contains("<|im_start|>user\nresult<|im_end|>"));
    }

    #[test]
    fn chatml_preserves_multiturn_order() {
        let out = format_chatml(&[msg("user", "q1"), msg("assistant", "a1"), msg("user", "q2")]);
        let q1 = out.find("q1").unwrap();
        let a1 = out.find("a1").unwrap();
        let q2 = out.find("q2").unwrap();
        assert!(q1 < a1 && a1 < q2);
    }

    #[test]
    fn stub_errors_when_feature_disabled() {
        // With `local-llm` off this asserts the stub path; with it on the call
        // would try to load a real model, so only check the disabled build.
        #[cfg(not(feature = "local-llm"))]
        {
            let mut sink = String::new();
            let r = stream_gguf("nonexistent.gguf", &[msg("user", "hi")], &mut |t| {
                sink.push_str(t)
            });
            assert!(r.is_err());
        }
    }
}
