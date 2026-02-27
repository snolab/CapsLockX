# macOS Adapter — Feasibility Notes

> Status: **not started** — recorded for future reference.
>
> **TL;DR**: macOS is achievable with ~90–95% feature parity using
> `CGEventTap` (no root, Accessibility permission required from the user).
> The architecture mirrors the Windows adapter closely. Main constraints are
> the Accessibility permission prompt and CapsLock LED management.

## What's achievable

| Feature | CGEventTap (Accessibility perm) | Root / SIP disabled |
|---|---|---|
| Intercept physical keyboard (global) | ✓ `kCGSessionEventTap` | ✓ `kCGHIDEventTap` |
| Cursor movement (HJKL → arrow keys) | ✓ `CGEventCreateKeyboardEvent` | ✓ |
| Mouse movement (WASD) | ✓ `CGWarpMouseCursorPosition` | ✓ |
| Scroll (RF) | ✓ `CGEventCreateScrollWheelEvent` | ✓ |
| Media keys (F5–F11) | ✓ via `NSEvent` / `CGEventTap` | ✓ |
| Mouse buttons (QE) | ✓ `CGEventCreateMouseEvent` | ✓ |
| CapsLock interception | Mostly ✓ (LED may still toggle) | ✓ full control |
| App Store distribution | ✗ (Accessibility API blocked) | ✗ |
| Notarized direct download | ✓ | ✗ |

**Feature parity with Windows: ~90–95%.** Distributed as a notarized app
bundle or Homebrew formula, not via the App Store.

## How it works

### Input — CGEventTap

`CGEventTap` is macOS's equivalent of `WH_KEYBOARD_LL`. It inserts a callback
into the event stream at the session level (all applications):

```
Physical keyboard
    │
    ▼
kCGSessionEventTap  ← Rust callback fires here
    │
    ├─ CoreResponse::Suppress    → CGEventTapEnable(tap, false) momentarily?
    │                              No — return NULL from callback to suppress
    └─ CoreResponse::PassThrough → return event unchanged (or modified)
```

Returning `NULL` from the callback suppresses the event. Returning the
(possibly modified) `CGEvent` passes it through.

### Output — CGEvent injection

```
LinuxPlatform::key_down(KeyCode::ArrowLeft)
    └─ CGEventCreateKeyboardEvent(NULL, kVK_LeftArrow, true)
       CGEventPost(kCGSessionEventTap, event)
```

Self-inject detection: macOS CGEvents have a `pid` field. Set a custom
event field (via `CGEventSetIntegerValueField`) on injected events — analogous
to Windows `CLX_EXTRA_INFO` — and check it in the tap callback.

Alternatively, temporarily disable the tap (`CGEventTapEnable(tap, false)`)
while injecting, then re-enable. Simpler but has a tiny race window.

### Accessibility permission

`CGEventTap` at `kCGSessionEventTap` requires the app to be listed in:
**System Settings → Privacy & Security → Accessibility**.

The first time the app runs, macOS will show a prompt. The permission persists
across reboots. No root needed, no SIP changes needed.

```rust
// Check at startup; exit with clear message if not granted
let trusted = unsafe { AXIsProcessTrustedWithOptions(options) };
if !trusted {
    eprintln!("[CLX] Accessibility permission required.");
    eprintln!("      Open System Settings → Privacy → Accessibility and add this app.");
    std::process::exit(1);
}
```

## Architecture

```
capslockx (Rust binary / .app bundle)
├── main.rs      — request Accessibility, install tap, run CFRunLoop
├── hook.rs      — CGEventTap callback → engine.on_key_event()
├── output.rs    — MacPlatform implementing Platform trait
│    ├── key_down/up    → CGEventCreateKeyboardEvent + CGEventPost
│    ├── mouse_move     → CGWarpMouseCursorPosition
│    ├── scroll_v/h     → CGEventCreateScrollWheelEvent
│    └── mouse_button   → CGEventCreateMouseEvent (kCGEventLeftMouseDown…)
└── key_map.rs   — macOS virtual key (kVK_*) → KeyCode
```

