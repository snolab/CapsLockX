# Local Brainstorm Backends — Dropping the Ollama Dependency

**Status:** design note · **Scope:** the text LLM behind `clx+B` (brainstorm) on
Windows/macOS · **Question:** can we run local-first *without* requiring the user
to install Ollama, and what do the alternative runtimes actually cost?

> This is about the **text** LLM (chat/brainstorm). For the **vision/agent**
> model zoo (screenshots → actions), see [LOCAL-AGENT-MODELS.md](/agent/LOCAL-AGENT-MODELS.md).

---

## TL;DR

- **Runtime ≠ weights.** The inference *library* is single-digit MB. The *model
  weights* are 1–4 GB. Whatever backend we pick, the download is dominated by
  weights — so "embed the runtime" saves the **install step and background
  server**, not disk.
- Today `clx+B` shells out to the **Ollama CLI** (`local_llm.rs`) and talks HTTP
  to `localhost:11434`. It's the only feature with an external-process dependency.
- Your **voice/STT stack already embeds a local runtime in-process**
  (`sherpa-onnx` / ONNX Runtime in `lib/otoji`). So "self-contained local
  inference, no server" is a pattern the repo already ships — just for speech,
  not text.
- **Recommendation:** add an in-process **llama.cpp (`llama-cpp-2`) GGUF
  provider** as the default, keep Ollama as an optional fast-path. ~+3 MB to the
  binary, no install, no server, reuses the existing model-sizing logic.

---

## The two sizes people conflate

| Layer | What it is | Typical size | Ships how |
|---|---|---|---|
| **Inference runtime** | the engine linked into / beside `clx.exe` | **1–20 MB** | compiled in, or a DLL |
| **Model weights** | the actual parameters | **0.4–4.5 GB** | downloaded once, cached on disk |

Everything below optimizes the *runtime* row. The *weights* row is essentially
fixed by the model you pick (next table) and is the same across all backends.

### Weight sizes (Qwen2.5, Q4 quant — same for any GGUF/ONNX backend)

| Model | Q4 on disk | RAM to run | Tier (by `recommend_model()`) |
|---|---|---|---|
| Qwen2.5-0.5B | ~0.4 GB | ~1 GB | (below floor) |
| Qwen2.5-1.5B | ~1.0 GB | ~2 GB | < 8 GB RAM |
| Qwen2.5-3B | ~2.0 GB | ~3–4 GB | 8–15 GB |
| Qwen2.5-7B | ~4.5 GB | ~6–8 GB | 16–31 GB |
| Qwen2.5-14B | ~9 GB | ~12 GB | 32–63 GB |
| Qwen2.5-32B | ~20 GB | ~24 GB | 64 GB+ |

CLX already tiers these by RAM/VRAM in `core/src/local_llm.rs::recommend_model()`
— that logic is backend-agnostic and stays as-is; only the tag format changes
(`qwen2.5:3b` → `qwen2.5-3b-instruct-q4_k_m.gguf`).

---

## Solution matrix — backends for local `clx+B`

Ratings are for **text chat/brainstorm**, Windows-first, on typical consumer
hardware. "Bin +" = added to `clx.exe`. "Install?" = does the *user* have to
install anything.

| # | Backend | Bin + | User install? | Server proc? | GPU accel | Model format | Text-LLM maturity | Maint. burden | License | Verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| **A** | **Ollama CLI** (status quo) | 0 MB | **Yes** (~700 MB–1 GB) | Yes (`ollama serve`) | ✅ CUDA/ROCm/Metal | GGUF | ★★★★ | ★ trivial | MIT | Works today; heavy install, extra process |
| **B** | **Bundle `ollama.exe`** | +200–900 MB | No | Yes (we spawn it) | ✅ | GGUF | ★★★★ | ★★ redistribution | MIT | Turnkey but bloats installer; ships their GPU runners |
| **C** | **Embed llama.cpp** (`llama-cpp-2`) | **+2–5 MB** | **No** | **No** (in-proc) | ✅ opt-in (CUDA/Vulkan/Metal build) | GGUF | ★★★★ | ★★ C++ build/link | MIT | **Recommended** — smallest, mature, self-contained |
| **D** | **Embed candle** (pure Rust) | +3–8 MB | No | No (in-proc) | ⚠️ CUDA/Metal; no Vulkan | safetensors/GGUF | ★★★ | ★★ pure-Rust, easy CI | Apache/MIT | No C++ toolchain; fewer quant/kernel options |
| **E** | **Reuse ONNX Runtime** (already shipped for STT) + onnxruntime-genai | +0 MB* | No | No (in-proc) | ✅ DirectML/CUDA | ONNX | ★★ | ★★★ conversion pain | MIT | Reuses STT dep, but LLM path immature + awkward weights |
| **F** | **mistral.rs** | +5–12 MB | No | Optional | ✅ | GGUF/safetensors | ★★★ | ★★★ heavier dep tree | MIT | Nice features (ISQ, paged attn) but heavier than C/D |
| **G** | **Cloud fallback only** (no local) | 0 MB | No | No | n/a | n/a | ★★★★ | ★ trivial | — | Not local/private; already the online fallback path |

