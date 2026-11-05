# CapsLockX 2.0

> Hold CapsLock or Space + a key. Navigate without leaving home row, talk to your computer, let an LLM drive it for you.

CapsLockX is a cross-platform keyboard productivity layer, rewritten in Rust. Press and hold `CapsLock` or `Space` as a trigger, then a second key fires an action — vim-style cursor motion, mouse simulation, window management, virtual desktops, voice dictation, or an AI agent that operates your computer.

| Platform | Status | Notes |
|---|---|---|
| **macOS** (Apple Silicon & Intel) | ✅ Available | CGEventTap + AppKit, code-signed binary |
| **Linux** (Wayland & X11) | ✅ Available | evdev + uinput |
| **Windows** | ✅ Available (1.x) | The original AutoHotkey build is stable; the Rust 2.0 Windows adapter is in progress |

---

## Quick start

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
clx                 # starts the daemon (forks to background)
```

Grant **Accessibility** permission when prompted (System Settings → Privacy & Security → Accessibility). Voice input needs the separate [otoji](https://github.com/snomiao/otoji/releases) app, which asks for its own Microphone permission.

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
clx
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.ps1 | iex
```

→ See [Installation](installation.md) for portable zips, Chocolatey, and source builds.

---

## Hotkey map

Hold **CapsLock** _or_ **Space** as the trigger, then press an action key. The two triggers are interchangeable. Press them simultaneously (chord) to **lock** CLX mode until you tap either again.

### Editing (right-hand cluster)

| Key | Action |
|---|---|
| `HJKL` | Cursor left / down / up / right (vim) |
| `YUIO` | Home / PageDown / PageUp / End |
| `G` | Enter |
| `T` | Delete |
| `N` / `P` | Tab / Shift+Tab |

Modifiers stack: hold `Shift` for selection, `Cmd`/`Ctrl` for word jumps, etc.

### Mouse (left-hand cluster)

| Key | Action |
|---|---|
| `WASD` | Move mouse (accelerated) |
| `QE` | Left / right click |
| `RF` | Scroll up / down |

### Window management

| Key | Action |
|---|---|
| `Z` | Cycle windows (Shift+Z reverses) |
| `X` | Close tab (Shift = window, Ctrl+Alt = kill) |
| `C` | Tile / arrange (Shift = side-by-side) |
| `1`–`9` | Switch virtual desktop |

### AI features

| Key | Action |
|---|---|
| `V` | **Voice input** — triggers the otoji app: hold to talk, tap to toggle. See below. |
| `B` | **Brainstorm chat** — floating overlay with streaming LLM reply. |

### Escape valve

| Key | Action |
|---|---|
| `Esc` (no trigger) | Dismiss any overlay, kill a running agent. |

---

## Voice input — `Space+V`

Voice input lives in [otoji](https://github.com/snomiao/otoji), not CapsLockX. `Space+V` triggers it: tap to turn otoji on or off, hold to talk. If otoji is installed but not running it is launched; if it isn't installed, CapsLockX suggests [getting it](https://github.com/snomiao/otoji/releases). All voice settings are in otoji.

otoji ships for macOS only so far; on Windows and Linux `Space+V` says it isn't available yet.

## Brainstorm chat — `Space+B`

A floating overlay panel. Streams replies from your configured LLM (Gemini, OpenAI, Anthropic, or local Ollama / MLX). Use it for quick questions without context-switching to a browser. History is preserved between sessions.

## LLM agent — `clx agent`

The `Space+M` hotkey is temporarily disabled while voice input moves to otoji. The CLI still works.

Give the agent a task in natural language; it sees your screen (Apple Vision OCR + AX tree) and operates your computer via the CLX command language:

```
k a              tap key 'a'
k c-c            Ctrl+C       (s=shift, c=ctrl, a=alt, w=cmd)
k "text"         type a string
m 400 300 c      move mouse to (400,300) and click
wf "Save" 3s     wait up to 3s for "Save" to appear in the AX tree
scan ID x y w h dark>N { k key }   60 Hz pixel reflex (e.g. Chrome Dino)
```

CLI alternatives:

```bash
clx agent --tree                  # dump the frontmost app's accessibility tree
clx agent --prompt "open notes and write today's date"
clx dino                          # auto-play Chrome Dino with the pixel-scan reflex
```

→ Full reference: [`docs/agent`](agent/) and the system prompt at [`skills/clx-agent/SKILL.md`](https://github.com/snolab/CapsLockX/blob/main/skills/clx-agent/SKILL.md).

---

## Why "CapsLockX"?

`CapsLock` is the largest key on every keyboard, in the most ergonomic position, and almost nobody uses it for its intended purpose. CapsLockX repurposes it (and `Space`) as a chord trigger, the way `Cmd`/`Ctrl` are used for system shortcuts — but for productivity actions you actually run dozens of times per minute.

The 1.x branch was an AutoHotkey script (Windows only). 2.0 is a ground-up Rust rewrite with a platform-agnostic core (`rs/core`) and per-OS adapters (`rs/adapters/{macos,linux,windows,browser}`). The Rust core also enables features the AHK version couldn't reach: voice STT, an LLM agent, and accurate physics-based mouse acceleration.

---

## Status & roadmap

CapsLockX 2.0 is in active development. The macOS adapter is the most mature; Linux works for the keyboard/mouse/window features; Windows 2.0 is in progress while 1.x continues to ship.

→ [Detailed roadmap & status table](Roadmap.md)
→ [Core trigger behaviors spec (2026-05-15)](2026-05-15-core-behaviors.html)

---

## Repository

- **GitHub**: <https://github.com/snolab/CapsLockX>
- **Releases**: <https://github.com/snolab/CapsLockX/releases>
- **License**: GPL-3.0

Other-language landing pages: [English](README.en.md) · [简体中文](README.zh.md) · [日本語](README.ja.md) · [Français](README.fr.md) · [Español](README.es.md) · [Русский](README.ru.md)
