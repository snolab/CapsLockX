# Mic Echo Cancellation — Isolating User Voice from System Audio

## Problem

When system audio plays through speakers (YouTube, music, video calls), the
microphone picks it up and STT transcribes both. We want **only the user's
voice** in the mic track, even when speakers are loud.

Hard constraint from `CLAUDE.md`: **never gate, suppress, or silence the mic
track**. Both mic and sys tracks must always produce output. So the solution
must be signal-level isolation (subtract the system audio from the mic),
not "stop recording when sys is playing."

## Approaches

### 1. macOS VoiceProcessingIO (VPIO) — recommended baseline

CoreAudio's `kAudioUnitSubType_VoiceProcessingIO` AudioUnit. Same engine used
by FaceTime, Zoom, Teams. Open the mic through VPIO instead of the default
AUHAL/HAL input, and the OS automatically references the speaker output and
performs Acoustic Echo Cancellation (AEC), noise suppression, and AGC.

**Pros**
- Zero extra capture: no need to tap system audio separately.
- Hardware-tuned, low latency (~10–20 ms).
- Free — built into macOS.

**Cons**
- Forces a "voice call" frequency response: thinner, more compressed mic tone.
- Sample-rate constraints (typically 16 kHz or 24 kHz mono).
- Output level is much quieter post-AEC — needs ~20–30× makeup gain.
- AGC and NS are bundled; turning them off individually requires extra
  AudioUnit properties (`kAUVoiceIOProperty_BypassVoiceProcessing` etc.).

**Status:** demo implemented at
`rs/adapters/macos/src/bin/test-vpio.rs`. See [Demo](#demo) below.

### 2. ScreenCaptureKit + WebRTC AEC

macOS 13+ `SCStream` captures system-audio-only (already used as the `sys`
track in dual-track STT). Feed that as the **reference signal** to a software
AEC such as the WebRTC Audio Processing Module (`webrtc-audio-processing`
crate, AEC3 inside).

**Pros**
- Mic tone preserved (no VPIO color).
- AEC strength tunable per track.
- Reference is the actual output mix — works for any output device.

**Cons**
- Heavier: extra capture path + per-frame AEC processing.
- Synchronization is non-trivial: need to align speaker-out → mic-in delay
  (typically 30–80 ms; varies by output device).
- More CPU, more code paths to maintain.

### 3. ML denoise (RNNoise / DeepFilterNet)

Learned models that suppress "non-speech." Does **not** distinguish "speech
from speakers" vs "speech from user," so a YouTube narrator still bleeds
through. Useful as a stack on top of #1 or #2 for ambient/BGM noise, not as
a replacement.

Rust crates: `nnnoiseless` (RNNoise), DeepFilterNet bindings.

### 4. Hardware

Headset, AirPods (built-in AEC), directional mic. Out of scope here, but the
"correct" answer for many users.

## Recommendation

**Two-stage adoption:**
1. Switch the PTT/voice mic capture to **VPIO** (approach 1) — biggest
   reduction in bleed for the least code.
2. If users complain about mic tone, layer **SCK + WebRTC APM** (approach 2)
   as a `voice.aec_quality = "high"` preference.

## Demo

`test-vpio` opens VPIO, captures mic, prints a live RMS bar, and runs
Whisper transcription every 3 seconds. Use it to A/B test:

```bash
cd rs && cargo build -p capslockx-macos --bin test-vpio --features full --release
./target/release/test-vpio
```

**Test plan**
1. Silent room → bar near zero, transcripts say `(empty/noise)`.
2. Speak only → bar moves, transcripts contain your words.
3. Play YouTube on speakers, do NOT speak → bar should stay low; transcripts
   should be empty/noise. This is the AEC working.
4. Play YouTube on speakers AND speak over it → only your words should appear
   in the transcript; the YouTube narrator should be largely absent.

If step 3 leaks the YouTube audio: AEC is not engaging — check that
`AudioOutputUnitStart` returned `0` and that the speaker-out stream actually
runs through the same default device VPIO is referencing.

If step 4 captures both: VPIO is in pass-through; verify the component
sub-type is `'vpio'` and not `'ahal'`.

## Implementation Notes

- VPIO bus layout: input bus = 1 (mic), output bus = 0 (speaker reference).
  Enable input via `AudioUnitSetProperty(kAudioOutputUnitProperty_EnableIO,
  scope=Input, element=1, value=1)`.
- The render callback may receive a 9-channel non-interleaved buffer on some
  hardware (Apple Silicon MacBook Pro internal mic). The demo falls back from
  1 ch to 9 ch and reads channel 0. Real integration should detect this once
  during init via `kAudioUnitProperty_StreamFormat`.
- Apply ~30× linear gain after AEC, then clamp to `[-1.0, 1.0]`. Without
  makeup gain, downstream VAD will treat everything as silence.
- VPIO ducks system audio by default. Suppress with property `2108`
  (`kAUVoiceIOProperty_DuckOthers`) set to `level=10, advanced=0` so the user
  can still hear what they're playing while VPIO captures.

## Privacy / Constraints

- The mic stream is still always-on; AEC removes echo, it does not gate.
- Both mic and sys tracks remain independent — sys track is unaffected.
- No user voice samples are baked into config or shipped to the cloud
  outside the normal STT path.
