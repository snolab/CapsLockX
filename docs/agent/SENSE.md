# SENSE — Input Signals to the Agent LLM

Everything the LLM can perceive. The agent is not just an output machine —
it has a rich sensory input loop. All sensor data is formatted for token
efficiency and streamed to the LLM as context.

## Overview

```
┌─────────────────────────────────────────────────────────┐
│                    SENSORY INPUTS                        │
│                                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │ Screen   │ │ Keyboard │ │  Mouse   │ │   Audio   │  │
│  │ Capture  │ │ Listener │ │ Listener │ │ Listener  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘  │
│       │             │            │              │        │
│  ┌────┴─────┐ ┌─────┴────┐ ┌────┴─────┐ ┌─────┴─────┐  │
│  │ Gamepad  │ │  MIDI    │ │  Window  │ │ Clipboard │  │
│  │ Listener │ │ Listener │ │  Info    │ │  Monitor  │  │
│  └────┬─────┘ └────┴────┘ └────┬─────┘ └─────┬─────┘  │
│       │             │           │              │        │
│       └─────────────┴───────────┴──────────────┘        │
│                         │                               │
│              ┌──────────▼──────────┐                    │
│              │   Signal Encoder    │                    │
│              │ (format for LLM)    │                    │
│              └──────────┬──────────┘                    │
│                         │                               │
│              ┌──────────▼──────────┐                    │
│              │     LLM Context     │                    │
│              │  (system prompt +   │                    │
│              │   sensor stream)    │                    │
│              └─────────────────────┘                    │
└─────────────────────────────────────────────────────────┘
```

## 1. Screen Capture (Vision)

### What the LLM sees
- Periodic screenshots sent as image tokens
- Configurable rate: 0.5-10 fps (higher = more tokens, more cost)
- Resolution: downscaled to 256x256 - 512x512 (balance detail vs tokens)
- Format: JPEG (lossy, small) or PNG (lossless, for text-heavy UIs)

### Delivery modes

**Periodic snapshots** (default):
```
[SCREEN t=1.2s 512x384 jpeg]
<base64 image data>
```

**Delta/diff mode** (token-efficient):
- Only send screen regions that changed since last frame
- Use perceptual hash to detect changed regions
- Send bounding boxes + cropped changed regions

**OCR mode** (most token-efficient):
- Run local OCR (Tesseract / Vision.framework) on screenshot
- Send text content + bounding box coordinates
- Example output to LLM:
```
[SCREEN OCR t=2.5s]
btn "Start Game" 400,300 200x50
txt "Score: 1250" 10,10
txt "Health: ███░░" 10,30
obj player 250,400 32x48
obj enemy 500,380 24x24
```

**Gemini Live mode** (real-time):
- Video frames streamed via WebSocket to Gemini Live API
- Model maintains visual context across frames
- Highest quality understanding but highest cost

### APIs
- macOS: ScreenCaptureKit (`SCStream`)
- Windows: DXGI Desktop Duplication
- Cross-platform: `xcap` Rust crate
- Local OCR: Apple Vision.framework, Windows OCR API, Tesseract

## 2. Keyboard Input (what the human types)

The LLM can observe the human's keyboard input to understand intent,
learn patterns, or provide assistance.

### Format (same language as LLM output)
```
[KEY t=0.00] k d:a         # human pressed 'a' down
[KEY t=0.05] k u:a         # human released 'a'
[KEY t=0.12] k c-s          # human pressed Ctrl+S
[KEY t=0.50] k "hello"     # burst typing detected, sent as string
```

### Modes
- **Raw mode**: Every key down/up event, timestamps
- **Cooked mode**: Combine rapid keystrokes into typed strings
- **Filtered mode**: Only report modifier combos and special keys
- **Off**: Don't report keyboard to LLM (privacy)

### Implementation
- Hook into CapsLockX's existing CGEventTap (macOS) / low-level hook (Windows)
- Filter out self-injected events (tagged with `SELF_INJECT_TAG`)
- Debounce rapid typing into string batches

## 3. Mouse Input (what the human does with mouse)

### Format
```
[MOUSE t=0.00] m 500 300          # position update
[MOUSE t=0.05] m +12 -3           # relative movement delta
[MOUSE t=0.10] m d                 # left button down
[MOUSE t=0.30] m u                 # left button up
[MOUSE t=0.30] m 520 295 c        # click at position (cooked)
[MOUSE t=0.50] m s -3             # scroll down 3
```

### Modes
- **Position mode**: Report absolute position at fixed rate (10-60 Hz)
- **Event mode**: Only report clicks, scroll, drag start/end
- **Delta mode**: Report relative movements (good for game context)
- **Off**: Don't report mouse to LLM

### Implementation
- CGEventTap with mouse event types (macOS)
- Low-level mouse hook (Windows)
- Downsample position updates to configurable rate

## 4. Gamepad Input (what the human does with controller)