The event loop is a `CFRunLoop` (macOS's native run loop), which is required for
`CGEventTap` to deliver callbacks — analogous to Win32's `GetMessageW` loop.

## Rust crate: `rs/adapters/macos/`

```toml
# Cargo.toml
[package]
name    = "capslockx-macos"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "capslockx"
path = "src/main.rs"

[dependencies]
capslockx-core  = { path = "../../core" }
once_cell       = "1.19"
core-graphics   = "0.23"   # CGEvent, CGEventTap, CGEventPost
core-foundation = "0.9"    # CFRunLoop, CFMachPort
```

### hook.rs sketch

```rust
use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventType};
use core_foundation::runloop::{CFRunLoop, kCFRunLoopDefaultMode};

pub fn install_hook() {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,   // kCGSessionEventTap
        CGEventTapPlacement::HeadInsert,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown, CGEventType::KeyUp,
             CGEventType::FlagsChanged],       // CapsLock comes as FlagsChanged
        |_proxy, event_type, event| {
            let code = cgevent_to_keycode(event);
            let pressed = event_type == CGEventType::KeyDown;
            match ENGINE.on_key_event(code, pressed) {
                CoreResponse::Suppress    => None,          // drop event
                CoreResponse::PassThrough => Some(event),   // forward unchanged
            }
        },
    ).expect("CGEventTap::new failed — Accessibility permission not granted?");

    let source = tap.mach_port.create_runloop_source(0).unwrap();
    CFRunLoop::get_current().add_source(&source, unsafe { kCFRunLoopDefaultMode });
    tap.enable();
    CFRunLoop::run_current();   // blocks; tap delivers callbacks on this thread
}
```

### CapsLock handling

CapsLock on macOS arrives as `CGEventType::FlagsChanged`, not `KeyDown`/`KeyUp`.
The key-down vs key-up must be inferred from the `CGEventFlags`:

```rust
CGEventType::FlagsChanged => {
    let flags = event.get_flags();
    let now_set = flags.contains(CGEventFlags::NX_ALPHASHIFTMASK);
    ENGINE.on_key_event(KeyCode::CapsLock, now_set)
}
```

The OS still toggles the CapsLock LED when using `kCGSessionEventTap`. To
prevent this, the tap would need `kCGHIDEventTap` (requires root) or a virtual
HID device driver (Karabiner's approach — complex, requires a kernel extension
or DriverKit driver for macOS 12+). Acceptable trade-off: LED toggles, but
CapsLockX intercepts the key.

## Key-code mapping

macOS uses `CGKeyCode` / `kVK_*` virtual key constants (defined in
`<Carbon/Carbon.h>`). The mapping table in `key_map.rs`:

```rust
pub fn cgevent_keycode_to_clx(vk: CGKeyCode) -> KeyCode {
    match vk {
        0x39 => KeyCode::CapsLock,   // kVK_CapsLock
        0x04 => KeyCode::H,          // kVK_ANSI_H
        0x26 => KeyCode::J,          // kVK_ANSI_J
        // …
    }
}
```

Media keys (Play, Next, etc.) arrive via `NX_SYSDEFINED` events in the
`CGEventTap`. They need to be handled separately from regular key events.

## Build toolchain

```bash
# Apple Silicon (M-series Macs)
rustup target add aarch64-apple-darwin
cargo build -p capslockx-macos --release --target aarch64-apple-darwin

# Intel Macs
rustup target add x86_64-apple-darwin
cargo build -p capslockx-macos --release --target x86_64-apple-darwin

# Universal binary (runs natively on both)
cargo build -p capslockx-macos --release --target aarch64-apple-darwin
cargo build -p capslockx-macos --release --target x86_64-apple-darwin
lipo -create \
    target/aarch64-apple-darwin/release/capslockx \
    target/x86_64-apple-darwin/release/capslockx \
    -output capslockx-macos-universal
```

For distribution, the binary must be **code-signed** and **notarized** by
Apple. Self-signed works for local use but Gatekeeper will block it on other
machines without `xattr -cr capslockx`.

## Distribution options

| Method | Signing required | Notes |
|---|---|---|
| Homebrew formula | ✓ notarized | Best UX for developers |
| `.app` bundle download | ✓ notarized | Drag-to-Applications |
| Homebrew Cask | ✓ notarized | `brew install --cask capslockx` |
| App Store | ✗ blocked | `CGEventTap` not allowed in sandbox |

Notarization requires an Apple Developer account ($99/year).
`cargo-bundle` or a simple shell script can produce the `.app` bundle.

## CI integration

macOS builds require a `macos-latest` runner. Add to `ci-rust.yml`:

```yaml
build-macos:
  runs-on: macos-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: aarch64-apple-darwin,x86_64-apple-darwin
    - uses: Swatinem/rust-cache@v2
      with:
        workspaces: rs -> rs/target
    - name: cargo build (Apple Silicon)
      working-directory: rs
      run: cargo build -p capslockx-macos --release --target aarch64-apple-darwin
    - name: cargo build (Intel)
      working-directory: rs
      run: cargo build -p capslockx-macos --release --target x86_64-apple-darwin
```

`macos-latest` is now `macos-14` (Apple Silicon runner) on GitHub Actions.

## Comparison with Windows adapter

| Aspect | Windows | macOS |
|---|---|---|
| Hook mechanism | `SetWindowsHookExW(WH_KEYBOARD_LL)` | `CGEventTap(kCGSessionEventTap)` |
| Hook callback return | `LRESULT(1)` to suppress | `NULL` to suppress |
| Self-inject tag | `dwExtraInfo = CLX_EXTRA_INFO` | Custom `CGEvent` integer field |
| Event loop | `GetMessageW` (Win32 message loop) | `CFRunLoop::run_current()` |
| Output | `SendInput()` | `CGEventPost(kCGSessionEventTap, …)` |
| Mouse move | `SendInput` with `MOUSEEVENTF_MOVE` | `CGWarpMouseCursorPosition` |
| Scroll | `SendInput` with `MOUSEEVENTF_WHEEL` | `CGEventCreateScrollWheelEvent` |
| Permission | None (runs as any user) | Accessibility permission (one-time prompt) |
| CapsLock | Full control | LED may toggle; functional intercept works |

## What does NOT need to change

- `capslockx-core` — zero changes.
- `AccModel2D` — ticker thread runs unchanged on macOS (pthreads).
- All physics, module logic, and key suppression logic.

## Effort estimate

| Component | Lines |
|---|---|
| `rs/adapters/macos/` Rust crate | ~450–600 |
| Code-signing / notarization scripts | ~50 |
| Homebrew formula | ~30 |
| **Total** | ~1–1.5 weeks |

Slightly more effort than Linux due to notarization requirements and the
`FlagsChanged` CapsLock quirk, but architecturally the closest non-Windows
target to the existing Windows adapter.
