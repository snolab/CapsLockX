# Local Multimodal Agent Models — Full Comparison

Open-source models that run **locally** (no API), accept **images natively**
(screenshots), and are either **trained for agentic tasks** or serve as a
**multimodal base** for agent scaffolds.

Goal: pick a self-hosted stack that matches your hardware and latency
budget. Replaces cloud `FAST` / `REASONING` models from `MODELS.md` for
privacy, zero-cost, or air-gapped use.

---

## TL;DR by Hardware

| Hardware | Recommended model | Role |
|---|---|---|
| CPU / M1 8 GB | ShowUI-2B (Q4) or MiniCPM-V 2.6 (Q4) | Slow turn-based agent |
| 12 GB GPU (3060) | Phi-3.5-V, ShowUI-2B, UI-TARS-2B | Light UI automation |
| 16 GB GPU (4060 Ti) | **UI-TARS-7B Q4**, Qwen2.5-VL-7B Q4 | **Default sweet spot** |
| 24 GB GPU (3090/4090) | UI-TARS-7B fp16, CogAgent-9B, Qwen2.5-VL-7B fp16 | Real-time agent |
| 2× 24 GB or 48 GB | Qwen2.5-VL-32B, InternVL3-38B, Aguvis-72B int4 | Reasoning-grade local |
| ≥ 80 GB (A100/H100) | UI-TARS-72B, Qwen2.5-VL-72B, Molmo-72B | SOTA open-source |

---

## Scoring Legend

- **Intelligence**: ★☆☆ weak · ★★☆ good · ★★★ strong · ★★★★ SOTA-open.
  Blends GUI-grounding (ScreenSpot), agentic (OSWorld/AndroidWorld),
  and general VLM (MMMU, MathVista) benchmarks. Qualitative.
- **Latency**: TTFT = time-to-first-token on one 1080p screenshot, single
  4090 at int4 unless noted. Throughput in decode tok/s.
- **VRAM**: int4 GGUF/AWQ first, fp16 in parens. int4 quality loss
  ≈ 1-3 points on benchmarks for 7B+ models.
- **License**: Apache-2.0 / MIT are friendliest; "Llama" / "research" have
  redistribution / commercial-use clauses.

---

## Class A — GUI-Native Agent Models

Post-trained on screenshot → action pairs. Output is a structured action
(click coords, type, scroll, key). Plug directly into CLX action parser.

