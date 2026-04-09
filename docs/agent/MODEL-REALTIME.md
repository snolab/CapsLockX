# MODEL-REALTIME — Bidirectional Streaming Models

Models with persistent WebSocket connections that accept continuous
audio/video input and stream text/audio output. Used for the **feedback
loop** — the model sees the screen and hears the user in real-time.

## Available Realtime APIs

### Google Gemini Live API

| Aspect | Detail |
|---|---|
| **Model** | Gemini 2.0 Flash |
| **Protocol** | WebSocket (`BidiGenerateContent`) |
| **Input types** | Text + Images + Audio (PCM 16-bit 16kHz) + **Video frames** |
| **Output types** | Text (streaming) + Audio (streaming) |
| **Text TTFT** | ~200-400ms |
| **Audio TTFT** | ~300-600ms |
| **Image input latency** | ~300ms per frame (added to text response) |
| **Sustainable video FPS** | 1-2 fps (higher bursts possible) |
| **Session context** | 1M tokens (persists across frames/turns) |
| **Image cost** | ~258-768 tokens per frame (resolution-dependent) |
| **Audio cost** | ~25 tokens/second of audio |
| **Image format** | JPEG/PNG, recommended ≤768x768 |
| **Audio format** | PCM 16-bit 16kHz |
| **Unique advantage** | **Only API with real-time video + audio input** |

#### IO Latency Breakdown
```
Frame captured ──(1ms)──→ JPEG encode ──(5-10ms)──→ WebSocket send ──(10-30ms network)──→
  Gemini inference ──(200-400ms)──→ First token ──(10-30ms network)──→ Parse + execute ──(1ms)──→
  Input injected

Total: ~230-470ms per decision from screen capture to input injection
```

#### Effective throughput by IO type
| Input | Tokens/frame | At 1fps | At 2fps |
|---|---|---|---|
| Image (512x512 JPEG) | ~400 | 400 tok/s input | 800 tok/s input |
| Image (768x768 JPEG) | ~768 | 768 tok/s input | 1536 tok/s input |
| Audio (continuous) | 25/s | 25 tok/s input | 25 tok/s input |
| Text context | varies | ~50-200 tok/s | ~50-200 tok/s |
| **Total input** | | **~475-968 tok/s** | **~875-1736 tok/s** |
| **Output** (CLX cmds) | | ~30-100 tok/s | ~30-100 tok/s |

**Cost at 1fps sustained**: ~$1.50-3.00/hour (input dominated by images)

### OpenAI Realtime API

| Aspect | Detail |
|---|---|
| **Model** | GPT-4o-realtime / GPT-4o-mini-realtime |
| **Protocol** | WebSocket |
| **Input types** | Text + Audio (PCM 16-bit 24kHz) |
| **Output types** | Text (streaming) + Audio (streaming) |
| **Text TTFT** | ~200-500ms |
| **Audio TTFT** | ~300-600ms |
| **Session context** | 128K tokens |
| **Audio cost** | ~100 tokens/second (higher than Gemini) |
| **Limitation** | **No image/video input in realtime API** |

#### IO Latency Breakdown
```
Audio captured ──(buffer 100-200ms)──→ WebSocket send ──(10-30ms)──→
  GPT-4o inference ──(200-400ms)──→ First token ──(10-30ms)──→ Parse + execute ──(1ms)──→

Total: ~320-660ms for voice command to input injection
```

For **vision**, must use separate standard API call:
```
Screenshot ──(standard API, 500-1500ms)──→ Response
```
This makes GPT-4o Realtime unsuitable for tight vision feedback loops.

## Latency Comparison (end-to-end by IO type)

| Scenario | Gemini Live | OpenAI Realtime | Fast model (Groq) + preprocessing |
|---|---|---|---|
| **Voice → text command** | 400-700ms | 350-650ms | 300-600ms (Whisper + API) |
| **Screenshot → command** | **250-500ms** | N/A (no image) | 200-400ms (YOLO+OCR → text → API) |
| **Audio → understand tone** | **Native** | **Native** | Lost (STT discards tone) |
| **Video → continuous control** | **1-2 fps, ~400ms** | N/A | YOLO 30fps + API 200ms |
| **Text → command stream** | 200-400ms TTFT | 200-500ms TTFT | **100-200ms TTFT** |
| **Token throughput** | 150-200 tok/s | ~100 tok/s | **300-2000 tok/s** |

### Key insight: Realtime APIs are NOT faster for text

For pure text-in → text-out, a **fast model** (Groq/Cerebras) is faster
and higher throughput. Realtime APIs shine specifically for:
- Native audio understanding (tone, emphasis, music)
- Continuous video input without per-request overhead
- Bidirectional conversation (talk while seeing)

## When to Use Realtime vs Fast + Preprocessing

| Scenario | Best choice | Why |
|---|---|---|
| Game playing (vision needed) | **Gemini Live** or **Fast + YOLO** | Gemini: simpler. Fast+YOLO: lower latency, higher FPS |
| Voice commands only | **Fast + local STT** | Lower latency, lower cost |
| Voice + emotion/tone matters | **Realtime API** | Native audio preserves prosody |
| Desktop automation | **Fast + OCR** | Text-based, no video needed |
| Music/audio reactive | **Gemini Live** | Needs native audio understanding |
| Cost-sensitive | **Fast + preprocessing** | 3-5x cheaper than continuous video |

## Session Management

Realtime APIs maintain a persistent session:

```
[Session start]
  → System prompt (CLX language reference)
  → Initial screenshot
  → Begin streaming

[During session]
  → Video frames at S-configured rate (default 0.5 fps)
  → Audio stream (if enabled)
  → Text sensor data (keyboard/mouse/gamepad events)
  ← CLX commands (streaming response)
  ← Audio responses (if voice mode)

[Context management]
  → At ~500K tokens: summarize history, reset session
  → Or: sliding window, drop oldest frames
```

## Local Realtime Alternatives

No true equivalent to Gemini Live exists locally, but you can approximate:

```
[Screen] → Florence-2 (10-30fps, local) → scene text
[Audio]  → Whisper.cpp (streaming, local) → transcript
[Both]   → Qwen 2.5 7B (local) → CLX commands
```

Latency: ~200-500ms total, fully offline. Quality is lower than Gemini
but viable for simple games and automation.

## References

- [2025-03] [Gemini Live API docs](https://ai.google.dev/gemini-api/docs/live) — BidiGenerateContent, audio/video input
- [2025-03] [Gemini 2.0 Flash model card](https://ai.google.dev/gemini-api/docs/models#gemini-2.0-flash) — capabilities, context window
- [2025-02] [OpenAI Realtime API](https://platform.openai.com/docs/guides/realtime) — WebSocket protocol, audio format
- [2025-04] [OpenAI Realtime model pricing](https://openai.com/api/pricing/) — audio token costs
- [2024-10] [GPT-4o Realtime launch](https://openai.com/index/introducing-the-realtime-api/) — original announcement
- [2024-12] [Gemini Live launch](https://blog.google/products/gemini/google-gemini-update-december-2024/) — video input support
- [2024-06] [Florence-2](https://huggingface.co/microsoft/Florence-2-base) — Microsoft perception model
- [2024-09] [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) — streaming local STT
