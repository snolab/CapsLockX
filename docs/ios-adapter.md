# iOS Adapter — Feasibility Notes

> Status: **not started** — recorded for future reference.
>
> **TL;DR**: iOS is significantly more constrained than Android. There is no
> system-wide keyboard hook available to App Store apps. The viable path is a
> **Custom Keyboard Extension** covering text-field cursor control, or a
> **within-app** `UIKeyCommand` integration. Full feature parity requires
> jailbreak.

## What's achievable

| Feature | Custom Keyboard Extension | Within-app (UIKeyCommand) | Jailbreak (IOHIDEvent) |
|---|---|---|---|
| Intercept physical keyboard globally | ✗ | ✗ | ✓ |
| Intercept keys in text fields | ✓ | Own app only | ✓ |
| Cursor movement (HJKL) | ✓ `UITextDocumentProxy` | ✓ `UIKeyCommand` | ✓ |
| Scroll (RF) | ✗ | Limited (`UIScrollView`) | ✓ |
| Media keys (F5–F11) | ✗ | `MPRemoteCommandCenter` (partial) | ✓ |
| Mouse / pointer movement (WASD) | ✗ | ✗ | ✓ |
| App Store compliant | ✓ | ✓ | ✗ |

### Why iOS is more restricted than Android

Android's `AccessibilityService` can observe **all** key events from a physical
keyboard, globally, without root. iOS has no equivalent API. Apple's sandbox
allows an app to see keyboard input only:

- via a **Custom Keyboard Extension** (replaces the on-screen keyboard; the OS
  routes text input through it)
- via `UIKeyCommand` / `pressesBegan(_:with:)` inside the **foreground app** only

There is no `WH_KEYBOARD_LL` equivalent, and no `flagRequestFilterKeyEvents`
equivalent, in any public iOS API.

## Architecture options

### Option A — Custom Keyboard Extension (recommended for App Store)

```
iOS Keyboard Extension (Swift)
├── UIInputViewController  ← the hook + output point
│    ├── pressesBegan / pressesEnded  ← physical key events (unreliable)
│    └── UITextDocumentProxy          ← cursor/text output
├── FFI bridge (Swift → Rust via UniFFI)
└── IosPlatform (Rust struct impl Platform)
     ├── key_down/up   → proxy.insertText / adjustTextPosition
     ├── scroll_v/h    → ✗ no access to host app scroll views
     ├── mouse_move    → ✗ not applicable
     └── media keys    → ✗ extension sandbox blocks audio session
```

**Limitations of this option:**
- Only active when the user has switched to the CLX keyboard
- External physical keyboard input routing through the extension is unreliable
  (Apple does not guarantee `pressesBegan` fires in extensions for all keys)
- No access to the host app's views — scroll, window management all no-op
- Extension memory limit: **48 MB** (Apple enforces this; the ticker threads and
  Rust runtime must fit comfortably inside)

### Option B — Within a single host app (UIKeyCommand)

```
Host app (Swift / SwiftUI)
├── UIKeyCommand registrations  ← CapsLock+H, CapsLock+J, …
├── pressesBegan / pressesEnded in UIResponder chain
├── FFI bridge → capslockx-core (Rust)
└── IosPlatform
     ├── cursor keys → UITextView / UITextField cursor movement APIs
     ├── scroll      → UIScrollView.setContentOffset
     └── media keys  → MPRemoteCommandCenter
```

This is the right path if you are building a CapsLockX-enhanced note-taking /
editor app, not a system-wide utility.

**Limitation:** `UIKeyCommand` cannot intercept `CapsLock` — iOS consumes it.
Use a configurable trigger key (e.g. right-Option, or a function key).

### Option C — Jailbreak (IOHIDEvent / Substrate)

Full feature parity with the Windows adapter. Uses `IOHIDEventSystemClient` to
intercept raw HID events before the OS dispatches them. Not App Store deployable.
Target audience: power users on jailbroken iPhones, similar to AHK on Windows.

Not detailed further here; design would mirror the Windows
`WH_KEYBOARD_LL` hook architecture.

## Rust crate: `rs/adapters/ios/` (`staticlib`)

iOS does not allow dynamic libraries (`.dylib`) inside App Store extensions.
The Rust code is compiled as a **static library** and linked into the Swift target.

```
rs/adapters/ios/
├── Cargo.toml
└── src/
    ├── lib.rs          — crate root, re-exports FFI symbols
    ├── platform.rs     — IosPlatform implementing Platform trait
    ├── key_map.rs      — UIKey.keyCode / UIKeyboardHIDUsage → KeyCode
    └── ffi.rs          — extern "C" functions called from Swift
```

`Cargo.toml` sketch:
```toml
[package]
name    = "capslockx-ios"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
capslockx-core = { path = "../../core" }
once_cell      = "1.19"
```

