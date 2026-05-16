# CapsLockX 2.0

> Hold `CapsLock` or `Space` + a key. Navigate without leaving home row, dictate by voice, let an LLM drive your computer.

**CapsLockX** is a cross-platform keyboard productivity layer in Rust. The original AutoHotkey 1.x script (Windows only) lives on in the [`legacy` branch](https://github.com/snolab/CapsLockX/tree/legacy); the `main` branch here is the 2.0 ground-up rewrite that runs on macOS, Linux, and Windows.

- 🌐 [capslockx.com](https://capslockx.com) — landing site
- 📖 [snolab.github.io/CapsLockX](https://snolab.github.io/CapsLockX/) — docs (multi-language)
- 📜 [Core trigger behaviors spec](https://snolab.github.io/CapsLockX/#/./2026-05-15-core-behaviors.html)

[![GitHub license](https://img.shields.io/github/license/snolab/CapsLockX)](./LICENSE.md)
![GitHub top language](https://img.shields.io/github/languages/top/snolab/CapsLockX)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/snolab/CapsLockX)
[![GitHub stars](https://img.shields.io/github/stars/snolab/CapsLockX)](https://github.com/snolab/CapsLockX/stargazers)
![GitHub release](https://img.shields.io/github/v/release/snolab/CapsLockX)
![GitHub all releases](https://img.shields.io/github/downloads/snolab/CapsLockX/total)

---

## What it does

| Trigger | Cluster | Result |
|---|---|---|
| `Space+HJKL` | Editing | Vim-style cursor motion (← ↓ ↑ →) |
| `Space+YUIO` | Editing | Home / PageDown / PageUp / End |
| `Space+G` / `Space+T` / `Space+N`/`P` | Editing | Enter / Delete / Tab / Shift+Tab |
| `Space+WASD` / `QE` / `RF` | Mouse | Move (accelerated) / left+right click / scroll up+down |
| `Space+Z` / `X` / `C` | Windows | Cycle / close / tile (Shift modifies) |
| `Space+1`–`9` | Windows | Switch virtual desktop |
| `Space+V` | **AI** | Voice dictation (local STT + optional LLM polish) |
| `Space+B` | **AI** | Brainstorm chat overlay |
| `Space+M` | **AI** | LLM agent operates the UI |
| `Space+CapsLock` (chord) | — | Lock into CLX mode until tapped again |
| `Esc` | — | Dismiss overlays / kill running agent |

CapsLock and Space are fully interchangeable as triggers. See the [trigger behaviors spec](https://snolab.github.io/CapsLockX/#/./2026-05-15-core-behaviors.html) for every edge case.

---

## Platforms

| Platform | Status | Adapter |
|---|---|---|
| **macOS** (Apple Silicon & Intel) | ✅ Available | `rs/adapters/macos` — CGEventTap + AppKit, code-signed binary |
| **Linux** (Wayland & X11) | ✅ Available | `rs/adapters/linux` — evdev + uinput |
| **Windows** (1.x AHK) | ✅ Available | [`legacy` branch](https://github.com/snolab/CapsLockX/tree/legacy) |
| **Windows** (2.x Rust) | 🚧 In progress | `rs/adapters/windows` |
| **Browser** (WASM demo) | 🧪 Experimental | `rs/adapters/browser` |

---

## Quick start

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
clx                 # forks to background
```

Grant **Accessibility** permission when prompted (System Settings → Privacy & Security → Accessibility). For `Space+V` voice features, also grant **Microphone**.

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
clx
```

### Windows 1.x (AutoHotkey)

```powershell
irm https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.ps1 | iex
```

See [docs/installation.md](./docs/installation.md) for portable zips, Chocolatey, npm, and source builds.

---

## AI features

### `Space+V` — Voice dictation

Hold to talk push-to-talk style, tap to leave it on. The pipeline:

1. **STT** — SenseVoice (local) or whisper.cpp; Gemini cloud as fallback. Mic and system-audio tracks transcribed in parallel.
2. **Polish chain** — short utterances (≤15 chars or ≤5 s) return raw to avoid LLM-induced CER regressions; longer dictation flows through MLX (Qwen 2.5-3B local) or an LLM corrector.
3. **Translation** (optional) — learning / interpreter / chat / conversation presets.
4. **TTS** — ElevenLabs → Gemini → OpenAI → msedge-tts → native, with automatic fallback.

### `Space+B` — Brainstorm chat

Streaming overlay panel to Gemini / OpenAI / Anthropic / Ollama / MLX. History preserved between sessions.

### `Space+M` (or `clx agent`) — LLM agent

Describe a task; the agent reads your screen (Apple Vision OCR + AX tree) and operates the UI via the CLX command language:

```
k a              tap key 'a'
k c-c            Ctrl+C   (s=shift, c=ctrl, a=alt, w=cmd; macOS w=Cmd)
k "text"         type a string
m 400 300 c      move mouse to (400, 300) + click
wf "Save" 3s     wait up to 3 s for "Save" in the AX tree
scan ID x y w h dark>N { k key }   60 Hz pixel reflex
```

CLI without entering FN mode:

```bash
clx agent --tree                    # AX tree of frontmost app
clx agent --prompt "open notes and write today's date"
clx dino                            # auto-play Chrome Dino
clx observe                         # screenshot + Gemini Vision description
clx ocr                             # Apple Vision OCR
```

The system prompt is editable at [`skills/clx-agent/SKILL.md`](./skills/clx-agent/SKILL.md) — no rebuild needed.

---

## Architecture

```
rs/
  core/                    platform-agnostic engine + modules
    src/engine.rs          key router (trigger detection, bypass, chord lock)
    src/modules/           edit, mouse, media, voice, brainstorm, agent, window_manager, virtual_desktop
    src/platform.rs        Platform trait — adapters implement this
    src/llm_client.rs      multi-provider streaming (Gemini, OpenAI, Anthropic, Ollama, MLX)
  adapters/
    macos/                 CGEventTap hook + CGEventPost output + AppKit FFI
    linux/                 evdev hook + uinput output
    windows/               WH_KEYBOARD_LL + SendInput (WIP)
    browser/               WASM + Web APIs (experimental)
lib/
  otoji/                   voice/STT/TTS engine (submodule)
  capslockx.com/           landing site (submodule)
docs/                      docsify site, mirrored to snolab.github.io/CapsLockX
skills/clx-agent/          agent system prompt + tool docs (editable at runtime)
```

---

## Build from source

### macOS (recommended path — always use `./build.sh`)

```bash
./build.sh
# Equivalent to:
#   cd rs && cargo build -p capslockx-macos --release
#   cp target/release/capslockx ../clx
#   codesign -s - --force --identifier "com.snomiao.capslockx" ../clx
```

Code-signing with the stable identifier `com.snomiao.capslockx` makes Accessibility and Screen Recording permissions persist across rebuilds.

```bash
./build.sh && pkill -f "CapsLockX/clx" || true; clx
```

### Linux

```bash
cd rs && cargo build -p capslockx-linux --release
sudo target/release/capslockx-linux        # uinput needs root or input group
```

### Windows (2.x Rust, WIP)

```powershell
cd rs && cargo build -p capslockx-windows --release
```

Binary: `rs/target/release/clx-rust.exe`.

---

## Docs in your language

- 🇬🇧 **[English](./docs/README.en.md)** (recommended)
- 🇨🇳 **[简体中文](./docs/README.zh.md)**
- 🇯🇵 **[日本語](./docs/README.ja.md)**
- 🇫🇷 **[Français](./docs/README.fr.md)**
- 🇪🇸 **[Español](./docs/README.es.md)**
- 🇷🇺 **[Русский](./docs/README.ru.md)**

The docs site at [snolab.github.io/CapsLockX](https://snolab.github.io/CapsLockX/) has a sidebar, search, and deeper material on architecture / agent / roadmap.

---

## Contributing

PRs welcome. Before opening one:

- Read [`CLAUDE.md`](./CLAUDE.md) for the project's conventions (build, voice rules, agent language, macOS permission hazards).
- For Rust changes: `cargo fmt && cargo clippy --workspace --all-targets` should be clean.
- For UI/voice features: include a brief test plan since unit tests don't cover them.

Issues and feature requests: <https://github.com/snolab/CapsLockX/issues>.

---

## License

[GPL-3.0](./LICENSE.md) © snolab / snomiao.

The `lib/capslockx.com` landing-page submodule is MIT.

---

## Acknowledgements

- AutoHotkey, where it all started.
- [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) for the local STT model.
- [Karabiner-Elements](https://karabiner-elements.pqrs.org/) and [Hammerspoon](https://www.hammerspoon.org/) for showing what's possible at the macOS HID layer.
- Everyone who hit `CapsLock` once too many times and thought *"there has to be a better use for this key."*