| Model | Size | Intel. | TTFT | Tok/s | VRAM (int4 / fp16) | RAM (CPU) | License | Notes |
|---|---|---|---|---|---|---|---|---|
| **UI-TARS-1.5-2B** | 2B | ★★☆ | 120 ms | 140 | 2.5 / 5 GB | 4 GB | Apache-2.0 | Smallest GUI-native; edge-device viable. |
| **UI-TARS-1.5-7B** | 7B | ★★★ | 250 ms | 70 | 6 / 16 GB | 10 GB | Apache-2.0 | **Default pick.** Beats Claude Computer Use on OSWorld. |
| **UI-TARS-1.5-72B** | 72B | ★★★★ | 900 ms | 18 | 44 / 160 GB | 48 GB | Apache-2.0 | SOTA open; needs A100 or 2×4090. |
| **UI-TARS-2-7B** | 7B | ★★★+ | 280 ms | 65 | 6 / 16 GB | 10 GB | Apache-2.0 | Multi-turn RL; +5pt over 1.5 on OSWorld. |
| **UI-TARS-2-72B** | 72B | ★★★★ | 950 ms | 17 | 44 / 160 GB | 48 GB | Apache-2.0 | Current open SOTA (2025-09). |
| **CogAgent-9B-20241220** | 9B | ★★★ | 400 ms | 50 | 8 / 20 GB | 14 GB | Apache-2.0 | Zhipu/THU. Strong grounding + multi-step. |
| **OS-Atlas-Base-4B** | 4B | ★★☆ | 200 ms | 90 | 4 / 10 GB | 7 GB | Apache-2.0 | InternVL2 base. Good grounding backbone. |
| **OS-Atlas-Pro-7B** | 7B | ★★★ | 260 ms | 70 | 6 / 16 GB | 10 GB | Apache-2.0 | Qwen2-VL base. Cross-platform data. |
| **Aguvis-7B** | 7B | ★★★ | 260 ms | 70 | 6 / 16 GB | 10 GB | MIT | Unified action space web+desktop+mobile. |
| **Aguvis-72B** | 72B | ★★★★ | 900 ms | 18 | 44 / 160 GB | 48 GB | MIT | Built-in planning+grounding stages. |
| **ShowUI-2B** | 2B | ★★☆ | 150 ms | 120 | 2.5 / 5 GB | 4 GB | MIT | UI-guided token pruning → fast on low VRAM. |
| **SeeClick-9.6B** | 9.6B | ★★☆ | 420 ms | 45 | 9 / 22 GB | 14 GB | Apache-2.0 | Earlier baseline; still reproducible. |
| **OS-Genesis-7B** | 7B | ★★★ | 270 ms | 68 | 6 / 16 GB | 10 GB | Apache-2.0 | Trajectory-synthesis; long-horizon tasks. |
| **Octopus v4-7B** | 7B | ★★☆ | 260 ms | 80 | 6 / 16 GB | 10 GB | Apache-2.0 | Functional-token action output, efficient. |
| **Octopus v2-2B** | 2B | ★★☆ | 130 ms | 150 | 2.5 / 5 GB | 4 GB | Apache-2.0 | Gemma base; on-device. |
| **Ferret-UI 2** | 7B / 13B | ★★★ | 260 / 450 ms | 70 / 35 | 6–13 / 16–32 GB | 10–20 GB | research | Apple. Mobile UI focus; research license. |
| **UGround-V1** | 7B | ★★☆ | 260 ms | 70 | 6 / 16 GB | 10 GB | Apache-2.0 | Pure grounding model; pair with planner. |
| **AutoGLM-Web-9B** | 9B | ★★★ | 400 ms | 50 | 8 / 20 GB | 14 GB | Apache-2.0 | Zhipu; web-agent specialized. |

---

## Class B — Multimodal Bases with Agent Capability

General VLMs that **also** expose a `computer_use` / `tool_use` mode in the
same weights — one model for chat, vision, and action.