No `jni` crate needed (that is Android-specific). Swift calls Rust via a plain
C header (`capslockx_ios.h`) generated from the `extern "C"` functions, or via
**UniFFI** for a more ergonomic Swift API.

### UniFFI vs hand-written FFI

| | UniFFI | Hand-written `extern "C"` |
|---|---|---|
| Boilerplate | Low — generate Swift bindings from a `.udl` file | High — manual `UnsafePointer` in Swift |
| Type safety | Strong — enums, structs mapped automatically | Weak — integers across the boundary |
| Precedent | Firefox iOS, Mozilla VPN | Legacy C libraries |
| Extra dep | `uniffi`, `uniffi-bindgen` | None |

UniFFI is recommended. The `.udl` interface would be essentially the same as the
Android adapter's JNI surface: `key_event(code: i32, pressed: bool) -> i32`.

## Key-code mapping note

iOS 13.4+ exposes `UIKey.keyCode` as a `UIKeyboardHIDUsage` enum (USB HID page
0x07 usage codes — the same standard that Windows virtual keys map from).
Mapping is straightforward:

```swift
// Swift side — convert UIKey to integer for Rust
let hidCode = key.keyCode.rawValue   // e.g. 0x04 = KeyboardA
```

```rust
// Rust side — key_map.rs
pub fn hid_usage_to_keycode(usage: u32) -> KeyCode { … }
```

HID usage codes are a universal standard so the table is well-documented
and stable across platforms.

## Build toolchain

```bash
# Add iOS targets
rustup target add aarch64-apple-ios          # physical device (ARM64)
rustup target add aarch64-apple-ios-sim      # simulator on Apple Silicon Mac
rustup target add x86_64-apple-ios           # simulator on Intel Mac

# Build static library for device
cargo build -p capslockx-ios \
    --target aarch64-apple-ios \
    --release
# output: rs/target/aarch64-apple-ios/release/libcapslockx_ios.a

# Build XCFramework (bundles device + simulator slices for Xcode distribution)
xcodebuild -create-xcframework \
    -library rs/target/aarch64-apple-ios/release/libcapslockx_ios.a \
    -library rs/target/aarch64-apple-ios-sim/release/libcapslockx_ios.a \
    -output CapsLockX.xcframework
```

`cargo-lipo` can also produce a fat `.a` for older Xcode setups, but XCFramework
is the modern approach.

## Threading note

`std::thread` and `std::sync::Condvar` work on iOS (backed by pthreads).
`AccModel2D`'s background ticker thread runs unchanged. Memory budget for the
Custom Keyboard Extension is 48 MB total — the Rust runtime + threads fit
comfortably inside that limit (the Windows binary is ~2–4 MB; iOS will be similar).

## CapsLock caveat

`CapsLock` (`UIKeyboardHIDUsage.keyboardCapsLock`) is consumed by iOS itself and
does not reach the app or extension. The trigger key must be user-configurable.
Good candidates for a physical keyboard on iPadOS:

- Right-Option (`UIKeyboardHIDUsage.keyboardRightAlt`)
- Globe/Language key (`UIKeyboardHIDUsage.keyboardLang1`)
- A spare function key (F13–F19 on full-size keyboards)

## Comparison: iOS vs Android adapter scope

| Aspect | Android adapter | iOS adapter |
|---|---|---|
| Global keyboard hook | ✓ Accessibility Service | ✗ not possible without JB |
| Rust FFI mechanism | JNI (`jni` crate) | `extern "C"` / UniFFI |
| Library type | `cdylib` (`.so`) | `staticlib` (`.a`) |
| Ticker thread | ✓ works on NDK | ✓ works on iOS (pthreads) |
| Key output | `AccessibilityNodeInfo` actions | `UITextDocumentProxy` |
| Scroll output | `ACTION_SCROLL_*` | No equivalent in extension |
| Deployment | Play Store + sideload | App Store (limited) + JB |
| Feature parity with Windows | ~60–70% (no root) | ~30–40% (App Store only) |

## What does NOT need to change

- `capslockx-core` — zero changes; `Platform` trait and `ClxEngine` are
  platform-agnostic.
- `AccModel2D` — ticker thread runs unchanged on iOS.
- All physics, module logic, and key suppression logic.

## Effort estimate

| Component | Lines |
|---|---|
| `rs/adapters/ios/` Rust crate | ~300–500 |
| UniFFI `.udl` definition | ~30 |
| Swift `AccessibilityInputViewController` + FFI glue | ~300–400 |
| Xcode project + entitlements + build scripts | ~100 |
| **Total** | ~1.5–2 weeks |

Larger than the Android adapter primarily because of Xcode project setup,
provisioning profile requirements, and the more limited output API surface
(which requires more creative workarounds for features like scroll).
