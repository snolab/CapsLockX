# CapsLockX – Rust core

A Rust reimplementation of the CapsLockX keyboard enhancer, split into a
**platform-agnostic core library** and **per-platform adapters**.

## Project structure

```
rs/
  core/                      ← capslockx-core  (pure Rust, no OS APIs)
    src/
      key_code.rs            KeyCode enum + Modifiers
      platform.rs            Platform trait (what adapters implement)
      state.rs               CLX mode state machine
      acc_model.rs           AccModel2D physics (std::time::Instant)
      engine.rs              ClxEngine – processes key events, calls Platform
      modules/
        edit.rs              HJKL cursor / YUIO page / PN tab
        mouse.rs             WASD mouse / QE buttons / RF scroll
        media.rs             F5-F11 media keys
        window_manager.rs    Z/X/C/V window management (dispatches to Platform)
  adapters/
    windows/                 ← capslockx-windows  (Win32)
      src/
        main.rs              Win32 message loop entry point
        hook.rs              WH_KEYBOARD_LL hook → ClxEngine
        output.rs            WinPlatform impl (SendInput + Win32 window APIs)
        vk.rs                Windows VK ↔ KeyCode mapping
```

## Building

```sh
# Windows adapter (produces capslockx.exe)
cargo build -p capslockx-windows
cargo build -p capslockx-windows --release
```

## How adapters work

Each platform adapter is responsible for **three things**:

### 1. Hooking into keyboard events

| Platform | Mechanism |
|----------|-----------|
| **Windows** | `WH_KEYBOARD_LL` (low-level hook) or `RegisterHotKey` |
| **macOS** | `CGEventTap` |
| **Linux** | `evdev` / `XGrabKey` / `uinput` |
| **Browser** | `window.addEventListener('keydown', handler, {capture:true})` compiled to WASM |
| **Android** | `AccessibilityService` / `InputMethodService` |

### 2. Converting native keys and calling the engine

```rust
// pseudo-code (same pattern for every adapter)
let code = platform_keycode_to_KeyCode(native_event.key);
match engine.on_key_event(code, pressed) {
    CoreResponse::Suppress    => prevent_default(native_event),
    CoreResponse::PassThrough => pass_through(native_event),
}
```

### 3. Implementing the `Platform` trait

```rust
impl Platform for MyPlatform {
    fn key_down(&self, key: KeyCode) { /* inject synthetic key press */ }
    fn key_up(&self, key: KeyCode)   { /* inject synthetic key release */ }
    fn mouse_move(&self, dx: i32, dy: i32) { /* move cursor */ }
    // optional window management: cycle_windows, arrange_windows, …
}
```

Window management methods have default no-op implementations, so adapters
that don't support them (e.g. browser) compile without extra code.

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
| W / A / S / D | Mouse ↑ ← ↓ → (with acceleration) |
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
| C / Shift+C | Arrange cascaded / side-by-side |
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

Single-tap restores native key function (toggle caps / type space).
CapsLock+Space together locks CLX mode until any trigger tap.
