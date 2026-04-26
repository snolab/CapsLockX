# Timescale-Split Agent Architecture

Two layers running at different speeds, sharing a rule table.
Fast layer reacts; slow layer learns and rewrites the rules.

Mirrors human cognition (System 1 / System 2), Hierarchical RL,
and proven game-agent projects (Voyager, Generative Agents).

---

## Overview

```
Screen / Audio / Mic  (continuous)
         │
         ▼
┌────────────────────────────────────┐
│  FAST LAYER   5–50ms, 60fps        │
│                                    │
│  YOLO + OCR + keyword match        │
│  rule-table lookup → CLX command   │
│  Whisper-tiny (voice keywords)     │
└────────────┬───────────────────────┘
             │ observation log (async)
             ▼
┌────────────────────────────────────┐
│  SLOW LAYER   seconds–minutes      │
│                                    │
│  VLM (UI-TARS-7B / Qwen2.5-VL-7B) │
│  reads last N seconds of log       │
│  + long-term memory                │
│  → rewrites rule table             │
│  → writes memory                   │
└────────────┬───────────────────────┘
             │
             ▼
┌────────────────────────────────────┐
│  SHARED STATE                      │
│                                    │
│  rules/live.toml   ← slow writes   │
│                    → fast reads    │
│  memory/*.md       (CLX format)    │
│  tmp/obs-log.jsonl (rolling 60s)   │
└────────────────────────────────────┘
```

---

## Fast Layer

**Responsibility**: react within one frame budget (~16ms ideal, 50ms max).
No LLM call. No network. Pure local signal → rule lookup → CLX output.

### Inputs

| Signal | Tool | Latency |
|---|---|---|
| Screen region (pixel) | `scan` built-in | <1ms |
| Object detection | YOLOv11n | 5–15ms |
| OCR / text on screen | PaddleOCR / Florence-2 | 20–30ms |
| Voice keyword | Whisper-tiny (streaming) | 30ms |
| Audio event | energy/spectral classifier | 5ms |

### Rule table format (`rules/live.toml`)

```toml
[[rule]]
trigger = {hp_ratio = "<0.3"}
action  = "k left"
note    = "boss rushes below 30% HP — dodge left"
confidence = 0.87

[[rule]]
trigger = {ocr_contains = "OK", region = [800,600,200,60]}
action  = "m {cx} {cy} c"
note    = "dismiss modal"
confidence = 0.95

[[rule]]
trigger = {voice_keyword = "stop"}
action  = "k escape"
confidence = 1.0
```

Rules are hot-reloaded on file change — no restart needed.

### What fast layer does NOT do

- No image understanding beyond YOLO boxes + OCR text
- No memory reads
- No planning
- No LLM calls

---

## Slow Layer

**Responsibility**: observe patterns over time, update the rules the fast
layer uses, and write durable memories.

Runs asynchronously — never blocks the fast layer.
Triggered by: timer (every 10–30s), significant state change, task failure.

### Inputs

```
tmp/obs-log.jsonl      rolling 60-second observation window
  each line: {ts, yolo_detections, ocr_text, action_taken, outcome}

memory/*.md            long-term memory (CLX format)

screenshot             one frame of current screen
```

### Output

```json
{
  "rule_updates": [
    {
      "trigger": {"hp_ratio": "<0.3"},
      "action": "k left",
      "confidence": 0.87,
      "note": "boss rushes below 30% HP — dodge left"
    }
  ],
  "memory_write": {
    "file": "memory/game-boss-2.md",
    "content": "Phase 2 starts at 50% HP. Pattern: rush → jump → AoE."
  },
  "strategy_note": "Ranged approach more efficient than melee in phase 2."
}
```

The slow layer literally edits `rules/live.toml` and `memory/*.md`.
Fast layer picks up changes on next hot-reload tick.

### Model choice

| Use case | Model | Why |
|---|---|---|
| Desktop automation | Qwen2.5-VL-7B | built-in computer_use schema |
| Game agent | UI-TARS-7B | trained on GUI action sequences |
| Deep planning | UI-TARS-72B or cloud | complex multi-step |

---

## Memory Tiers

```
working memory    tmp/obs-log.jsonl      rolling 60s, overwritten
                  (fast layer appends, slow layer reads)

episodic memory   memory/session-*.md    per-session summaries
                  (slow layer writes after each session)

semantic memory   memory/rules-learned.md   stable learned patterns
                  (slow layer writes when confidence > 0.8)

procedural        rules/live.toml           active rule set
                  (slow layer edits, fast layer executes)
```

This maps to the existing CLX memory format in `memory/*.md`.

---

## Voice / Audio Split

Same timescale split applies:

```
Fast:  Whisper-tiny (realtime transcription) → keyword match → instant action
       audio energy classifier → attack sound → dodge

Slow:  full conversation context → VLM interprets intent
       → update rules or write memory
```

MiniCPM-o 2.6 attempts this in one model (live streaming VLM + audio),
but splitting gives tighter latency control. Use MiniCPM-o when you need
low-effort setup; use split when latency budget is strict.

---

## Latency Budget

| Layer | Budget | Slack |
|---|---|---|
| Pixel scan (`scan`) | <1ms | large |
| YOLO + rule lookup | 20ms | comfortable |
| OCR + rule lookup | 50ms | ok for UI |
| Slow VLM (7B, 4090) | 300ms async | non-blocking |
| Slow VLM (7B, CPU) | 4s async | still non-blocking |

Async means: fast layer never waits. Rule updates arrive between frames
and are applied on the next hot-reload tick.

---

## Prior Art

| Project | Fast layer | Slow layer | Medium |
|---|---|---|---|
| **Voyager** (Minecraft) | learned skill execution | GPT-4 writes new skill code | text |
| **Generative Agents** | immediate action | memory summarization + reflection | text |
| **AlphaGo** | MCTS rollouts | deep value network | RL |
| **DreamerV3** | latent policy | world model update | RL |
| **CLX (this design)** | YOLO + rule table | VLM rewrites rules + memory | multimodal |

---

## See Also

- [LOCAL-AGENT-MODELS.md](./LOCAL-AGENT-MODELS.md) — which VLM to use for slow layer
- [PREPROCESSING.md](./PREPROCESSING.md) — YOLO / OCR pipeline (fast layer inputs)
- [MODELS.md](./MODELS.md) — cloud model alternatives for slow layer
- [GAMES-INPUT-PATTERNS.md](./GAMES-INPUT-PATTERNS.md) — rule patterns per game genre
- `src/bin/clx-agent.rs` — current agent implementation (AX tree + LLM loop)