\* ONNX Runtime is *already* compiled into your build via `sherpa-onnx`; the
marginal cost is onnxruntime-genai glue + a DirectML provider, not a new base
runtime. The catch is model conversion and weight availability, not binary size.

### Runtime library sizes (engine only, no weights)

| Runtime | Added to binary | GPU story on Windows | Notes |
|---|---|---|---|
| llama.cpp (static, CPU) | ~2–5 MB | rebuild with CUDA or **Vulkan** feature | most mature GGUF text runtime |
| candle | ~3–8 MB | CUDA / Metal only (no Vulkan) | pure Rust, trivial cross-compile |
| ONNX Runtime (CPU DLL) | ~10–20 MB | **DirectML** = works on any GPU (incl. AMD/Intel) | you already ship it for STT |
| mistral.rs | ~5–12 MB | CUDA / Metal | ISQ in-situ quant, more deps |
| **Ollama (installed)** | ~700 MB–1 GB | bundles CUDA + ROCm runners | that's *why* it's huge |

---

## Why not just reuse the ONNX Runtime we already ship? (Option E)

Tempting — `sherpa-onnx` already links ONNX Runtime for STT, so the base engine
is "free." But for a **text LLM** specifically:

- **Weights are awkward.** LLMs are distributed as GGUF/safetensors. Running one
  through ORT means converting to ONNX + a separate `.onnx.data` blob, and the
  ecosystem for that (onnxruntime-genai) covers far fewer models than GGUF.
- **KV-cache/decoding** for autoregressive text is a solved, batteries-included
  thing in llama.cpp; in ORT you lean on onnxruntime-genai, which is newer.
- **Upside worth noting:** ORT's **DirectML** provider runs on *any* Windows GPU
  (AMD, Intel, NVIDIA) with zero user setup — llama.cpp needs a Vulkan/CUDA
  build to match that. If broad GPU coverage on Windows becomes a priority, E is
  the dark-horse option. For CPU/NVIDIA-first, C wins.

**STT (sherpa-onnx) stays on ONNX regardless** — this note only concerns the
text brainstorm path.

---

## Decision matrix — pick by constraint

| If the must-have is… | Pick | Why |
|---|---|---|
| **Zero user install, smallest binary** | **C · llama.cpp** | +3 MB, no server, mature GGUF |
| Pure-Rust CI, no C++ toolchain | D · candle | easiest cross-compile, safetensors |
| Broad Windows GPU support (AMD/Intel too) | E · ORT + DirectML | reuses STT dep, any-GPU via DML |
| Ship nothing new, fastest to keep working | A · Ollama CLI | already implemented |
| One-click installer, don't care about size | B · bundle Ollama | turnkey, but +hundreds of MB |
| Privacy off / best quality, HW-light | G · cloud fallback | existing online path |

---

## Recommended path

1. **Add a `LlmProvider::LocalGguf` backend** behind the existing enum, powered
   by `llama-cpp-2`. In-process, no server.