### Format
```
[PAD t=0.00] p ls 0.45 -0.82     # left stick position
[PAD t=0.00] p rs 0.00 0.10      # right stick position
[PAD t=0.00] p lt 0.00           # left trigger
[PAD t=0.00] p rt 0.60           # right trigger
[PAD t=0.05] p d:a               # A button pressed
[PAD t=0.20] p u:a               # A button released
[PAD t=0.30] p dp right          # D-pad right
```

### Modes
- **Full mode**: All axes + buttons at polling rate
- **Event mode**: Only report button presses and significant stick changes
- **Stick-only mode**: Just analog stick positions (for driving/flying games)
- **Off**: Don't report gamepad

### Implementation
- macOS: GameController.framework (`GCController`)
- Windows: XInput API (`XInputGetState`)
- Rust: `gilrs` crate (cross-platform gamepad input)
- Dead zone filtering: don't report tiny stick movements

## 5. MIDI Input (what the human plays on MIDI controller)

### Format
```
[MIDI t=0.00] M n 60 100         # note on: C4, velocity 100
[MIDI t=0.20] M o 60             # note off: C4
[MIDI t=0.30] M cc 1 80          # mod wheel at 80
[MIDI t=0.35] M pb 10000         # pitch bend
```

Same language as MIDI output commands — bidirectional.

### Implementation
- `midir` crate: listen on all MIDI inputs
- Filter by device/channel if configured
- Timestamps from `mach_absolute_time` / QPC

## 6. Audio Input (voice, game sounds)

### Modes

**Voice transcription** (STT → text):
```
[VOICE t=1.2s] "jump over that obstacle"
[VOICE t=3.5s] "go left, there's a power-up"
```
- Uses CapsLockX's existing STT (Whisper/SenseVoice)
- Most token-efficient — voice becomes text

**Audio level/features** (for game awareness):
```
[AUDIO t=0.00] level=0.3 freq=440     # ambient sound
[AUDIO t=0.50] level=0.9 freq=1200    # loud event (explosion?)
[AUDIO t=1.00] music=true tempo=120    # background music detected
```
- Useful for rhythm games (detect beat)
- Detect audio cues (enemy footsteps, alerts)

**Raw audio stream** (Gemini Live API):
- Stream PCM audio directly to Gemini Live API
- Model hears game sounds + human voice simultaneously
- Highest bandwidth but most capable

### Implementation
- STT: Existing CapsLockX voice module
- Audio features: `cpal` crate for audio capture + simple FFT
- Gemini Live: PCM 16-bit 16kHz via WebSocket

## 7. Window / Application Context

### Format
```
[APP] name="Safari" title="GitHub - snolab/CapsLockX"
[APP] name="RetroArch" title="Super Mario Bros" fullscreen=true
[APP] name="Ableton Live" title="My Song - 120 BPM"
```

### What it provides
- Active application name and window title
- Fullscreen state
- Window position and size
- Helps LLM understand context without vision

### Implementation
- macOS: `NSWorkspace.shared.frontmostApplication`, Accessibility API
- Windows: `GetForegroundWindow` + `GetWindowText`
- Already partially implemented in CapsLockX

## 8. Clipboard Monitor

### Format
```
[CLIP t=5.2s text] "https://example.com/page"
[CLIP t=8.0s image 800x600]
```

### Use cases
- LLM can see what user copied (for context)
- Detect URL copies → auto-navigate
- Detect code copies → auto-format/execute

### Implementation
- macOS: `NSPasteboard.general` polling or change count
- Windows: clipboard listener (`AddClipboardFormatListener`)

## 9. Time Sense

Time is a first-class sensor. The LLM must know when things happen to
coordinate actions, measure durations, and react to timing.

### Format
```
[T 0.000]                              # elapsed time since session start
[T 1.234] [KEY] k a                    # every event is timestamped
[T 2.500] [SCREEN OCR] txt "Score: 10" 10,10
```

Every signal message includes `[T elapsed_seconds]` as prefix. The LLM
can compute durations between events, measure its own reaction time,
and schedule future actions.

### Time-related signals
```
[TIME] now=2025-03-23T14:30:00.123 tz=Asia/Tokyo
[TIME] session=45.2s                   # seconds since agent started
[TIME] idle=3.5s                       # seconds since last human input
[TIME] fps=59.8                        # game/app framerate estimate
[TIME] latency=180ms                   # last LLM round-trip time
```

### Why time matters
- **Reaction timing**: LLM can learn "I see the obstacle at T=1.0, I need to jump by T=1.3"
- **Idle detection**: "No human input for 5s, maybe they stepped away"
- **Rhythm**: Music games, timed puzzles need beat-accurate awareness
- **Self-awareness**: LLM knows its own latency and can compensate ("I'm 200ms slow, act earlier")
- **Scheduling**: "At T=10.0, the power-up respawns — be there"
- **Elapsed tracking**: "I've been running right for 2s, should be near the edge"

### Implementation
- `std::time::Instant` at session start, all timestamps relative
- Include `[T ...]` prefix on every signal message automatically
- Periodic `[TIME]` system clock messages (every 5-10s)
- FPS estimation from screen capture frame intervals
- Latency measurement from LLM request/response timing

## 10. System State

