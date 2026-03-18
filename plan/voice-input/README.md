# Voice Input Feature (CLX+V)

## Hotkey Behavior

### Toggle Mode (CLX+V click)
```
[V down] → start listening + VAD
  → VAD detects voice → buffer audio
  → VAD detects silence → send chunk to server for transcription
  → server returns text → type it at cursor
  → keep listening for next voice segment
[V down again] → stop listening, cleanup
```

### Hold Mode (CLX+V hold)
```
[V down] → start listening + VAD (same as toggle)
  → VAD detects voice → buffer audio
  → VAD detects silence → send chunk for transcription
  → keep listening while held...
[V up] → send any remaining audio → stop listening
```

Both modes share the same listening pipeline — the only difference is the stop trigger.

## Architecture

```
┌──────────────────────┐     ┌─────────────────────────┐
│  CapsLockX Client    │     │  Brainstorm Server      │
│  (Rust, macOS/Win)   │     │  (Next.js)              │
│                      │     │                         │
│  ┌────────────────┐  │     │  ┌───────────────────┐  │
│  │ Audio Capture  │  │     │  │ /api/asr          │  │
│  │ (CoreAudio /   │  │     │  │ Whisper API       │  │
│  │  WASAPI)       │  │     │  │                   │  │
│  └───────┬────────┘  │     │  └───────┬───────────┘  │
│          ↓           │     │          ↓              │
│  ┌────────────────┐  │     │  ┌───────────────────┐  │
│  │ VAD            │  │ HTTP│  │ /api/voice-fix    │  │
│  │ (silero-vad /  │──┼────→│  │ LLM typo-fix      │  │
│  │  webrtc-vad)   │  │     │  │ (gpt-4o-mini)     │  │
│  └───────┬────────┘  │     │  └───────┬───────────┘  │
│          ↓           │     │          ↓              │
│  ┌────────────────┐  │     │  ┌───────────────────┐  │
│  │ Local Whisper  │  │     │  │ Streaming Response │  │
│  │ (optional,     │  │     │  │ rough → refined    │  │
│  │  fast draft)   │  │     │  └───────────────────┘  │
│  └────────────────┘  │     │                         │
└──────────────────────┘     └─────────────────────────┘
```

## Transcription Pipeline (streaming, non-blocking)

For each voice segment detected by VAD:

1. **Immediate (local, ~100ms)**: Local Whisper tiny/base model → rough draft → type at cursor
2. **Fast (~1s)**: Send audio to server → OpenAI Whisper → refined text → replace rough draft
3. **Polish (~2s)**: Server runs LLM typo-fix on refined text → final clean text → replace again

Each stage replaces the previous text seamlessly. User sees text appear fast (local) and get refined in place.

## 30-Second Window Handling

Whisper models have a 30s input limit. Strategy:
- VAD splits audio into natural utterances (silence boundaries)
- If a single utterance exceeds 25s, force-split at 25s boundary
- Each chunk is transcribed independently
- Context from previous chunk passed as `prompt` parameter to Whisper for continuity

## Files to Implement

### Client (Rust)
- `rs/core/src/modules/voice.rs` — VoiceModule: hotkey state machine (toggle/hold), manages pipeline
- `rs/adapters/macos/src/audio_capture.rs` — CoreAudio microphone capture
- `rs/adapters/windows/src/audio_capture.rs` — WASAPI microphone capture
- `rs/core/src/vad.rs` — Voice Activity Detection (webrtc-vad crate or silero)
- `rs/core/src/voice_client.rs` — HTTP client to send audio chunks to server

### Server (brainstorm)
- `app/api/voice-transcribe/route.ts` — Combined transcribe + typo-fix endpoint
- Returns streaming: `{stage: "rough", text: "..."}` → `{stage: "refined", text: "..."}` → `{stage: "polished", text: "..."}`

## Dependencies

### Client
- `cpal` crate — cross-platform audio capture
- `webrtc-vad` or `silero-vad` crate — voice activity detection
- `reqwest` — HTTP client for server communication
- `whisper-rs` (optional) — local Whisper model for fast draft

### Server
- Already has OpenAI Whisper integration (`/api/asr`)
- Already has LLM integration (gpt-4o-mini for typo-fix)
- Needs new streaming endpoint combining both
