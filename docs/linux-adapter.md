# Linux Adapter — Feasibility Notes

> Status: **not started** — recorded for future reference.
>
> **TL;DR**: Linux is the most capable non-Windows target. `evdev` + `uinput`
> gives full feature parity with the Windows adapter — including global keyboard
> hook, mouse movement, scroll, and media keys — with no root required (only a
> `udev` group rule). Works on both X11 and Wayland.

## What's achievable

| Feature | evdev + uinput (no root) | Root |
|---|---|---|
| Intercept physical keyboard (global) | ✓ `EVIOCGRAB` exclusive grab | ✓ |
| Cursor movement (HJKL → arrow keys) | ✓ uinput `EV_KEY` | ✓ |
| Mouse movement (WASD) | ✓ uinput `EV_REL` (REL_X / REL_Y) | ✓ |
| Scroll (RF) | ✓ uinput `EV_REL` (REL_WHEEL) | ✓ |
| Media keys (F5–F11) | ✓ uinput `KEY_PLAYPAUSE` etc. | ✓ |
| Mouse buttons (QE) | ✓ uinput `BTN_LEFT / BTN_RIGHT` | ✓ |
| X11 support | ✓ (evdev is display-server agnostic) | ✓ |
| Wayland support | ✓ (evdev is display-server agnostic) | ✓ |

**Feature parity with Windows: ~95–100%.** This is the closest of all
non-Windows targets to the original AHK behaviour.

## How it works

### Input — `evdev` exclusive grab

```
/dev/input/eventX  (physical keyboard device)
    │
    │  open() + ioctl(EVIOCGRAB)  → exclusive access; kernel stops
    │                                forwarding events to the OS
    ▼
Rust hook thread reads InputEvent { type: EV_KEY, code: KEY_CAPSLOCK, value: 1 }
    │
    ├─ map code → KeyCode
    ├─ engine.on_key_event(code, pressed)
    │
    ├─ CoreResponse::Suppress    → discard (already grabbed exclusively)
    └─ CoreResponse::PassThrough → write event to uinput virtual device
```

`EVIOCGRAB` takes exclusive ownership of the physical device. The kernel
suppresses the event from all other consumers (X11, Wayland compositor, etc.).
The Rust process decides per-event whether to forward, modify, or swallow it.
This is the same principle as `WH_KEYBOARD_LL` on Windows.

### Output — `uinput` virtual device

A virtual keyboard + mouse device is created via `/dev/uinput`:

```
LinuxPlatform::key_down(KeyCode::ArrowLeft)
    └─ write InputEvent { type: EV_KEY, code: KEY_LEFT, value: 1 }
       write InputEvent { type: EV_SYN, code: SYN_REPORT, value: 0 }
       → to the uinput fd
```

The OS presents this virtual device to all applications exactly like a real
keyboard/mouse. No self-inject detection needed: events from the uinput fd
come from a different device node than the grabbed physical keyboard, so
the hook thread never reads them back.

### Self-inject detection

Simpler than Windows — not needed at all. The hook thread reads from
`/dev/input/eventX` (the real keyboard). Synthesised events are written to
a separate `/dev/uinput` virtual device. The two file descriptors are
completely independent; there is no feedback loop.

## Architecture

```
capslockx (Rust binary)
├── main.rs      — open devices, run event loop
├── hook.rs      — evdev reader thread (one per keyboard device)
│    └── reads InputEvent, calls engine.on_key_event()
├── output.rs    — LinuxPlatform implementing Platform trait
│    ├── key_down/up   → EV_KEY to uinput
│    ├── mouse_move    → EV_REL REL_X/REL_Y to uinput
│    ├── scroll_v/h    → EV_REL REL_WHEEL/REL_HWHEEL to uinput
│    └── mouse_button  → BTN_LEFT/BTN_RIGHT to uinput
└── key_map.rs   — Linux evdev keycode → KeyCode
```