### Format
```
[SYS] os=macos cpu=12% mem=8.2GB/16GB battery=85% wifi=connected
[SYS] display=2560x1600@60Hz scale=2x
[SYS] agent_load=low tokens_used=1250 tokens_budget=5000/s
```

### Use cases
- Display info for coordinate calculations
- Battery/performance awareness
- **Agent self-monitoring**: token budget remaining, system load

## Adaptive Sensing — LLM Controls Its Own Perception

**Key principle**: Sensors start at minimum detail. The LLM actively
escalates when it needs more information, and de-escalates to save tokens.

### Default startup state (minimal token cost)
```
Screen:   OCR mode, 0.5 fps, 256x256       ~25 tok/s
Keyboard: event mode (specials only)        ~2 tok/s
Mouse:    moved/idle flag only              ~1 tok/s
Gamepad:  off                               0
MIDI:     off                               0
Voice:    STT transcription                 ~5 tok/s (when speaking)
Audio:    off                               0
App:      on (change events only)           ~1 tok/s
Clipboard: off                              0
Time:     always on (prefix on every msg)   ~0 (included in other signals)
─────────────────────────────────────────────
Total default:                              ~30 tok/s
```

### LLM-driven escalation examples

**Desktop automation task** (open app, type, click):
```
S screen mode ocr fps 1 res 512     # need to read UI text
S mouse mode event                   # need to see clicks landing
# ... do the task ...
S all low                            # done, back to minimum
```

**Game playing** (platformer):
```
S screen mode yolo fps 5 res 512    # need object positions fast
S pad mode full 30                   # reading my own stick output
S key off                            # not using keyboard
S audio mode level 10                # detect sound cues
# ... play the game ...
S screen fps 0.5                     # game paused, save tokens
```

**Music/MIDI performance**:
```
S midi on                            # need to hear what's playing
S audio mode fft                     # frequency analysis for harmony
S screen off                         # don't need visuals
S key off
S mouse off
# ... perform ...
```

**Idle/monitoring** (watching for something):
```
S all low                            # bare minimum
S screen mode ocr fps 0.1 res 256   # check once every 10s
S app on                             # notify on app switch
# ... wait ...
# LLM sees something interesting
S screen mode image fps 2 res 768   # escalate to see details
```

### Token budget enforcement
The runtime enforces a total token budget (configurable, default 500 tok/s).
If the LLM's sense configuration would exceed the budget:
1. Log a warning in the signal stream: `[WARN] token budget exceeded, capping screen fps`
2. Automatically reduce the highest-cost sensor's frequency
3. LLM can respond by explicitly prioritizing: `S screen fps 5; S key off; S mouse off`

### On-demand queries (`?`) — cheapest option
Instead of continuous streaming, the LLM can request a single reading:
```
? screen                    # one screenshot now
? screen ocr               # OCR this frame
? screen yolo              # YOLO detections this frame
? mouse                    # current mouse position
? pad                      # current gamepad state
? time                     # current time + session elapsed
? sys                      # system load, token usage
```

This costs zero tokens/s when not querying — the LLM only pays when it
asks. Best for low-frequency checks ("has the screen changed?").

## Signal Encoding Strategy

### Token budget (adaptive — LLM controls via `S` commands)

| Signal | Default (low) | Escalated (high) | Tokens/update |
|---|---|---|---|
| Screen (OCR, 0.5fps) | 25 tok/s | 250 tok/s (image, 5fps) | 20-400 |
| Time | ~0 (prefix) | ~0 | 3-5 per message |
| Keyboard (event) | 2 tok/s | 10 tok/s (raw) | 3-5 |
| Mouse (moved flag) | 1 tok/s | 15 tok/s (pos 30Hz) | 2-4 |
| Gamepad | 0 (off) | 20 tok/s (full 30Hz) | 3-6 |
| MIDI | 0 (off) | 10 tok/s | 3-4 |
| Voice (STT) | 5 tok/s | 20 tok/s (audio) | variable |
| Audio | 0 (off) | 10 tok/s (fft) | 5-10 |
| App context | 1 tok/s | 1 tok/s | 5-10 |
| **Default total** | **~30 tok/s** | | |
| **Max escalated** | | **~340 tok/s** | |
| **Budget cap** | | **500 tok/s** (configurable) | |

### Prioritization
When total token budget is exceeded, prioritize:
1. Screen (vision) — most information-dense
2. Voice commands — direct human intent
3. Input events — context for what's happening
4. System state — least urgent

### Unified signal stream
All signals merge into a single chronological stream sent to the LLM:
```
[SCREEN OCR t=0.0] btn "Play" 400,300; txt "Menu" 380,250
[KEY t=0.2] k space
[SCREEN OCR t=1.0] txt "Level 1" 300,100; obj player 100,400
[PAD t=1.1] p ls 1.0 0.0
[VOICE t=1.5] "go right and jump"
[SCREEN OCR t=2.0] obj player 300,400; obj gap 400,380
```

The LLM sees this as a real-time feed and responds with commands in the
same language:
```
p ls 1.0 0.0
w 200ms
p d:a
w 100ms
p u:a
```
