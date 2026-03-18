# Voice Modes: Note vs Input

## Two independent features that can run simultaneously

### Voice Note (click Space+V)
- Toggle on/off with quick tap (<300ms)
- Records audio continuously to `~/.capslockx/voice/[timestamp].wav`
- Transcribes to `~/.capslockx/voice/[timestamp].srt` (SRT subtitle format with timestamps)
- Shows last sentence in transparent overlay at top of screen
- Does NOT type into any app
- Can run for hours (meeting recording, lecture, etc.)
- Click Space+V again to stop recording

### Voice Input (hold Space+V >300ms)
- Starts on V down, stops on V release
- Transcribes and types at cursor (current behavior)
- Does NOT interfere with voice note session
- If voice note is running, voice input reuses the same audio stream
- On release: stop typing, voice note continues if it was running

### Key Insight: Both share the same audio capture
- One AudioCapture instance runs whenever either mode is active
- Voice Note: saves raw audio + generates SRT
- Voice Input: runs streaming transcription + types at cursor
- Both read from the same audio buffer

### Architecture
```
                    ┌─ Voice Note: save .wav + .srt + show overlay
AudioCapture ──→ VAD ──┤
                    └─ Voice Input: streaming transcribe + type at cursor
```

### State Machine
```
[Idle]
  ├─ V down → start audio capture + VAD
  │   ├─ V up <300ms → enter NOTE mode (toggle on)
  │   │   └─ Next V click → exit NOTE mode (toggle off)
  │   └─ V up >300ms → INPUT session done, check if NOTE was running
  │       ├─ NOTE was running → keep audio capture alive
  │       └─ NOTE was not running → stop audio capture
  │
[Note Running]
  ├─ V down → start INPUT overlay
  │   ├─ V up <300ms → toggle NOTE off, stop recording
  │   └─ V up >300ms → INPUT done, NOTE continues
```