| Model | Size | Intel. | TTFT | Tok/s | VRAM (int4 / fp16) | License | Agent feature |
|---|---|---|---|---|---|---|---|
| **Qwen2.5-VL-3B** | 3B | ★★☆ | 180 ms | 100 | 3 / 7 GB | Apache-2.0 | `computer_use`/`mobile_use` tools built-in. |
| **Qwen2.5-VL-7B** | 7B | ★★★ | 300 ms | 65 | 6 / 16 GB | Apache-2.0 | **Single-model stack winner.** |
| **Qwen2.5-VL-32B** | 32B | ★★★+ | 600 ms | 30 | 20 / 70 GB | Apache-2.0 | Reasoning-grade; fits 24 GB int4 tight. |
| **Qwen2.5-VL-72B** | 72B | ★★★★ | 950 ms | 17 | 44 / 160 GB | Apache-2.0 | Matches GPT-4o on many VLM tasks. |
| **InternVL3-1B / 2B / 8B** | 1–8B | ★★–★★★ | 100–300 ms | 60–180 | 1–8 / 2–18 GB | MIT | Scales cleanly; agent variants exist. |
| **InternVL3-38B** | 38B | ★★★+ | 650 ms | 25 | 24 / 80 GB | MIT | Strong mid-size. |
| **InternVL3-78B** | 78B | ★★★★ | 1000 ms | 15 | 46 / 170 GB | MIT | 72.2 MMMU — top open-source. |
| **MiniCPM-V 2.6 / o-2.6** | 8B | ★★★ | 280 ms | 80 | 6 / 16 GB | Apache-2.0 | Phone-capable; OCR-strong. |
| **MiniCPM-V 4.0** (if shipped) | 4B | ★★★ | 180 ms | 110 | 3 / 8 GB | Apache-2.0 | Edge-optimized. |
| **Phi-3.5-Vision** | 4.2B | ★★☆ | 180 ms | 100 | 3 / 9 GB | MIT | Small, fast, grounding ok. |
| **Phi-4-multimodal** | 5.6B | ★★★ | 220 ms | 90 | 4 / 11 GB | MIT | Adds audio; agent-tunable. |
| **Molmo-1B** | 1B | ★★☆ | 100 ms | 200 | 1 / 2 GB | Apache-2.0 | AI2. Trained to **point** at pixels. |
| **Molmo-7B-D / O** | 7B | ★★★ | 300 ms | 70 | 6 / 16 GB | Apache-2.0 | Pointing VLM ≈ GPT-4V class. |
| **Molmo-72B** | 72B | ★★★★ | 950 ms | 18 | 44 / 160 GB | Apache-2.0 | ≈ Claude 3.5 Sonnet on VLM benches. |
| **Llama-3.2-Vision-11B** | 11B | ★★★ | 350 ms | 60 | 8 / 22 GB | Llama | Needs agent fine-tune/scaffold. |
| **Llama-3.2-Vision-90B** | 90B | ★★★★ | 1100 ms | 14 | 52 / 180 GB | Llama | Reasoning-grade; no native agent head. |
| **Pixtral-12B** | 12B | ★★★ | 380 ms | 55 | 9 / 24 GB | Apache-2.0 | Mistral. General VLM, no built-in agent. |
| **Idefics3-8B** | 8B | ★★★ | 300 ms | 70 | 6 / 16 GB | Apache-2.0 | HF; long-image handling. |
| **Gemma 3 (multimodal)** | 4B / 12B / 27B | ★★–★★★ | 180–500 ms | 30–110 | 3–18 / 9–60 GB | Gemma | Google; vision in 4B+. |
| **DeepSeek-VL2** | 4.5B / 16B / 27B MoE | ★★★ | 200–500 ms | 40–100 | 3–18 / 10–60 GB | MIT | MoE; active params small. |
| **GLM-4V-9B** | 9B | ★★★ | 350 ms | 55 | 7 / 18 GB | MIT-like | Zhipu; decent grounding. |
| **NVLM-D-72B** | 72B | ★★★★ | 950 ms | 18 | 44 / 160 GB | CC-BY-NC | NVIDIA; research only. |

---

## Class C — Perception / Grounding Helpers (non-agent, but useful)

These are **not** agents — they feed an agent. Pair with a Class A or B
model to reduce its vision work and improve latency.

| Model | Task | Size | Latency | Hardware |
|---|---|---|---|---|
| **Florence-2-base/large** | Detect + caption + OCR | 0.23B / 0.77B | 30–80 ms | CPU ok, GPU 2 GB |
| **YOLOv11-n/s** | Real-time detection | 2–10 M | 5–15 ms | CPU/GPU trivial |
| **PaddleOCR v4** | Fast OCR | ~10 M | 20 ms/page | CPU |
| **Surya / docTR** | High-quality OCR + layout | ~100 M | 100 ms/page | GPU 2 GB |
| **GroundingDINO** | Text-prompted detection | 0.6 B | 120 ms | GPU 4 GB |
| **SAM2** | Segment anything | 0.2–2 B | 40–300 ms | GPU 4–12 GB |

See `PREPROCESSING.md` for how CLX wires these in front of the VLM.

---

## Head-to-Head Benchmarks (reported / reproduced)

Higher is better. Blank = not reported. int4 when local; source models
cloud for reference.

