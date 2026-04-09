# Implementation Roadmap

## Phase 0 — Foundation (1-2 weeks)

### 0.1 Parser prototype
- [ ] Add `winnow` dependency to `capslockx-core`
- [ ] Implement `rs/core/src/agent/lang.rs` — grammar for `k`, `m`, `w` commands
- [ ] Unit tests: parse individual commands from complete strings
- [ ] Unit tests: streaming parse with `Partial<&str>`, verify Incomplete behavior

### 0.2 Command types
- [ ] Define `Command` enum in `rs/core/src/agent/commands.rs`
- [ ] Keyboard: `KeyTap`, `KeyDown`, `KeyUp`, `TypeString`
- [ ] Mouse: `MouseMove` (abs/rel/lerp), `MouseClick`, `MouseScroll`
- [ ] Wait: `Wait(Duration)`

### 0.3 Executor skeleton
- [ ] `rs/core/src/agent/mod.rs` — `AgentInterpreter` struct
- [ ] crossbeam channel: parser → executor
- [ ] Executor thread with `spin_sleep` timing loop
- [ ] Wire keyboard/mouse commands to existing CapsLockX platform trait

### 0.4 CLI test harness
- [ ] `echo "k a; w 100ms; k b" | clx --agent` — pipe commands to executor
- [ ] Verify keys are injected correctly
- [ ] Measure end-to-end latency (echo → keypress)

## Phase 1 — LLM Streaming (1-2 weeks)

### 1.1 SSE stream consumer
- [ ] `rs/core/src/agent/stream.rs` — async SSE reader
- [ ] Support OpenAI-compatible SSE format (`data: {"choices":[...]}`)
- [ ] Support Gemini SSE format
- [ ] Feed text chunks into parser thread via channel

### 1.2 API integration
- [ ] Groq API client (OpenAI-compatible, fastest cloud)
- [ ] Gemini API client (best vision)
- [ ] Configurable model/provider in CapsLockX preferences
- [ ] API key management (reuse existing CapsLockX config)

### 1.3 Prompt engineering
- [ ] System prompt that teaches the LLM the CLX agent language
- [ ] Few-shot examples for common tasks
- [ ] Test: "open Finder and create a new folder" → correct command stream
- [ ] Test: "play Mario — jump over the first goomba" → gamepad commands

### 1.4 Hotkey activation
- [ ] `CLX+A` — activate agent mode, open prompt (reuse brainstorm prompt subprocess)
- [ ] Stream LLM response directly into agent executor
- [ ] `ESC` — cancel agent execution

## Phase 2 — Advanced Input (2-3 weeks)

### 2.1 Gamepad emulation
- [ ] macOS: IOHIDUserDevice virtual gamepad
- [ ] Windows: ViGEm virtual Xbox 360 controller
- [ ] `p` command: stick, trigger, button, d-pad
- [ ] Lerp interpolation for smooth stick movement

### 2.2 MIDI output
- [ ] `midir` integration — create virtual MIDI source
- [ ] `M` command: note on/off, CC, pitch bend, program change
- [ ] Named channels: `@midi1: M n 60 100`

### 2.3 Smooth mouse movement
- [ ] Lerp executor: interpolate mouse position over duration
- [ ] Bezier curves: `m ~500 300 ~600 100 500ms` (control points)
- [ ] Configurable update rate (default 120 Hz)

### 2.4 Concurrent commands
- [ ] `|` operator: run commands in parallel
- [ ] Useful for: move mouse while holding keys, stick + buttons

## Phase 3 — Vision Feedback Loop (2-3 weeks)

### 3.1 Screen capture integration
- [ ] macOS: ScreenCaptureKit wrapper
- [ ] Windows: DXGI Desktop Duplication
- [ ] Configurable capture rate (default 2 fps for LLM, higher for local)
- [ ] Downscale + JPEG encode pipeline

### 3.2 Gemini Live API integration
- [ ] WebSocket client for bidirectional streaming
- [ ] Send video frames + receive text commands
- [ ] Handle interleaved audio/text responses

### 3.3 Closed-loop agent
- [ ] Screen → LLM → Commands → Input → Screen (full loop)
- [ ] "Watch me play and suggest improvements" mode
- [ ] "Play this game for me" autonomous mode
- [ ] Safety: kill switch (ESC or CLX+ESC always stops agent)

### 3.4 Local vision model
- [ ] Small vision model (e.g., Florence-2, PaliGemma) via ONNX runtime
- [ ] Detect UI elements, game objects, text
- [ ] Feed structured scene description to command LLM
- [ ] Enables offline/private operation

## Phase 4 — Polish & Extensions (ongoing)

### 4.1 Language extensions
- [ ] `r` repeat loops
- [ ] Variables: `$x = 500; m $x 300`
- [ ] Labels + jump: `:loop ... j loop`
- [ ] Conditionals (maybe): `? screen_has "button" { m click }`

### 4.2 Recording & replay
- [ ] Record user input → CLX agent language script
- [ ] Replay scripts with timing
- [ ] Edit recorded scripts, re-run modified

### 4.3 Multi-model pipeline
- [ ] Vision model (periodic) → scene description
- [ ] Fast command model (streaming) → CLX commands
- [ ] Separate latency budgets for each stage

### 4.4 Web UI
- [ ] Live view of agent commands being executed
- [ ] Manual override: type CLX commands directly
- [ ] Visualization of mouse movement, button states

## Dependencies to Add

```toml
# In capslockx-core/Cargo.toml
winnow = "0.6"          # streaming parser
crossbeam-channel = "0.5" # lock-free channels
spin_sleep = "1"         # sub-ms timing
midir = "0.10"           # MIDI I/O

# Optional / platform-specific
screencapturekit = "0.2" # macOS screen capture
# vigem-client             # Windows gamepad (via FFI)
```

## Latency Targets

| Metric | Target | How to measure |
|---|---|---|
| Parse latency | < 0.1ms | Benchmark parse_statement on 50-char input |
| Inject latency | < 1ms | Timestamp before/after CGEventPost |
| LLM TTFT | < 200ms | Measure from request to first SSE chunk |
| First command | < 250ms | From user prompt to first input injection |
| Continuous rate | 50-200 Hz | Commands/second during active streaming |
| Feedback loop | < 500ms | Screen capture → LLM → command → injection |
