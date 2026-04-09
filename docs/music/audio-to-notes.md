# Audio-to-Notes / Audio-to-MIDI Libraries & Tools

Survey of libraries for converting audio (humming, piano, singing) into musical notes/MIDI.

## Rust Crates

| Crate | Description | Mono/Poly | Real-time | License |
|-------|-------------|-----------|-----------|---------|
| **pitch-detection** | McLeod pitch method (autocorrelation). Fast, O(NlogN). ~39k downloads. | Mono | Yes | MIT |
| **pitch-detector** | Frequency + note detector, returns note name + cents offset. | Mono | Yes | MIT |
| **pitch** | Bitstream Autocorrelation (BCF). Pure Rust, no deps. | Mono | Yes | MIT |
| **pyin** / **pyin-rs** | Probabilistic YIN, based on librosa. Returns F0 per frame + voicing probability. | Mono | Offline | MIT |
| **aubio** (crate) | Safe Rust bindings for the aubio C library. Pitch, onset, tempo. | Mono | Yes | GPL-3.0 |
| **midly** | MIDI file encoder/decoder (~204k downloads). Essential for writing .mid output. | N/A | N/A | MIT |

**Polyphonic gap**: No Rust crate does polyphonic transcription natively. Strategy: run basic-pitch ONNX model via the `ort` crate + `midly` for MIDI output.

## Python Libraries

| Library | Mono/Poly | Real-time | Best For | License |
|---------|-----------|-----------|----------|---------|
| **[basic-pitch](https://github.com/spotify/basic-pitch)** (Spotify) | **Poly** | Near-RT | General audio-to-MIDI, any instrument. CLI: `basic-pitch output/ input.wav` | Apache-2.0 |
| **[CREPE](https://github.com/marl/crepe)** | Mono | Yes | Vocal/humming pitch tracking. Deep CNN on raw waveform. Also: `torchcrepe` (PyTorch). | MIT |
| **[librosa](https://github.com/librosa/librosa)** (pyin) | Mono | Offline | `librosa.pyin()` for probabilistic YIN. General audio analysis. | ISC |
| **[madmom](https://github.com/CPJKU/madmom)** | Mono/Poly | Offline | Beat/onset/tempo detection. Neural models. | BSD / CC-BY-NC-SA |
| **[omnizart](https://github.com/Music-and-Culture-Technology-Lab/omnizart)** | **Poly** | Offline | All-in-one: piano, vocal, drums, chords, beats. CLI: `omnizart music transcribe input.wav` | MIT |
| **[piano_transcription_inference](https://github.com/qiuqiangkong/piano_transcription_inference)** | **Poly** | Offline | Highest accuracy solo piano + pedal detection. | MIT |
| **[MT3](https://github.com/magenta/mt3)** (Google Magenta) | **Poly, multi-instrument** | Offline | T5 Transformer. Transcribes multiple instruments simultaneously. | Apache-2.0 |
| **[Onsets and Frames](https://github.com/magenta/magenta)** (Google) | **Poly** | Offline | CNN+LSTM piano transcription. Foundational AMT research model. | Apache-2.0 |

## C/C++ Libraries

| Library | Mono/Poly | Real-time | Notes | License |
|---------|-----------|-----------|-------|---------|
| **[aubio](https://aubio.org/)** | Mono | Yes | Pitch (YIN, McLeod), onset, beat, tempo. Mature, widely used. Has Rust bindings. | GPL-3.0 |
| **[Essentia](https://github.com/MTG/essentia)** | Mono + some Poly | Both | Comprehensive MIR from UPF Barcelona. TensorFlow integration. | AGPL-3.0 |
| **[basicpitch.cpp](https://github.com/sevagh/basicpitch.cpp)** | **Poly** | Offline | C++20 port of Spotify's basic-pitch using ONNX Runtime. Also has WASM build. CLI: `./basic-pitch input.wav output.mid` | MIT |
| **[pYIN](https://github.com/c4dm/pyin)** (Vamp plugin) | Mono | Yes | Probabilistic YIN from QMUL. Vamp plugin for Audacity/Sonic Visualiser. | GPL |
| **[Tony](https://github.com/sonic-visualiser/tony)** | Mono | Offline | Melody annotation GUI tool using pYIN. From QMUL. | GPL |

## JavaScript / Web

| Library | Mono/Poly | Real-time | Notes |
|---------|-----------|-----------|-------|
| **[@spotify/basic-pitch](https://github.com/spotify/basic-pitch-ts)** | **Poly** | Near-RT | Full TypeScript port. TF.js. `npm i @spotify/basic-pitch`. Apache-2.0. |
| **[Pitchy](https://www.npmjs.com/package/pitchy)** | Mono | Yes | McLeod pitch method. Pure JS, lightweight. MIT. |
| **[Pitchfinder](https://github.com/peterkhayes/pitchfinder)** | Mono | Yes | Collection: YIN, AMDF, ACF2+, Macleod, dynamic wavelet. MIT. |
| **[ml5.js](https://github.com/ml5js/ml5-library)** pitch | Mono | Yes | CREPE model via TensorFlow.js. Easy p5.js integration. MIT. |
| **[Meyda](https://meyda.js.org/)** | Mono | Yes | Audio feature extraction (MFCC, spectral, chroma). Building block. MIT. |

## Pre-trained ML Models

| Model | Architecture | Best For |
|-------|-------------|----------|
| **Basic Pitch** (Spotify) | Lightweight CNN | General audio-to-MIDI, any instrument, pitch bends |
| **Onsets and Frames** (Google) | CNN + LSTM | Solo piano transcription |
| **MT3** (Google Magenta) | T5 Transformer | Full ensemble (piano, guitar, drums, etc.) |
| **High-Res Piano** (ByteDance/Kong) | CNN regression | Highest accuracy piano + pedal |
| **CREPE** | Deep CNN | Vocal/humming pitch tracking |
| **NeuralNote** | basic-pitch via RTNeural + ONNX | DAW plugin (VST3/AU), real-time |

## Command-Line Quick Reference

```bash
# Spotify basic-pitch (best general-purpose)
pip install basic-pitch
basic-pitch output_dir/ input.wav

# Omnizart (multi-mode: music, vocal, drum, chord)
pip install omnizart
omnizart music transcribe input.wav

# aubio (mono, real-time capable)
brew install aubio
aubionotes input.wav

# basicpitch.cpp (C++ standalone)
./basic-pitch input.wav output.mid
```

## Rust Integration Strategy

- **Monophonic** (humming/singing): `pitch-detection` or `pyin` crate. Pure Rust, real-time, no deps.
- **Polyphonic** (piano/chords): Run basic-pitch ONNX model via `ort` crate (ONNX Runtime bindings) + `midly` for MIDI output. Mirrors what `basicpitch.cpp` does in C++.
- **Quick hack**: Shell out to `basic-pitch` CLI (Python).

## Web Demo

Try it in browser: https://basicpitch.spotify.com/
