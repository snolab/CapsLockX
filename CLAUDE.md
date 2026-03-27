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
      voice.rs             — Space+V voice input (STT via SenseVoice/Whisper)
      brainstorm.rs        — Space+B LLM chat (streams to overlay)
      agent.rs             — Space+M LLM agent (controls computer via CLX commands)
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
    src/voice_overlay.rs   — voice STT overlay
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
| V | Voice input (hold=listen, tap=toggle) | voice.rs |
| B | Brainstorm AI chat | brainstorm.rs |
| M | Agent (LLM controls computer) | agent.rs |
| Z | Window cycle (Shift+Z = reverse) | window_manager.rs |
| X | Close tab (Shift=window, Ctrl+Alt=kill) | window_manager.rs |
| C | Tile windows (Shift=side-by-side) | window_manager.rs |
| 1-9 | Switch virtual desktop | virtual_desktop.rs |
| ESC | Dismiss overlays / kill agent (bare, no trigger needed) | engine.rs |

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
The `bin/clx` wrapper script sets `DYLD_LIBRARY_PATH` for onnxruntime.

## Build (Windows)
```
cargo build -p capslockx-windows --release
```
Binary: `rs/target/release/clx-rust.exe`

## After fixing bugs / building
**Always rebuild and relaunch after code changes — without being asked.**
```bash
./build.sh && pkill -f "CapsLockX/clx" || true; clx
```

## Voice Module Rules
- **NEVER suppress or gate mic STT** — both mic and sys tracks must always run and produce output, even if system audio is playing. The user wants to see both transcriptions simultaneously.
- Echo/bleed in the mic track is acceptable; do NOT add energy gates, mic suppression, or silence-feeding to VAD when sys audio is active.

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
