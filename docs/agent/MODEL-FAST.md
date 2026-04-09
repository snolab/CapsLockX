# MODEL-FAST — High-Speed Streaming Models

Models optimized for raw throughput and minimal time-to-first-token.
Used as the **primary command generation engine** — they receive
structured sensor data and stream CLX commands in real-time.

## Cloud Providers — Speed Leaderboard

### Custom Silicon (fastest)

| Rank | Provider | Model | tok/s | TTFT | API |
|---|---|---|---|---|---|
| 1 | **Cerebras** | Llama 3.1 8B | 1000-2000 | ~100ms | OpenAI-compat |
| 2 | **Cerebras** | Llama 3.1 70B | 400-800 | ~150ms | OpenAI-compat |
| 3 | **Groq** | Llama 3.1 8B | 700-1000 | ~100ms | OpenAI-compat |
| 4 | **Groq** | Llama 3.3 70B | 300-350 | ~200ms | OpenAI-compat |
| 5 | **SambaNova** | Llama 3.1 8B | 300-1000 | ~100ms | OpenAI-compat |

**Why custom silicon matters**: Cerebras (wafer-scale) and Groq (LPU) use
purpose-built hardware that delivers 5-10x the tok/s of GPU providers.
For streaming CLX commands where every token = latency, this is critical.

**Groq Llama 3.3 70B** is the sweet spot: high quality (70B) at 300+ tok/s.
A typical CLX command is 3-8 tokens, so one command streams in ~10-25ms.

### GPU Inference Providers

| Provider | Model | tok/s | TTFT | Notes |
|---|---|---|---|---|
| Fireworks | Llama 3.1 8B | 200-400 | ~100ms | Speculative decoding |
| Together | Llama 3.1 8B Turbo | 200-400 | ~100ms | Custom kernels |
| Fireworks | Llama 3.1 70B | 100-200 | ~200ms | |
| Together | Llama 3.1 70B Turbo | 100-200 | ~200ms | |

### Proprietary Fast Models

| Provider | Model | tok/s | TTFT | Notes |
|---|---|---|---|---|
| Google | Gemini 2.0 Flash-Lite | 180-250 | ~100ms | Cheapest Gemini |
| Google | Gemini 2.0 Flash | 150-200 | ~150ms | Best proprietary balance |
| OpenAI | GPT-4.1-nano | 100-140 | ~150ms | Smallest GPT-4.1 |
| OpenAI | GPT-4o-mini | 80-120 | ~200ms | |
| Anthropic | Claude 3.5 Haiku | 80-120 | ~300ms | |

## Local Models (zero network latency)

Best for offline use, privacy, or ultra-low TTFT.

| Model | Params | Q4_K_M size | tok/s (M2) | tok/s (4090) | Quality |
|---|---|---|---|---|---|
| Llama 3.2 1B | 1B | 0.7 GB | 80-120 | 150+ | Basic |
| SmolLM2 1.7B | 1.7B | 1.0 GB | 60-100 | 130+ | Basic+ |
| Llama 3.2 3B | 3B | 1.8 GB | 40-70 | 100-150 | Decent |
| Phi-3.5 Mini | 3.8B | 2.2 GB | 35-60 | 80-120 | Good |
| Gemma 2 2B | 2.6B | 1.5 GB | 50-80 | 120+ | Decent |
| Qwen 2.5 3B | 3B | 1.8 GB | 40-70 | 100-140 | Good |
| Qwen 2.5 7B | 7B | 4.4 GB | 20-35 | 60-90 | Very good |
| Llama 3.1 8B | 8B | 4.9 GB | 15-30 | 50-80 | Very good |
| Mistral 7B v0.3 | 7B | 4.4 GB | 20-35 | 60-90 | Good |

**TTFT for local**: 50-200ms (no network). Dominated by prompt processing.

### Local model recommendations
- **Autocomplete-level speed** (< 100ms TTFT): Llama 3.2 1B-3B
- **Best quality locally**: Qwen 2.5 7B Q4_K_M
- **Best quality/speed tradeoff**: Llama 3.2 3B or Phi-3.5 Mini

### Running locally
- **llama.cpp** / **Ollama**: Best for Apple Silicon (Metal backend)
- **vLLM**: Best for NVIDIA GPUs (CUDA, paged attention)
- **GGUF format**: Standard for llama.cpp, Q4_K_M quantization recommended

