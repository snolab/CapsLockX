# Architecture — CLX Agent Language

## Overview

```
                         ┌──────────────────────────────────┐
                         │         LLM Provider             │
                         │  (Gemini Flash / Groq / Local)   │
                         └──────────┬───────────────────────┘
                                    │ SSE / WebSocket stream
                                    │ (tokens arrive ~5-20ms apart)
                         ┌──────────▼───────────────────────┐
                         │     Stream Accumulator            │
                         │  (UTF-8 byte buffer, async)       │
                         └──────────┬───────────────────────┘
                                    │ char-by-char feed
                         ┌──────────▼───────────────────────┐
                         │     Streaming Parser (winnow)     │
                         │  Partial<&str> → Command | Need   │
                         │  more | Error                     │
                         └──────────┬───────────────────────┘
                                    │ parsed commands
                         ┌──────────▼───────────────────────┐
                         │     Command Scheduler             │
                         │  - Immediate execution            │
                         │  - Timed sequences (wait/at)      │
                         │  - Continuous axes (lerp/hold)    │
                         └──────┬────┬────┬────┬────────────┘
                                │    │    │    │
                    ┌───────────┘    │    │    └───────────┐
                    ▼                ▼    ▼                ▼
              ┌──────────┐  ┌──────────┐ ┌─────┐  ┌──────────┐
              │ Keyboard │  │  Mouse   │ │ Pad │  │   MIDI   │
              │ Injector │  │ Injector │ │ Emu │  │  Output  │
              └──────────┘  └──────────┘ └─────┘  └──────────┘
              CGEventPost   CGEventPost  ViGEm/   CoreMIDI/
              /SendInput    /SendInput   IOKit    midir

                         ┌──────────────────────────────────┐
                         │     Screen Capture (feedback)     │
                         │  ScreenCaptureKit / DXGI DD       │
                         │  → frames → LLM vision input     │
                         └──────────────────────────────────┘
```

## Latency Budget (target: < 100ms end-to-end)

| Stage | Budget | Notes |
|---|---|---|
| LLM TTFT | 50-200ms | Use Groq/Cerebras/Gemini Flash for < 150ms |
| Token→Parser | < 1ms | In-process, zero-copy |
| Parse statement | < 0.1ms | winnow: ~600MB/s, statements are tiny |
| Schedule+Inject | < 1ms | Pre-allocated event structs, RT thread |
| OS→App delivery | 1-8ms | Depends on app polling rate |
| **Total** | **~55-210ms** | Dominated by LLM TTFT |

For continuous control (mouse/stick), the LLM streams position updates at
token speed (~50-200 tok/s). Each token parses and executes in < 2ms.

## Key Design Principles

### 1. Token Efficiency
Every token from the LLM costs latency and money. The language must be
**maximally dense** — a single token should encode a meaningful action.
Avoid JSON, XML, or any verbose format. Use single-character operators
and short mnemonics.

### 2. Streaming Execution
Commands execute **as soon as they are syntactically complete** in the
stream. The parser never waits for the full response. A newline or
semicolon terminates a command and triggers immediate execution.

### 3. Continuous Axes
Mouse movement, analog sticks, and MIDI controllers need **continuous
value updates**, not just discrete events. The language supports:
- Absolute positioning: `m 500 300` (move mouse to 500,300)
- Relative deltas: `m +10 -5` (move mouse by +10,-5)
- Interpolated motion: `m ~500 300 200ms` (lerp to 500,300 over 200ms)
- Hold/release semantics for buttons with concurrent axis updates

### 4. Real-Time Thread
Input injection runs on a dedicated high-priority thread with:
- `spin_sleep` for sub-ms timing
- Pre-allocated event buffers (no allocation in hot path)
- Lock-free command queue (crossbeam channel)

### 5. Feedback Loop
Screen capture feeds back to the LLM for closed-loop control:
- Capture at game framerate (60-144 fps)
- Downscale to ~256x256 for fast LLM processing
- Use Gemini Live API for real-time vision or periodic screenshots

## Module Structure

```
rs/core/src/agent/
  mod.rs            — public API: AgentInterpreter
  lang.rs           — language grammar (winnow parser)
  commands.rs       — Command enum and execution
  scheduler.rs      — timed execution, interpolation
  devices/
    mod.rs
    keyboard.rs     — key tap/hold/release
    mouse.rs        — move/click/scroll
    gamepad.rs      — virtual gamepad (ViGEm/IOKit)
    midi.rs         — MIDI note/CC output
  stream.rs         — LLM stream consumer (SSE/WebSocket)
  feedback.rs       — screen capture → LLM
```