| Model | ScreenSpot-V2 (grounding %) | OSWorld (task success %) | AndroidWorld (%) | MMMU | VisualWebArena |
|---|---|---|---|---|---|
| UI-TARS-2-72B | **90.3** | **42.5** | **73.0** | — | — |
| UI-TARS-1.5-72B | 88.4 | 38.1 | 70.6 | — | — |
| UI-TARS-1.5-7B | 84.2 | 27.5 | 64.2 | 58 | — |
| CogAgent-9B | 85.4 | 22.0 | 60.1 | 55 | — |
| Aguvis-72B | 89.2 | 35.9 | 71.8 | — | — |
| Aguvis-7B | 82.8 | 22.4 | 59.7 | — | — |
| OS-Atlas-Pro-7B | 82.5 | 18.7 | 57.5 | — | — |
| ShowUI-2B | 75.1 | 10.2 | 48.0 | — | — |
| Qwen2.5-VL-72B | 86.8 | 35.0 | 65.0 | 70.2 | — |
| Qwen2.5-VL-7B | 78.9 | 20.5 | 55.6 | 58.6 | — |
| Molmo-72B | — | — | — | 65.5 | — |
| InternVL3-78B | — | — | — | **72.2** | — |
| Llama-3.2-90B-V | — | — | — | 60.3 | — |
| Claude 3.5 Sonnet (cloud, ref) | 87.6 | 22.0 | 57.8 | 70.4 | — |
| GPT-4o (cloud, ref) | 78.5 | 19.8 | 52.0 | 69.1 | — |

*Numbers drawn from each paper's reported results; re-check on release
before quoting externally.*

---

## Latency Budget for Real-Time Agents

Frame-in → action-out, single-agent loop:

| Config | 4090 | M2 Max | CPU-only (AVX-512) |
|---|---|---|---|
| ShowUI-2B int4 | 180 ms | 350 ms | 1.5 s |
| UI-TARS-7B int4 | 320 ms | 700 ms | 4 s |
| Qwen2.5-VL-7B int4 | 380 ms | 800 ms | 5 s |
| CogAgent-9B int4 | 480 ms | 1.0 s | 7 s |
| UI-TARS-72B int4 | 1.1 s | (OOM) | (infeasible) |
| UI-TARS-7B + YOLO preproc | 120 ms | 250 ms | 800 ms |

For **games** (sub-100 ms): Class A ≤ 2B + local YOLO, or pre-planned
reflex (`scan`) — see `GAMES-INPUT-PATTERNS.md`.

For **desktop automation** (turn-based, 0.5–3 s): UI-TARS-7B or
Qwen2.5-VL-7B on 16 GB GPU is more than fast enough.

For **reasoning-heavy** (complex forms, unfamiliar UI): step up to 32–72B
or keep local 7B for execution and send a single screenshot to a cloud
reasoner on hard steps.

---

## Inference Stacks

| Stack | Best for | Notes |
|---|---|---|
| **vLLM** | Servers, 4090+/H100, max throughput | PagedAttention; int4/AWQ; vision support for Qwen2.5-VL, InternVL, Llava. |
| **SGLang** | Low-latency serving | Radix cache; strong on UI-TARS. |
| **llama.cpp** (GGUF) | CPU / M-series / low-VRAM | MiniCPM-V, Qwen2.5-VL, Gemma-3, Molmo supported. Slowest but universal. |
| **Ollama** | Easy local dev | Wraps llama.cpp; vision models via `ollama pull qwen2.5vl`. |
| **LM Studio** | GUI users | Same backends as Ollama; used in UI-TARS-Desktop zero-config flow. |
| **MLX** | Apple Silicon | Efficient on M-series; Qwen-VL, Phi-V, MiniCPM-V have ports. |
| **TensorRT-LLM** | NVIDIA production | Best latency on H100/B200; more setup. |
| **HuggingFace Transformers** | Prototyping | Slowest, most flexible. |

---

## End-to-End Frameworks (open source)

Wire a model to screen capture + input injection out of the box.

| Project | Model fit | Platform |
|---|---|---|
| **UI-TARS-Desktop** (`bytedance/UI-TARS-desktop`) | UI-TARS 1.5/2, any OpenAI-compat | macOS / Win / Linux |
| **Agent TARS** (`bytedance/agent-tars`) | Same + MCP tools | CLI / Web |
| **OpenInterpreter `os` mode** | Any local VLM via Ollama | All |
| **Self-Operating Computer** | Any VLM | All |
| **browser-use** | Any VLM | Browser |
| **Cerebellum** | Any VLM | Browser |
| **Cradle** (`BAAI-Agents/Cradle`) | GPT-4V default, swappable | Games |
| **OpenACT** | UI-TARS / Qwen2.5-VL | Desktop |
| **Skyvern** | GPT-4V default, swappable | Browser |

