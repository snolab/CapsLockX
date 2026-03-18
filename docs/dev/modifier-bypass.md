# Modifier+Space Bypass on macOS

## Problem

CapsLockX uses Space as a trigger key. When Space is pressed, the CGEventTap intercepts it and enters "FN mode" instead of passing it to the OS. This breaks system shortcuts like:

- **Cmd+Space** — Spotlight / input method switcher
- **Shift+Space** — input method switching (CJK)
- **Win+Space** — input language switcher (Windows)

## Solution: Three-layer bypass

### Layer 1: Detect modifier+Space in `clx_dn`

When Space is pressed as a trigger key, `clx_dn()` checks if Shift or Cmd/Win is currently held by inspecting `held_keys`. If so, it returns `true` (bypass) instead of entering FN mode.

```rust
let bypass = if code == KeyCode::Space {
    shift_held || win_held
} else {
    non_modifier_held
};
```

### Layer 2: Sync CGEvent modifier flags (macOS-specific)

On macOS, fast Cmd+Space combos cause a race: the `FlagsChanged` key-up event for Cmd can arrive *before* the Space key-down event. This removes Cmd from `held_keys` before the bypass check runs.

Fix: on every KeyDown event, read the CGEvent's modifier flags (which macOS stamps on the event at creation time) and inject any held modifiers into `held_keys`:

```rust
// In hook.rs, before calling ENGINE.on_key_event():
if pressed {
    let flags = event.get_flags();
    sync_modifier_flags(&flags);  // ensures LWin is in held_keys
}
```

### Layer 3: Pass through both key-down AND key-up

macOS (and Windows) require a complete key-down + key-up cycle for shortcuts to trigger. The original bug:

| Event | Engine response | OS sees |
|-------|----------------|---------|
| Cmd down (FlagsChanged) | PassThrough | Cmd down |
| Space down (KeyDown) | **PassThrough** (bypass) | Space down |
| Space up (KeyUp) | **Suppress** (trigger key default) | *nothing* |
| Cmd up (FlagsChanged) | PassThrough | Cmd up |

The OS saw Cmd+Space down but never saw Space up — so the shortcut didn't fire.

Fix: track bypass state with `trigger_bypassed: AtomicBool`. When `clx_dn` returns bypass=true, set the flag. On the subsequent key-up for the same trigger key, check the flag and return `PassThrough` instead of `Suppress`:

```rust
// Key down:
if self.clx_dn(code, prior) {
    self.trigger_bypassed.store(true, Ordering::Relaxed);
    return CoreResponse::PassThrough;
}

// Key up:
if self.trigger_bypassed.swap(false, Ordering::Relaxed) {
    return CoreResponse::PassThrough;  // complete the down+up pair
}
self.clx_up(code);
```

### Layer 4: Never suppress modifier FlagsChanged events (macOS)

On macOS, the CGEventTap hook always passes through `FlagsChanged` events (modifier key presses). Suppressing them would prevent macOS from tracking modifier state, breaking all modifier-based shortcuts system-wide. The engine still processes them internally for its own state tracking.

## Debugging

Add `eprintln!` in these locations to diagnose bypass issues:

1. `clx_dn` — log `shift_held`, `win_held`, `bypass`, `held_keys`
2. `hook.rs` FlagsChanged handler — log keycode, pressed state
3. `hook.rs` KeyDown handler — log CGEvent flags (`event.get_flags().bits()`)

Check that:
- Cmd FlagsChanged events reach the engine (not being filtered)
- LWin appears in `held_keys` when Space arrives
- Both Space down AND Space up return PassThrough when bypassed

## Files involved

- `rs/core/src/engine.rs` — bypass logic, `trigger_bypassed` flag, `ensure_held()`
- `rs/adapters/macos/src/hook.rs` — `sync_modifier_flags()`, FlagsChanged always PassThrough
