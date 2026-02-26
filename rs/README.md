# CapsLockX – Rust Core

A Rust reimplementation of the CapsLockX keyboard enhancer core.

## Building

```
cargo build          # debug
cargo build --release
```

The binary is `target/release/capslockx.exe`.

## How it works

A `WH_KEYBOARD_LL` hook intercepts all keystrokes system-wide. Holding a **trigger key** (CapsLock or Space by default) activates **FN mode**; pressing CapsLock+Space together locks **CLX mode**. In either mode, mapped keys are remapped to the actions below.

## Hotkeys

### CLX-Edit (HJKL)

| CLX + | Action |
|-------|--------|
| H / J / K / L | ← ↓ ↑ → (with acceleration) |
| Y / O | Home / End |
| U / I | Page Up / Page Down |
| G | Enter |
| T | Delete |
| N / P | Tab / Shift+Tab |

### CLX-Mouse (WASD)

| CLX + | Action |
|-------|--------|
| W / A / S / D | Mouse move ↑ ← ↓ → (with acceleration) |
| E / Q | Left / Right mouse button |
| R / F | Scroll up / down |

### CLX-MediaKeys

| CLX + | Action |
|-------|--------|
| F5 | Play / Pause |
| F6 / F7 | Prev / Next track |
| F8 | Stop |
| F9 / F10 | Volume + / − |
| F11 | Mute |

### CLX-WindowManager

| CLX + | Action |
|-------|--------|
| Z / Shift+Z | Cycle windows forward / backward |
| X | Close tab (Ctrl+W) |
| Shift+X | Close window & cycle |
| Ctrl+Alt+X | Kill process & cycle |
| C / Shift+C | Arrange cascaded / grid |
| V (hold) | Transparent + always-on-top |
| V (release) | Restore |
| Shift+V | Toggle always-on-top |

## Trigger keys

| Key | Default |
|-----|---------|
| CapsLock | ✓ on |
| Space | ✓ on |
| Insert | off |
| ScrollLock | off |
| Right Alt | off |

Single-tap a trigger with no other key → restores its native function (CapsLock toggles caps, Space types a space).

## Module structure

```
src/
  main.rs          – entry point, Win32 message loop
  hook.rs          – WH_KEYBOARD_LL callback & CLX state machine
  state.rs         – global atomic mode state
  vk.rs            – virtual key code constants
  input.rs         – SendInput wrappers (keyboard + mouse)
  acc_model.rs     – AccModel2D physics (exp+poly acceleration)
  modules/
    edit.rs        – cursor / page / tab navigation
    mouse.rs       – mouse movement & buttons
    media.rs       – media key shortcuts
    window_manager.rs – window cycling, tiling, close/kill, transparency
```