---

## Real-Time Video Streaming

### Short answer

**Almost none of the Class A / B models above do true live streaming.**
They all follow the same loop: `screenshot → send frame → wait for action → repeat`.
At 7B int4 on a 4090 that loop runs at ~2–4 Hz — which is fine for desktop
automation but is not the same as a model receiving a continuous video feed.

### What "real-time video" actually means here

| Mode | How it works | Practical FPS |
|---|---|---|
| **Frame-per-request** | Your code grabs a screenshot, sends it as an image in the prompt, gets an action back | 2–4 FPS (7B, 4090) |
| **Multi-frame / video clip** | Send a short clip (8–16 frames) per call, model reasons across them | 1–2 effective Hz |
| **Persistent KV streaming** | Model keeps an open session; you push frames into the KV cache continuously, never restart | true continuous |

All Class A/B models above use frame-per-request or video-clip mode.
**Persistent KV streaming** is architecturally different — only the models
below are built for it.

### Models with native streaming architecture

| Model | Size | License | Notes |
|---|---|---|---|
| **MiniCPM-o 2.6** | 8B | Apache-2.0 | **Best local live-streaming option.** Designed as an open-source Gemini Live. Takes real-time video + audio in; outputs speech + text continuously. Beats GPT-4o-202408 on StreamingBench. Runs on iPad via llama.cpp, Ollama-compatible. |
| **StreamingVLM** (MIT Han Lab) | base: Qwen2.5-VL-7B | research | KV-sink + sliding window streaming; up to 8 FPS on H100. Not consumer-grade. Academic proof-of-concept. |

### Qwen2.5-VL / InternVL / UI-TARS — video capable but not streaming

These models accept video (list of frames) as input and can reason across
them, but they process a fixed clip per call — there is no open session.
You can simulate streaming by calling them in a tight loop at 1–4 Hz,
which is adequate for desktop agent tasks.

### Practical recommendation

- **Desktop automation (≤ 4 Hz)**: any Class A/B model; frame-per-request is fine.
- **Continuous screen watching, commentary, Q&A while playing**: **MiniCPM-o 2.6** (8B, Ollama: `openbmb/minicpm-o2.6`).
- **Sub-100 ms game reflex**: skip VLM in the hot loop entirely; use local YOLO/pixel-scan (`scan` command in CLX).

---

## World Models

### What is a world model?

A world model predicts "what happens next" given the current state and an
action — it **simulates** rather than **observes**. Different from a GUI
agent (which observes a screen and outputs clicks).

For a desktop agent the main use case would be **lookahead planning**:
"if I click here, what will the UI look like?" — rather than direct
screen observation.

### Open-source options

| Model | Size | License | What it does | Local? |
|---|---|---|---|---|
| **COSMOS 1.0** (NVIDIA) | 4B / 7B / 14B | permissive commercial | Physical-world video generation; conditioned on action. Robotics / autonomous-driving focus. | Yes (4090 for 4B, H100 for 14B) |
| **DreamerV3** | varies | Apache-2.0 | RL world model; learns a compact latent world from interaction. Not multimodal by default. | Yes, lightweight |
| **LingBot-World** | — | Apache-2.0 | Interactive game-environment generation at 16 FPS. Early-stage. | Yes |
| **Open-Sora** | — | Apache-2.0 | Video generation (diffusion); not an interactive world model. | Yes (A100) |
| **Genie 2 / 3** (DeepMind) | — | **closed** | SOTA interactive world model; 24 FPS 3D worlds. Not available locally. | No |
| **GameNGen** (Google) | — | research | Runs Doom on a diffusion model; proof-of-concept. No public weights. | No |

### Reality check for CLX use

- No open-source world model is currently mature enough to simulate
  arbitrary desktop GUIs or games with sufficient fidelity to replace
  real screen captures.
- COSMOS is the only one practical to run locally today, but it targets
  physical scenes (robotics), not software UIs.
- **DreamerV3** is useful if you want an RL agent that learns a game
  internally without calling an LLM every frame — but it requires training
  on the specific game.
