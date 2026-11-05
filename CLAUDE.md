# CapsLockX – Claude Instructions

## Project Overview
CapsLockX 2.0 — keyboard productivity tool + LLM agent, rewritten in Rust.
Uses CapsLock and Space as trigger keys for hotkey combos (hold trigger + press action key).
macOS adapter uses CGEventTap for keyboard hook + CGEventPost for input injection.

## Architecture
```
rs/
  core/                    — platform-agnostic engine + modules
    src/engine.rs          — main key event router (trigger detection, bypass logic)
    src/modules/
      edit.rs              — Space+HJKL cursor, Space+YUIO page nav, Space+G enter, Space+T delete
      mouse.rs             — Space+WASD mouse movement (AccModel physics)
      media.rs             — media keys
      voice.rs             — Space+V trigger for the otoji app (tap=toggle, hold=push-to-talk)
      brainstorm.rs        — Space+B LLM chat (streams to overlay)
      agent.rs             — LLM agent module (temporarily unbound, see agent::ENABLED)
      window_manager.rs    — Space+Z window cycle, Space+X close, Space+C tile
      virtual_desktop.rs   — Space+1-9 virtual desktops
    src/platform.rs        — Platform trait (key_tap, mouse_move, etc.)
    src/llm_client.rs      — multi-provider LLM streaming (Gemini, OpenAI, Anthropic, Ollama)
  adapters/macos/
    src/main.rs            — entry point, subcommand routing (clx, clx agent, clx dino)
    src/hook.rs            — CGEventTap keyboard hook
    src/output.rs          — CGEventPost input injection + AX window management
    src/bin/clx-agent.rs   — standalone agent binary (AX tree, LLM loop, scan reflex)
    src/bin/clx-prompt.rs  — subprocess prompt dialog (no CGEventTap interference)
    src/agent_cmd.rs       — bridges `clx agent` subcommand to clx-agent.rs
    src/brainstorm_overlay.rs — floating overlay panel for brainstorm/agent output
    src/otoji.rs           — delivers CLX+V commands to otoji-tray (launches otoji.app if needed)
    src/tray.rs            — menu bar tray icon
    src/config_store.rs    — preferences persistence
    src/prefs.rs           — preferences GUI
skills/clx-agent/SKILL.md — system prompt for LLM agent (editable without rebuild)
docs/agent/               — comprehensive design docs for the agent system
```

## Hotkey Map (hold CapsLock or Space as trigger)
| Key | Action | Module |
|-----|--------|--------|
| HJKL | Cursor movement (vim-style) | edit.rs |
| YUIO | Page nav (Home/End/PgUp/PgDn) | edit.rs |
| G | Enter | edit.rs |
| T | Delete | edit.rs |
| N/P | Tab / Shift+Tab | edit.rs |
| WASD | Mouse movement | mouse.rs |
| V | Trigger otoji (hold=push-to-talk, tap=toggle); suggests installing it if missing | voice.rs |
| B | Brainstorm AI chat | brainstorm.rs |
| Z | Window cycle (Shift+Z = reverse) | window_manager.rs |
| X | Close tab (Shift=window, Ctrl+Alt=kill) | window_manager.rs |
| C | Tile windows (Shift=side-by-side) | window_manager.rs |
| 1-9 | Switch virtual desktop | virtual_desktop.rs |
| ESC | Dismiss overlays / kill agent (bare, no trigger needed) | engine.rs |
| Ctrl+Cmd+Shift+T | Toggle system Dark Mode (global, no trigger needed) | engine.rs → dark_mode.rs |
| Ctrl+Alt+Shift ×2 | Send focused window to back — escape full-screen RDP/VM (global, no trigger needed) | engine.rs → rdp_escape.rs |

**Chord layer** — hold *both* triggers (CapsLock **and** Space) and the number
row becomes function keys: `1`-`0`, `-`, `=` emit F1-F12 (`fn_row.rs`). Held
modifiers decorate them, so CapsLock+Space+Alt+`4` is a real Alt+F4. A single
trigger keeps the virtual-desktop bindings above. Tapping the chord with no
other key still toggles CLX lock, and the lock now latches when both triggers
come *up*, not on key-down.

## Build (macOS)
**Always use `./build.sh`** — it builds, copies, AND code-signs the binary.
Code signing with stable identifier `com.snomiao.capslockx` makes
Accessibility + Screen Recording permissions persist across rebuilds.
```bash
./build.sh
# Equivalent to:
# cd rs && cargo build -p capslockx-macos --release
# cp target/release/capslockx ../clx
# codesign -s - --force --identifier "com.snomiao.capslockx" ../clx
```

After building, clx auto-deduplicates (kills old instance on startup).
Launch: `clx` (forks to background) or `clx -f` (foreground).

## Build (Windows)
```
cargo build -p capslockx-windows --release
```
Binary: `rs/target/release/clx.exe`

