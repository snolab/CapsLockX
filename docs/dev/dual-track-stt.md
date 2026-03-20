# Dual-Track STT Architecture

## Overview

CapsLockX supports simultaneous transcription of microphone audio and system audio
(what the computer is playing). This enables use cases like:
- Meeting transcription with speaker labels (🎤 you, 🔊 remote participants)
- Voice notes that capture both your commentary and the content you're watching
- Dual-language transcription (you speak Japanese, system plays English)

## Activation

- **Space+V** — Mic only (hold to type, click to toggle voice note)
- **Shift+Space+V** — Dual capture: mic + system audio with echo cancellation

## Architecture

```
                    ┌──────────┐
User speaks ───────→│ Mic      │──→ NLMS Echo Cancel ──→ VAD ──→ Queue ──┐
                    │ (VPIO)   │         ▲                                │
                    └──────────┘         │                                ▼
                                         │ (reference)         ┌──────────────┐
                    ┌──────────┐         │                     │  STT Worker  │
Computer plays ────→│ System   │─────────┘                     │  Thread      │
                    │ Audio    │──→ VAD ──→ Queue ─────────────→│              │
                    │(SCKit)   │                                │ SenseVoice   │
                    └──────────┘                                └──────┬───────┘
                                                                       │
                                                                ┌──────▼───────┐
                                                                │ LLM Corrector│
                                                                │ (Gemini)     │
                                                                └──────┬───────┘
                                                                       │
                                                                🎤/🔊 output
```

## Audio Capture

### Microphone
- **Primary**: VoiceProcessingIO (VPIO) AudioUnit — hardware echo cancellation
  - 9-channel non-interleaved format, 1-buffer mono render
  - Built-in AEC removes ~95% of system audio bleed
  - Ducking minimized via kAUVoiceIOProperty_OtherAudioDuckingConfiguration
- **Fallback**: cpal AudioCapture (cross-platform, no AEC)

### System Audio
- ScreenCaptureKit (macOS 13+) via ObjC block FFI
- Captures all system audio output (not just one app)
- Used as NLMS reference signal AND transcribed separately

## Echo Cancellation (Triple-Layer)

1. **VPIO Hardware AEC** — AudioUnit-level, removes ~95% of echo
2. **NLMS Adaptive Filter** — 1600 taps at 16kHz (100ms), mu=0.3
   - Applied to raw VPIO output BEFORE gain amplification (critical ordering)
   - Uses system audio as reference signal
3. **Noise Gate** — 0.002 threshold, removes residual low-level bleed

## STT Worker Thread

The audio loop is non-blocking — it only captures, resamples, runs echo
cancellation, and performs VAD. Complete speech segments are pushed to a
bounded channel (`sync_channel(10)`) for processing by the STT worker.

### Backpressure Handling
- Channel capacity: 10 segments
- When full: oldest segment dropped (stale audio is less valuable)
- `try_send` in audio loop — never blocks
- Worker processes mic segments with priority over sys segments

### Processing Pipeline (per segment)
1. Receive segment from channel (tagged with `Mic` or `Sys`)
2. Pad to 1s minimum (SenseVoice requirement)
3. Transcribe via SenseVoice (~30-60ms per segment)
4. Apply LLM correction if enabled (Gemini 2.5 Flash, ~100-200ms)
5. Update overlay subtitle
6. Type at cursor (input mode) or write to SRT (note mode)

## LLM-Based STT Correction

Optional post-processing that fixes SenseVoice recognition errors using an LLM.

### How It Works
- Maintains a conversation context with the LLM
- Each committed STT segment is appended as a user message
- LLM returns corrected text (fixes homophones, garbled words)
- **Incremental**: LLM KV cache is reused for prior context — only new tokens cost

### Configuration
- Enable in Preferences → Voice → "LLM correction"
- Requires an API key (OpenAI, Anthropic, or Gemini)
- Default model: Gemini 2.5 Flash (fast, cheap, multilingual)

### System Prompt
```
You are a speech-to-text error corrector. Fix obvious STT mistakes
while preserving the speaker's intended meaning. Do NOT translate
between languages. Return ONLY the corrected text.
```

### Example
- Raw STT: `今日は私の事故紹介をします` (事故 = accident)
- Corrected: `今日は私の自己紹介をします` (自己 = self ✓)

### Context Management
- History trimmed at 40 messages to avoid context overflow
- Reset on new voice session (Space+V release/re-press)
- Separate contexts for mic and sys tracks

## Voice Overlay

- Floating transparent NSWindow at top-center of screen
- Dual waveform: green (mic, top) + blue (sys, bottom)
- Color-coded subtitle lines: 🎤 green background, 🔊 blue background
- Drag handle: appears on mouse hover (⠿ grip on left edge)
- Hidden from screen sharing (`setSharingType: NSWindowSharingNone`)

## Voice Note Output

When in voice note mode (click Space+V):
- **WebM audio**: Streaming via ffmpeg stdin pipe, dual-channel (mic L, sys R)
- **SRT subtitles**: Speaker-labeled (`🎤`/`🔊`), timestamped
- Saved to `~/CapsLockX-Voice/` with date-based filenames

## Benchmark Results (SenseVoice vs Whisper)

| Engine | Size | EN Accuracy | JA Accuracy | KO Accuracy | Speed |
|---|---|---|---|---|---|
| **SenseVoice** | 486MB | **95.6%** | **94.7%** | 71.6% | 37x RT |
| Whisper-tiny | 75MB | 92.8% | 80.5% | 65.6% | 44x RT |
| Whisper-small | 466MB | 95.6% | 87.2% | 68.0% | 7.5x RT |
| Whisper-large-v3 | 3GB | 95.8% | 76.7% | 83.8% | 1.4x RT |

SenseVoice matches Whisper-small accuracy at 5x the speed, and crushes it on Japanese.

## Key Files

- `rs/core/src/modules/voice.rs` — Voice pipeline (VAD, STT worker, NLMS, SRT)
- `rs/core/src/local_sherpa.rs` — SenseVoice wrapper (sherpa-rs)
- `rs/core/src/local_whisper.rs` — Whisper wrapper (whisper-rs)
- `rs/core/src/llm_client.rs` — Multi-provider LLM client (OpenAI/Anthropic/Gemini)
- `rs/core/src/stt_corrector.rs` — Incremental LLM-based STT correction
- `rs/adapters/macos/src/voice_capture.rs` — VPIO AudioUnit with AEC
- `rs/adapters/macos/src/system_audio.rs` — ScreenCaptureKit system audio
- `rs/adapters/macos/src/voice_overlay.rs` — Floating subtitle overlay

## Common Issues

- `-10877` on VPIO format set → format is read-only, accept defaults
- `rms=0.0000` on mic → try 9ch buffer or check AudioUnitInitialize order
- English leaking on mic → increase NOISE_GATE or NLMS mu parameter
- Speakers muted → check ducking config (property 2108, level should be 10)
- STT correction slow → reduce LLM model size or disable correction
- Queue full → STT worker can't keep up, segments dropped (logged)