The event loop is a simple `poll()` / `epoll()` over the evdev fds — no
platform message loop needed (unlike Win32's `GetMessageW` requirement).

## Rust crate: `rs/adapters/linux/`

```toml
# Cargo.toml
[package]
name    = "capslockx-linux"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "capslockx"
path = "src/main.rs"

[dependencies]
capslockx-core = { path = "../../core" }
evdev          = "0.12"   # safe Rust evdev + uinput API
once_cell      = "1.19"
```

The [`evdev`](https://crates.io/crates/evdev) crate (by the `input-linux`
authors) wraps the kernel evdev interface cleanly:

```rust
// hook.rs sketch
use evdev::{Device, EventType, Key};

let mut dev = Device::open("/dev/input/event3")?;
dev.grab()?;   // EVIOCGRAB

for event in dev.fetch_events()? {
    if event.event_type() != EventType::KEY { continue; }
    let key  = Key::new(event.code());
    let code = evdev_key_to_keycode(key);
    match ENGINE.on_key_event(code, event.value() != 0) {
        CoreResponse::Suppress    => {}            // already grabbed, just drop
        CoreResponse::PassThrough => uinput.emit(&[event])?,
    }
}
```

```rust
// output.rs sketch
use evdev::uinput::VirtualDeviceBuilder;

let uinput = VirtualDeviceBuilder::new()?
    .name("CapsLockX Virtual Keyboard")
    .with_keys(&AttributeSet::from_iter([KEY_LEFT, KEY_RIGHT, …]))?
    .with_relative_axes(&AttributeSet::from_iter([REL_X, REL_Y, REL_WHEEL]))?
    .build()?;
```

## Permissions (no root required)

By default, `/dev/input/event*` and `/dev/uinput` require root. A `udev` rule
grants access to any user in the `input` group:

```udev
# /etc/udev/rules.d/99-capslockx.rules
KERNEL=="uinput",   SUBSYSTEM=="misc",    GROUP="input", MODE="0660"
KERNEL=="event[0-9]*", SUBSYSTEM=="input", GROUP="input", MODE="0660"
```

```bash
sudo usermod -aG input $USER   # add user to input group
# re-login (or: newgrp input)
```

Alternatively, grant via `setcap`:
```bash
sudo setcap cap_dac_read_search,cap_dac_override+eip /usr/bin/capslockx
```

## Key-code mapping

Linux evdev uses USB HID-derived keycodes defined in
`<linux/input-event-codes.h>` (e.g. `KEY_CAPSLOCK = 58`, `KEY_H = 35`).
The mapping table in `key_map.rs` is a `match evdev_code { 58 => KeyCode::CapsLock, … }`.

CapsLock is available as a normal key (`KEY_CAPSLOCK`) — no OS interception
issue. With `EVIOCGRAB`, the kernel's caps-lock toggle never fires.

## Multi-keyboard support

The hook thread should watch **all** keyboards, not just a hardcoded device path.
At startup, scan `/dev/input/` for devices that have `EV_KEY` + `KEY_CAPSLOCK`
capability (via `evdev::Device::supported_keys()`). Spawn one hook thread per
device. Use `inotify` on `/dev/input/` to detect hotplug.

## Wayland note

`evdev` reads from the kernel input subsystem, completely bypassing the display
server. The hook works identically on X11, Wayland (any compositor), and even
on a raw Linux TTY. No X11 or Wayland libraries are needed.

## Build toolchain

```bash
# Native build (most common)
cargo build -p capslockx-linux --release
# output: rs/target/release/capslockx

# Cross-compile for ARM (e.g. Raspberry Pi)
rustup target add aarch64-unknown-linux-gnu
cargo build -p capslockx-linux --release --target aarch64-unknown-linux-gnu
```

## CI integration

Add `ubuntu-latest` job to `ci-rust.yml` alongside the existing `windows-latest`:

```yaml
build-linux:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
      with:
        workspaces: rs -> rs/target
    - run: cargo build -p capslockx-linux --release
      working-directory: rs
```

No extra dependencies needed on `ubuntu-latest` (kernel headers for evdev are
pre-installed).

## What does NOT need to change

- `capslockx-core` — zero changes.
- `AccModel2D` — ticker thread runs unchanged on Linux.
- All physics, module logic, and key suppression logic.

## Effort estimate

| Component | Lines |
|---|---|
| `rs/adapters/linux/` Rust crate | ~400–500 |
| udev rules + install script | ~20 |
| **Total** | ~1 week |

Linux is the **easiest** non-Windows port — `evdev` is well-documented, the
Rust crate is mature, no special FFI bridge or permissions framework is needed,
and the architecture mirrors the Windows adapter almost 1:1.
