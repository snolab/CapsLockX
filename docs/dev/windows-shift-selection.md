# Windows: CLX+Shift+HJKL text selection & modifier-injection isolation

How CLX makes `Space/CapsLock + Shift + HJKL/YUIO` *select* text on Windows, and
the multi-day rabbit hole that led to the (simple) final solution.

## TL;DR

Inject the navigation keys (arrows / Home / End / PageUp / PageDown) **by
scancode with `KEYEVENTF_EXTENDEDKEY`**, let the user's **physical Shift pass
through untouched**, and do **nothing else** — no synthetic Shift, no
suppression, no timing. This mirrors the original AHK CapsLockX
(`SendEvent {Blind}{Left}`) and the macOS adapter (modifier flags on the
`CGEvent`). See `adapters/windows/src/output.rs::kbd` and `core/src/engine.rs`
step 3a.

## The problem: modifier-injection isolation

On Windows, when you inject a keystroke with `SendInput`/`keybd_event` using a
**virtual-key code** while the user is **physically holding a modifier**
(Shift), the OS *isolates* the injected key from the physical modifier: it emits
a spurious `LSHIFT UP` (and later a restore `LSHIFT DN`) around your injected
arrow, so the arrow lands **un-shifted** and the caret just moves instead of
selecting.

Verified directly from the low-level hook log — every injected arrow was
bracketed by an `inj=false` Shift lift:

```
LShift DN  inj=false                 <- user holds Shift (passed through)
Left   DN  inj=true  ours=true        <- our injected arrow
LShift UP  inj=false                  <- OS "isolation" lift -> arrow is now unshifted
```

This happens with **both** `SendInput` and `keybd_event` when the key is sent by
**VK**. It is below the LL hook, so it cannot be prevented from user space —
only avoided.

## Dead ends (do not repeat these)

These all "work" for *continuous* selection but fall apart on the transitions
(tap-extend, releasing Shift mid-drag, the deceleration tail):

1. **Synthetic Shift + suppress the physical Shift.** Hold one app-visible
   Shift down for the gesture and swallow every physical Shift event. The arrows
   stay shifted, BUT the OS still emits `inj=false` isolation Shift-ups, which
   are byte-identical to a real release — so deciding *when to release* the
   synth Shift becomes guesswork.
2. **Timing heuristics** ("off-beat" release, only release if &gt;N ms since the
   last injected arrow). Measured the artifact delay at anywhere from **2 ms to
   150 ms+**; no threshold separates a real release from a late artifact. There
   is *always* a tap cadence that defeats it. **This is the wrong axis entirely
   — AHK and macOS never time modifiers.**
3. **`GetAsyncKeyState` / `GetKeyState` / `GetKeyboardState`.** All
   **injection-poisoned**: while we hold the synthetic Shift they read "down"
   even after the user physically lets go. Measured: `async SH=1` persisted
   through the real release and past key-up.
4. **Raw Input (`WM_INPUT`).** The *only* API that reports the physical HID
   device (excludes injected events) — but in our process it delivered **zero
   `WM_INPUT`** despite `RegisterRawInputDevices` succeeding, even in a minimal
   no-Tauri, shown-window build. Left unsolved; not needed by the final fix.

## The fix: scancode + extended-key injection (AHK's `{Blind}`)

The clue was in the AHK source (`Modules/CLX-Edit.ahk`):

```ahk
; 这里用 SendEvent 防止把 hl 按出来
SendEvent {Blind}{Left}
```

- `{Blind}` = send the bare key, do **not** touch modifier state — the user's
  *physical* Shift applies.
- AHK's `SendEvent` sends **scancodes with the extended-key flag**, not bare
  VKs.

Sending by scancode makes Windows treat the injected key as a real hardware key,
so a physically-held Shift modifies it **without** triggering the isolation
lift. Replicated in Rust:

```rust
// adapters/windows/src/output.rs
fn kbd(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let mut f = flags | KEYEVENTF_SCANCODE;
    if is_extended_vk(vk) { f |= KEYEVENTF_EXTENDEDKEY; } // arrows / nav cluster
    // ... wScan = scan, dwFlags = f
}
```

`is_extended_vk` covers the nav cluster (`0x21–0x28`, `0x2D/0x2E`), Win keys, and
right-side modifiers — they share scancodes with the numpad and need the `E0`
prefix.

With that, the engine's Shift handling collapses to a single line (no synth, no
suppression, no timing):

```rust
// core/src/engine.rs step 3a
if matches!(code, KeyCode::Shift | KeyCode::LShift | KeyCode::RShift) {
    self.state.set_shift_held(pressed); // track for mouse-precision; pass through
}
```

`held_modifiers` (in `core/src/modules/edit.rs`) omits Shift on Windows (the real
one applies); macOS still embeds the flag on each injected `CGEvent`.

## Why this is correct, not just lucky

Selection now *is* the physical Shift — start/stop are exactly the real key, so
tap-extend, mid-drag release, and the deceleration coast all "just work" with no
state to get out of sync. There is nothing to time because there is no synthetic
modifier and therefore no isolation artifacts.

## Cross-platform model

| Platform | How a modifier reaches an injected key |
|----------|----------------------------------------|
| macOS    | Flags set directly on the `CGEvent` (`CGEventSetFlags`). |
| Windows  | Physical modifier passes through; key injected **by scancode + extended** so the OS lets it apply. |

Both avoid a separate stateful "held modifier" that has to be tracked/released.

## Debugging tooling (removed, but worth recreating if needed)

A localhost Node server + HTML page (`tmp/clx-keytest.html`) showed, live and
side-by-side: what the app actually receives (arrow events + `shiftKey`), the
live selection length, and lamps for physical vs. injected vs. raw Shift parsed
from clx's hook log. This input-vs-output comparison is what finally made the
isolation visible and the scancode fix verifiable.
