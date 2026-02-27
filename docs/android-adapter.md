# Android Adapter — Feasibility Notes

> Status: **not started** — recorded for future reference.

## What's achievable

| Feature | Without root | With root |
|---|---|---|
| Intercept physical keyboard keys (CapsLock, Space…) | ✓ Accessibility Service | ✓ |
| Cursor movement (HJKL → arrow keys) | Partial — IME active only | ✓ fully |
| Scroll (RF) | ✓ `AccessibilityNodeInfo` scroll actions | ✓ |
| Media keys (F5–F11) | ✓ `AudioManager` / `MediaSession` | ✓ |
| Mouse cursor movement (WASD) | Touch gesture simulation only (`dispatchGesture`) | ✓ real pointer |
| Arbitrary key injection | ✗ blocked by Android security model | ✓ via `InputManager` |

The practical no-root target: **physical Bluetooth/USB keyboard + Accessibility Service**.
That covers HJKL editing, scrolling, and media keys on any Android device.

## Architecture

```
Android app (Kotlin)
├── AccessibilityService
│    ├── onKeyEvent(KeyEvent) → JNI → engine.on_key_event()
│    └── returns CoreResponse::Suppress / PassThrough
├── JNI bridge  ←→  capslockx-core (Rust, compiled via cargo-ndk)
└── AndroidPlatform (Rust struct impl Platform)
     ├── key_down/up   → performAction or InputConnection dispatch
     ├── scroll_v/h    → ACTION_SCROLL_FORWARD / BACKWARD
     ├── mouse_move    → dispatchGesture() swipe
     └── media keys    → AudioManager / Intent broadcast
```

`AccModel2D` ticker thread works unchanged — `std::thread` and `std::sync` are fully
supported on Android NDK.

## What needs to be written

### Rust crate: `rs/adapters/android/` (`cdylib`)

| File | Purpose |
|---|---|
| `src/lib.rs` | crate root, re-exports JNI entry points |
| `src/platform.rs` | `AndroidPlatform` implementing the `Platform` trait |
| `src/key_map.rs` | `android_keycode → KeyCode` and reverse mapping |
| `src/jni_bridge.rs` | `extern "C"` JNI functions exposed to Kotlin |

`Cargo.toml` sketch:
```toml
[package]
name    = "capslockx-android"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
capslockx-core = { path = "../../core" }
jni            = "0.21"
once_cell      = "1.19"
```

### Kotlin side (~200–300 lines)

- `ClxAccessibilityService : AccessibilityService` — the keyboard hook
  - `onKeyEvent(event: KeyEvent): Boolean` calls JNI, returns `true` to suppress
- JNI loader in `companion object` (`System.loadLibrary("capslockx_android")`)
- Callbacks from Rust back into Kotlin for scroll / media / gesture actions

### Manifest / config

```xml
<!-- AndroidManifest.xml -->
<service android:name=".ClxAccessibilityService"
         android:permission="android.permission.BIND_ACCESSIBILITY_SERVICE">
    <intent-filter>
        <action android:name="android.accessibilityservice.AccessibilityService"/>
    </intent-filter>
    <meta-data android:name="android.accessibilityservice"
               android:resource="@xml/accessibility_service_config"/>
</service>
```

```xml
<!-- res/xml/accessibility_service_config.xml -->
<accessibility-service
    android:accessibilityEventTypes="typeAllMask"
    android:accessibilityFlags="flagRequestFilterKeyEvents"
    android:canRequestFilterKeyEvents="true"/>
```

The flag `flagRequestFilterKeyEvents` / `canRequestFilterKeyEvents` is what grants
`onKeyEvent()` callbacks — without it the service never sees keyboard input.

## Build toolchain

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android    # ARM64 — all modern devices
rustup target add armv7-linux-androideabi  # ARM32 — older devices (optional)

# Build the .so
cargo ndk -t arm64-v8a build -p capslockx-android --release
# output: rs/target/aarch64-linux-android/release/libcapslockx_android.so
```

Copy the `.so` into the Android project's `app/src/main/jniLibs/arm64-v8a/`.

## Key-code mapping note

Android uses `KeyEvent.KEYCODE_*` integers (e.g. `KEYCODE_CAPS_LOCK = 115`).
A `android_keycode_to_clx(code: i32) -> KeyCode` lookup table is needed in
`key_map.rs`, analogous to `rs/adapters/browser/src/key_map.rs`.

## CapsLock caveat

Most Android versions consume `KEYCODE_CAPS_LOCK` before the Accessibility Service
sees it (to toggle caps-lock state). The trigger key may need to be remapped to
something Android doesn't eat — e.g. right-Alt (`KEYCODE_ALT_RIGHT`) or made
user-configurable. This is a config change, not an architecture change.

## What does NOT need to change

- `capslockx-core` — zero changes; `Platform` trait and `ClxEngine` are already
  platform-agnostic.
- `AccModel2D` — the background ticker thread runs as-is on NDK.
- All physics, module logic, and key suppression logic.

## Reference: browser adapter as template

The browser adapter (`rs/adapters/browser/`) is the closest analogue:

| Browser adapter | Android adapter |
|---|---|
| `wasm-bindgen` entry point | JNI `extern "C"` functions |
| `web-sys` keyboard events | `jni` crate + Android `KeyEvent` |
| `setInterval(tick, 16)` | Rust ticker thread (already works on NDK) |
| `window.scrollBy()` | `AccessibilityNodeInfo.performAction(SCROLL_*)` |
| `isTrusted` self-inject filter | separate JNI flag or `KeyEvent.getFlags()` |

## Effort estimate

| Component | Lines |
|---|---|
| `rs/adapters/android/` (Rust) | ~400–600 |
| Kotlin `AccessibilityService` + JNI glue | ~200–300 |
| Manifest + config XML | ~50 |
| **Total** | ~1 week |