- For CLX the more useful near-term investment is the YOLO+OCR
  preprocessing pipeline (`PREPROCESSING.md`) rather than a world model.

---

## Decision Matrix

Pick the row whose "must have" applies to you; the last column is the model.

| Must have | Secondary | Model |
|---|---|---|
| Fits 8 GB laptop GPU | any | ShowUI-2B int4 |
| Best quality at 16 GB | speed | **UI-TARS-1.5-7B int4** |
| Best quality at 16 GB | one model for everything | **Qwen2.5-VL-7B int4** |
| Best quality at 24 GB | agent-specific | UI-TARS-1.5-7B fp16 or CogAgent-9B |
| Single-GPU SOTA | has 48 GB | UI-TARS-2-72B int4 or Aguvis-72B int4 |
| CPU-only / no GPU | phone-class | MiniCPM-V 2.6 Q4 or ShowUI-2B Q4 |
| Max grounding precision | can add planner | Molmo-7B + LLM planner |
| Commercially safe license | — | any Apache-2.0 / MIT row |
| Mobile/Android automation | — | Ferret-UI 2 or Aguvis |
| Game reflex (<100 ms) | simple tasks | YOLO + pre-planned scan, no VLM in hot loop |
| Live streaming (continuous video) | local, no API | **MiniCPM-o 2.6** (8B, Ollama) |
| World model / lookahead planning | any HW | COSMOS 4B (physical) or DreamerV3 (RL) — both immature for UI use |

---

## Integration Notes for CLX

- **Action format conversion**: UI-TARS / Qwen2.5-VL / OS-Atlas emit
  JSON actions with coords in 0–1000 normalized space. Adapter:
  `x_px = x_norm / 1000 * screen_w`. Map `click` → `m x y c`,
  `type` → `k "text"`, `hotkey` → `k <chord>`, `scroll` → `m x y c` +
  scroll event. See `PARSER.md`.
- **Screen cadence**: 7B VLM on 4090 = ~2–4 Hz full frames. Use local
  YOLO+OCR on every frame (30–60 Hz) and only invoke VLM on state change
  (see `PREPROCESSING.md`).
- **Multi-model fallback**: ShowUI-2B (fast) → UI-TARS-7B (primary) →
  Qwen2.5-VL-32B (hard cases) mirrors the cloud Fast/Reasoning split
  from `MODELS.md`.
- **Quantization choice**: AWQ-int4 for NVIDIA, GGUF Q4_K_M for
  llama.cpp/CPU/Metal. Avoid Q3 on < 7B models — grounding accuracy
  collapses.

---

## References

- UI-TARS paper: https://arxiv.org/abs/2501.12326
- UI-TARS-2 report: https://arxiv.org/abs/2509.02544
- OS-Atlas: https://arxiv.org/abs/2410.23218
- Aguvis: https://arxiv.org/abs/2412.04454
- ShowUI: https://arxiv.org/abs/2411.17465
- CogAgent: https://github.com/zai-org/CogAgent
- MiniCPM-o: https://github.com/OpenBMB/MiniCPM-o
- StreamingVLM: https://arxiv.org/abs/2510.09608
- COSMOS: https://www.nvidia.com/en-us/ai/cosmos/
- DreamerV3: https://github.com/danijar/dreamerv3
- Qwen2.5-VL: https://qwenlm.github.io/blog/qwen2.5-vl/
- Molmo: https://molmo.allenai.org/
- InternVL3: https://internvl.github.io/
- GUI-Agents paper list: https://github.com/OSU-NLP-Group/GUI-Agents-Paper-List

## See Also

- [MODELS.md](./MODELS.md) — cloud model categories
- [MODEL-FAST.md](./MODEL-FAST.md) · [MODEL-REASONING.md](./MODEL-REASONING.md) · [MODEL-REALTIME.md](./MODEL-REALTIME.md)
- [PREPROCESSING.md](./PREPROCESSING.md) — YOLO / OCR front-end
- [PARSER.md](./PARSER.md) — action-JSON → CLX command adapter