2. **Reuse `recommend_model()`** verbatim — just map tiers to GGUF filenames.
3. **First-run flow** = same UX as today's Ollama wizard, minus the install:
   download the GGUF (~1–2 GB for the 1.5B/3B tier) into
   `%APPDATA%\CapsLockX\models\` with a progress overlay.
4. **`local_status()`** becomes "is the weights file present & loadable?" instead
   of "is the server reachable?".
5. **Keep Ollama as an optional fast-path**: if it's already running, use it
   (the local-first routing in `state.rs::brainstorm_llm_key_and_model()` already
   prefers local). New installs need nothing.
6. **Gate GPU** behind a build feature (`vulkan`/`cuda`) so the default binary
   stays tiny and CPU-portable; power users opt into an accelerated build.

Net effect: `clx` becomes a **single self-contained binary** with local AI and
no external install — the same philosophy the voice module already proves with
`sherpa-onnx`.

---

## Implementation status (Option C — llama-cpp-2)

The recommended path is implemented in `rs/core`, behind the off-by-default
`local-llm` cargo feature (built & tested against **llama-cpp-2 0.1.139**):

| Piece | Where | What it does |
|---|---|---|
| `LlmProvider::LocalGguf` | `llm_client.rs` | new provider; GGUF path carried in `base_url` |
| `LlmConfig::local_gguf(path)` + auto-detect | `llm_client.rs` | a `*.gguf` model or `local` key routes in-process (Windows `\` paths handled) |
| `stream_gguf()` + `format_chatml()` | `local_gguf.rs` | streams tokens via llama.cpp; Qwen2.5 ChatML prompt |
| **model cache** (`get_or_load_model`) | `local_gguf.rs` | each multi-GB model stays resident across turns; only the first `clx+B` pays the load |
| `recommend_gguf()` / `download_gguf()` / `models_dir()` | `local_llm.rs` | tier → weight file + streaming HF downloader (atomic `.part` rename) |
| **default routing** | `state.rs` | `prefer_local` + `local-llm` → `("local", <gguf path>)`; falls back to Ollama without the feature |
| **first-run download gate** | `modules/brainstorm.rs` | downloads the hardware-sized GGUF on first `clx+B` with % progress in the overlay; concurrency-guarded |
| dispatch in `stream_chat` + `agent_chat` | `llm_client.rs` / `agent.rs` | local runs as a **plain streaming turn** (no tool-calling — small models don't follow it reliably) |

### Build variants

```bash
cargo build -p capslockx-core --features local-llm          # CPU (default local)
cargo build -p capslockx-core --features local-llm-cuda     # NVIDIA
cargo build -p capslockx-core --features local-llm-vulkan   # any GPU (AMD/Intel/NVIDIA)
cargo build -p capslockx-core --features local-llm-metal    # Apple Silicon
```
GPU builds set `n_gpu_layers` to offload the whole model (llama.cpp clamps to
fit). `rocm` is also available on llama-cpp-2 if an AMD/ROCm variant is wanted.

Verified: default build (feature off) stays green; feature build compiles the
native llama.cpp lib + all API usage; 11 unit tests cover the ChatML formatter,
provider detection, GGUF tiering, and default routing.

### Remaining follow-ups

- **Per-token CJK streaming** — uses the deprecated `token_to_str`; the
  incremental `token_to_piece(&mut Decoder, …)` preserves multi-byte UTF-8
  across token boundaries (matters for zh/ja). `TODO`.
- **Greedy sampling** — deterministic; swap for a temp/top_p/dist chain.
- **First-run UX** — the download currently runs before the prompt box on the
  very first `clx+B`; could move to the Tauri setup wizard for a nicer flow.
- **32B tier** GGUF is multipart on HF, so its single-file URL 404s — fall back
  to the repo (or pick a single-file quant) for that tier.

## Open questions

- **GGUF fetch source & integrity** — pull from Hugging Face with a pinned
  SHA256? Mirror for CN users (like the Google-Translate CN fallback in the docs
  site)?
- **First-token latency** on CPU for 3B Q4 — acceptable for brainstorm (turn-based)
  but measure before defaulting a tier.
- **Vulkan vs CPU default** on Windows — does an auto GPU-probe (`nvidia-smi`
  already exists in `local_llm.rs`) justify shipping a Vulkan build by default?

---

## See also

- [LOCAL-AGENT-MODELS.md](/agent/LOCAL-AGENT-MODELS.md) — vision/agent model zoo + inference stacks
- [WINDOWS-BRAINSTORM-LOCAL-FIRST.md](/agent/WINDOWS-BRAINSTORM-LOCAL-FIRST.md) — current Ollama-based design
- `core/src/local_llm.rs` — hardware probe, `recommend_model()`, Ollama bootstrap
- `core/src/modules/brainstorm.rs` — the `clx+B` turn loop + readiness gate
