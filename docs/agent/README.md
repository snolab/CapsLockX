# CLX Agent Language — Real-Time LLM-to-Input Control System

A token-efficient, streaming DSL designed for LLMs to operate computers in
real-time: keyboard, mouse, gamepad sticks, MIDI, and more.

## Design Documents

| Document | Description |
|---|---|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System architecture, data flow, latency budget |
| [LANGUAGE.md](./LANGUAGE.md) | Language specification and grammar |
| [SENSE.md](./SENSE.md) | Sensory inputs — everything the LLM can perceive |
| [PROMPTS.md](./PROMPTS.md) | System prompts for each agent mode |
| [MODELS.md](./MODELS.md) | Model overview — when to use which category |
| [MODEL-FAST.md](./MODEL-FAST.md) | Fast streaming models (Groq, Cerebras, local) |
| [MODEL-REASONING.md](./MODEL-REASONING.md) | Reasoning models (Gemini Pro, Claude, o3) |
| [MODEL-REALTIME.md](./MODEL-REALTIME.md) | Realtime bidirectional (Gemini Live, OpenAI RT) |
| [PARSER.md](./PARSER.md) | Streaming parser design and technology choice |
| [PREPROCESSING.md](./PREPROCESSING.md) | YOLO/STT/OCR preprocessing vs native LLM input |
| [INPUT.md](./INPUT.md) | Input injection APIs (keyboard, mouse, gamepad, MIDI) |
| [STREAMING-OUTPUT.md](./STREAMING-OUTPUT.md) | Async command queue design for streaming execution |
| [ROADMAP.md](./ROADMAP.md) | Implementation phases and milestones |