## Key Metrics for Agent Use

### What matters most
1. **TTFT** (time to first token): Determines how fast the first command arrives
2. **tok/s** (throughput): Determines command stream rate during sustained output
3. **Instruction following**: Must reliably output CLX syntax, not prose

### CLX command token costs
| Command | Example | Tokens |
|---|---|---|
| Key tap | `k a\n` | 2-3 |
| Key combo | `k c-s\n` | 3-4 |
| Mouse move | `m 500 300\n` | 4-5 |
| Mouse click | `m c\n` | 2 |
| Pad stick | `p ls 0.5 -0.3\n` | 5-6 |
| Wait | `w 100ms\n` | 3 |
| Type string | `k "hello"\n` | 3-4 |
| Sense control | `S screen fps 5\n` | 5-6 |

At 300 tok/s (Groq 70B), that's **50-100 CLX commands per second** — far
more than any game needs.

### Instruction following quality
Small models (1-3B) may drift from CLX syntax and output prose.
Mitigations:
- Strong few-shot examples in system prompt
- `stop` sequences on common prose starters ("Sure", "I'll", "Let me")
- Post-processing: strip any line that doesn't match CLX grammar
- Fine-tuning on CLX command datasets (future)

## API Compatibility

All fast providers use **OpenAI-compatible APIs**:
```
POST /v1/chat/completions
{
  "model": "llama-3.3-70b",
  "messages": [...],
  "stream": true,
  "max_tokens": 200,
  "stop": ["\n\n"]   // stop on double newline (end of command block)
}
```

Endpoints:
- Groq: `api.groq.com/openai/v1/`
- Cerebras: `api.cerebras.ai/v1/`
- Fireworks: `api.fireworks.ai/inference/v1/`
- Together: `api.together.xyz/v1/`
- SambaNova: `api.sambanova.ai/v1/`

Switching between providers requires only changing the base URL and API key.

## Cost Comparison (per 1M input tokens / 1M output tokens)

| Provider | Model | Input $/M | Output $/M | Notes |
|---|---|---|---|---|
| Groq | Llama 3.3 70B | $0.59 | $0.79 | Free tier: 30 req/min |
| Cerebras | Llama 3.1 70B | $0.60 | $0.60 | |
| Fireworks | Llama 3.1 70B | $0.90 | $0.90 | |
| Together | Llama 3.1 70B | $0.88 | $0.88 | |
| Google | Gemini 2.0 Flash | $0.10 | $0.40 | Cheapest proprietary |
| Google | Flash-Lite | $0.075 | $0.30 | Cheapest overall cloud |
| OpenAI | GPT-4o-mini | $0.15 | $0.60 | |
| Local | Any | $0 | $0 | Hardware cost only |

At ~500 tok/s total sensor input + 100 tok/s command output, a 1-hour
gaming session costs roughly:
- Groq 70B: ~$1.50
- Gemini Flash: ~$0.30
- Local: $0

## References

- [2025-03] [Groq API docs & pricing](https://console.groq.com/docs/models) — Groq model list, speed benchmarks
- [2025-03] [Cerebras inference docs](https://www.cerebras.ai/product-cloud) — wafer-scale inference API
- [2025-01] [SambaNova Cloud](https://sambanova.ai/fast-api) — RDU inference benchmarks
- [2025-03] [Gemini API pricing](https://ai.google.dev/pricing) — Google AI token pricing
- [2025-04] [GPT-4.1 model card](https://openai.com/index/gpt-4-1/) — OpenAI model specs
- [2025-01] [Together AI docs](https://docs.together.ai/) — model list, Turbo variants
- [2025-01] [Fireworks AI docs](https://docs.fireworks.ai/) — speculative decoding, benchmarks
- [2024-12] [Llama 3.3 70B](https://ai.meta.com/blog/llama-3-3/) — Meta model release
- [2024-09] [Qwen 2.5 release](https://qwenlm.github.io/blog/qwen2.5/) — Qwen model family
- [2025-02] [llama.cpp benchmarks](https://github.com/ggerganov/llama.cpp/discussions/4225) — local inference speeds
- [2024-07] [SmolLM2](https://huggingface.co/HuggingFaceTB/SmolLM2-1.7B) — HuggingFace edge model