## After fixing bugs / building
**Always rebuild and relaunch after code changes — without being asked.**
```bash
./build.sh && pkill -f "CapsLockX/clx" || true; clx
```

**Do NOT commit or push without explicit user approval.**
After building, test the feature locally, report findings, and wait.
Only commit when the user says "commit". Only push when the user says "push".
Never chain implement → build → commit → push in one shot.

## Voice lives in otoji
CapsLockX has no voice stack: no audio capture, STT, polish, TTS or wake word. All of it is in [otoji](https://github.com/snomiao/otoji) (`lib/otoji` is only a pinned submodule; nothing in `rs/` builds against it). **Do not add voice code here — send it to otoji.**

CLX+V is a thin trigger (`rs/core/src/modules/voice.rs`): tap → `toggle`, hold (≥300 ms) → `ptt-start` / `ptt-end`, delivered through `Platform::otoji_send`.
- **macOS** (`rs/adapters/macos/src/otoji.rs`): one ASCII command per datagram on otoji-tray's Unix socket `<otoji data dir>/tray-ctl.sock`. If nothing is listening and otoji.app (`com.snomiao.otoji`) is installed, launch it with `open -g -b` and retry, sending `on` instead of `toggle`. If it isn't installed, the overlay offers the download page.
- **Windows / Linux**: the default `otoji_send` reports `Unsupported`, and CLX+V says "not available yet".
- **The microphone belongs to otoji.** clx must never request Microphone permission or open an input device.
- The voice fields of old `config.json` files are kept on save (`FullConfig::extra`) because otoji imports them on its first run.

## Agent is temporarily disabled
`agent::ENABLED = false` unbinds CLX+M: agent mode was fed by CLX's own transcripts, which no longer exist. The module still compiles and is tested, and `clx agent` still works. Flip the constant once otoji can hand transcripts back.

## Privacy: never use user's real prompts as examples
When writing few-shot examples or documentation for LLM prompts (polish, translate, agent system prompts, `lib/otoji/src/polish.rs`, `skills/clx-agent/SKILL.md`, etc.), **NEVER copy phrases from the user's voice logs, chat messages, transcripts, or `tmp/clx-watchdog.log`**. Always synthesize generic, unrelated examples (e.g. `kafka`, `npm run`, `下雨了`, `water bottle`).

Why: the user speaks personal, work-in-progress content through the mic. Anything embedded in a committed system prompt (a) ships to the cloud LLM on every subsequent call, and (b) lives in git history forever. This rule is a hard requirement — violating it is a privacy regression even when the example "seems harmless."

Rule of thumb: if you can find the candidate example string by grepping `tmp/` or scrolling the conversation, it is forbidden. Invent a new one.

## Key Design Decisions

### macOS Permission Hazard
**NEVER call AX or CG APIs directly** without confirming permission first.
- `AXUIElementCopyAttributeValue` → hangs in kernel (UE) without Accessibility permission
- `CGWindowListCreateImage` → hangs without Screen Recording permission
- UE processes cannot be killed (even kill -9), only cleared by reboot
- **Solution**: Use osascript for AX tree, fork() child for screen capture test

### Agent Architecture
- `clx agent --tree` — dumps AX tree via osascript (safe, no UE risk)
- `clx agent --prompt "task"` — full LLM loop (AX tree + screenshot → Gemini → CLX commands)
- `clx agent --exec` — executes CLX commands from stdin
- `clx dino` — auto-plays Chrome Dino game with 60fps pixel-scan reflex
- Agent logs to `./tmp/agent.log` and `./tmp/agent-live.log` (overlay watches live log)
- All temp files in `./tmp/` (never /tmp/ or ~/Library/)

### CLX Agent Language
```
k a          tap key 'a'
k c-c        Ctrl+C (c=ctrl, s=shift, a=alt, w=cmd)
k w-p        Cmd+P (macOS)
k "text"     type string (\n \t \" \\ escapes)
m 400 300    move mouse absolute
m 400 300 c  move + click
w 200ms      wait
wf "text" 3s wait for text in AX tree (polls every 200ms)
scan ID x y w h dark>N { k key }   pixel-scan reflex at 60fps
scan_stop ID / scan_stop all
S screen region x y w h   set capture region
? screen                  one-shot screenshot
```

### Bypass Logic (engine.rs)
These modifier+Space combos bypass CLX mode (pass through to OS):
- Shift+Space → IME switching
- Ctrl+Space → IME switching (like AHK)
- Cmd+Space → Spotlight
```

## No Claude Artifacts
Never publish lab pages, reports, or any HTML from this repo as a Claude Artifact (the `Artifact` tool), and never suggest it. Lab pages are viewed from the local server (`bun lab/serve.ts` → http://localhost:4550/lab/, or via portless at https://capslockx.localhost/lab/) or from the published site (capslockx.com/lab, deployed from `main`). Content lives in git only.
