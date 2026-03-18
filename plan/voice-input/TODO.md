# Voice Input (CLX+V) — Implementation TODO

## Phase 1: Core Hotkey + Audio Capture
- [ ] `rs/core/src/modules/voice.rs` — VoiceModule skeleton
  - [ ] V key down/up handlers with toggle/hold state machine
  - [ ] Integrate into Modules dispatcher (engine.rs)
  - [ ] Track listening state (idle → listening → sending)
- [ ] `rs/adapters/macos/src/audio_capture.rs` — macOS mic capture via CoreAudio/cpal
  - [ ] Open default input device
  - [ ] Capture PCM f32 samples at 16kHz mono
  - [ ] Ring buffer for audio data
  - [ ] Start/stop capture on demand
- [ ] `rs/adapters/windows/src/audio_capture.rs` — Windows mic capture via WASAPI/cpal
  - [ ] Same interface as macOS
- [ ] Add `cpal` dependency to both platform adapters

## Phase 2: VAD (Voice Activity Detection)
- [ ] `rs/core/src/vad.rs` — VAD wrapper
  - [ ] Integrate `webrtc-vad` crate (lightweight, C library)
  - [ ] Detect speech start → buffer audio
  - [ ] Detect silence (>500ms) → emit audio chunk
  - [ ] Force-split at 25s to stay within Whisper's 30s limit
  - [ ] Return audio chunks as Vec<f32> or WAV bytes
- [ ] Add `webrtc-vad` dependency to core

## Phase 3: Server Endpoint
- [ ] `app/api/voice-transcribe/route.ts` — New streaming endpoint
  - [ ] Accept audio (WAV/MP3) + optional context prompt
  - [ ] Call Whisper API for transcription
  - [ ] Call gpt-4o-mini for typo-fix with streaming
  - [ ] Return NDJSON stream: `{stage, text, is_final}`
  - [ ] Handle 30s chunk stitching with context carry-over
- [ ] Deploy to brainstorm.snomiao.com

## Phase 4: HTTP Client + Text Output
- [ ] `rs/core/src/voice_client.rs` — HTTP client
  - [ ] Send audio chunk as multipart POST
  - [ ] Parse streaming NDJSON response
  - [ ] Emit transcription events (rough → refined → polished)
  - [ ] Configuration: server URL, API key
- [ ] Text output integration
  - [ ] On rough transcript: type text at cursor via Platform::key_tap
  - [ ] On refined: select previous rough text, replace with refined
  - [ ] On polished: select previous refined text, replace with polished
  - [ ] Track cursor position for replacement
- [ ] Add `reqwest` dependency (with streaming feature)

## Phase 5: Local Whisper (Optional, for low-latency draft)
- [ ] Evaluate `whisper-rs` crate (binds to whisper.cpp)
  - [ ] Download tiny.en model (~75MB)
  - [ ] Benchmark: can it transcribe <1s of audio in <500ms?
- [ ] If viable: run local model on VAD chunk immediately
  - [ ] Type rough draft at cursor
  - [ ] Server response replaces it later
- [ ] If not viable: skip this phase, rely on server only

## Phase 6: Polish & UX
- [ ] Audio feedback: beep on start/stop listening
- [ ] Tray icon state: show recording indicator
- [ ] Preferences UI: voice input settings (server URL, local model toggle)
- [ ] Handle network errors gracefully (retry, fallback to local)
- [ ] Language detection / multi-language support
- [ ] Configurable VAD sensitivity

## Phase 7: Platform Trait Extensions
- [ ] Add to Platform trait:
  ```rust
  fn start_audio_capture(&self) -> AudioStream;
  fn stop_audio_capture(&self);
  ```
- [ ] Or keep audio capture in a separate module that the VoiceModule owns

## Notes
- Use `cpal` for cross-platform audio — it abstracts CoreAudio (macOS) and WASAPI (Windows)
- WAV format for server upload (simple, lossless, Whisper accepts it)
- MP3 encoding optional (saves bandwidth but adds complexity — `lame` crate)
- Server already has Whisper integration at `/api/asr`, can reuse
- Each VAD segment should carry previous transcript as `prompt` for Whisper continuity
