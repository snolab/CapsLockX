# Models Overview

Three categories of models serve different roles in the CLX Agent system.
The right model depends on the task's latency, quality, and cost requirements.

## Model Categories

| Category | Role | Latency | Quality | Cost | Doc |
|---|---|---|---|---|---|
| **Fast** | Streaming command generation | <200ms TTFT, 300-2000 tok/s | Good enough | Low | [MODEL-FAST.md](./MODEL-FAST.md) |
| **Reasoning** | Complex planning, scene analysis | 500ms-5s | Highest | High | [MODEL-REASONING.md](./MODEL-REASONING.md) |
| **Realtime** | Live vision + audio feedback loop | ~300ms bidirectional | Good | Medium | [MODEL-REALTIME.md](./MODEL-REALTIME.md) |

## When to Use Which

```
User says "open Safari and go to github.com"
  → FAST model: stream CLX commands directly, no vision needed

User says "play this platformer for me"
  → REALTIME model: continuous vision + audio + command output
  → or FAST model fed by local YOLO/OCR preprocessing

User says "analyze this complex UI and figure out how to file a tax return"
  → REASONING model: think deeply about the screenshot, plan steps
  → then FAST model: execute the plan as CLX commands

User says "watch me play and give tips"
  → REALTIME model: observe screen + audio, speak advice
```

## Multi-Model Pipeline

The most capable setup uses all three together:

```
┌─────────────────────────────────────────────────────────┐
│  LOCAL PREPROCESSING (30-60 fps)                        │
│  YOLO + OCR → structured detections + text              │
└──────────────┬──────────────────────────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
    ▼                     ▼ (every 5-10s or on-demand)
┌────────────┐    ┌──────────────────┐
│ FAST model │    │ REASONING model  │
│ Groq/      │    │ Gemini 2.5 Pro / │
│ Cerebras   │    │ Claude Opus /    │
│            │    │ GPT-4.1          │
│ Streams    │    │                  │
│ CLX cmds   │    │ Returns plan:    │
│ at 300+    │    │ high-level steps │
│ tok/s      │    │ + strategy       │
└─────┬──────┘    └────────┬─────────┘
      │                    │
      │    ┌───────────────┘
      │    │ plan feeds into fast model's context
      ▼    ▼
┌─────────────────┐
│ CLX Executor    │
│ (RT thread)     │
└─────────────────┘
```

**Alternative — Realtime model handles everything:**
```
┌─────────────────────────────────────────┐
│  REALTIME model (Gemini Live API)       │
│  Receives: video frames + audio + text  │
│  Outputs: CLX commands (streaming)      │
│  Latency: ~300-600ms per decision       │
└──────────────────┬──────────────────────┘
                   │
            ┌──────▼──────┐
            │ CLX Executor │
            └─────────────┘
```

Simpler but limited to 1-2 fps and higher cost per frame.

## Recommended Configurations

### Game control (lowest latency)
1. **Primary**: Groq Llama 3.3 70B (fast, streaming CLX commands)
2. **Vision**: Gemini Live API (periodic screenshots) or local YOLO+OCR
3. **Planner**: Gemini 2.5 Flash (on pause/complex decisions)

### Desktop automation
1. **Primary**: Gemini 2.0 Flash (fast + vision in one model)
2. **Planner**: Claude Sonnet (best UI understanding, used on complex screens)

### Offline / privacy
1. **Primary**: Qwen 2.5 7B via llama.cpp (best local quality)
2. **Perception**: Florence-2 via ONNX (local vision)

### Cost-optimized
1. **Primary**: Gemini 2.0 Flash-Lite (cheapest cloud)
2. **Fallback**: Llama 3.2 3B local (zero cost)

## Game Speed Feasibility

| Game type | Feasibility | Decision cycle | Best model config |
|---|---|---|---|
| Turn-based (XCOM, Civ) | Excellent | 2-5s | Reasoning + Fast |
| Strategy (paused) | Good | 0.5-2s | Reasoning + Fast |
| Slow platformer | Good | 0.3-1s | Fast + YOLO |
| Menu/UI navigation | Excellent | 0.5-1s | Fast + OCR |
| RPG (Stardew) | Good | 0.5-2s | Realtime or Fast + YOLO |
| Fast platformer | Marginal | 50-100ms | Fast + pre-planned sequences |
| FPS (competitive) | Not feasible | <16ms | LLM too slow |
| Fighting games | Not feasible | <16ms | LLM too slow |

## Existing Game Agent Projects

| Project | Games | Model | Speed | Approach |
|---|---|---|---|---|
| **CRADLE** | RDR2, Stardew Valley | GPT-4V | ~5-10s/action | Screenshot → reasoning → action |
| **Voyager** | Minecraft | GPT-4 (text) | ~2-5s/action | Text API, code generation |
| **SteveEye** | Minecraft | GPT-4V | ~3-5s/action | Multimodal perception |
| **DeepMind SIMA** | Multiple 3D | Custom NN | Real-time | Not LLM, specialized NN (not public) |
