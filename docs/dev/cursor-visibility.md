# Cursor Visibility on Mouseless Systems

## Problem
On laptops/tablets with no physical mouse plugged in, Windows applies
**cursor suppression**: `GetCursorInfo` reports `CURSOR_SUPPRESSED` and the
pointer is hidden until the system sees a real mouse-like input. CLX-Mouse
(WASDQERF → motion) drives the pointer via `SendInput`, but on suppressed
systems the cursor stays invisible — the feature is unusable exactly when
the user needs it most.

## Solution
While CLX mode is engaged, silently enable Windows' **Mouse Keys**
accessibility feature. Mouse Keys makes Windows treat the system as having
an active pointing device, which unsuppresses the cursor. CLX continues to
drive motion with `SendInput` on top — Mouse Keys just keeps the cursor
visible. On CLX exit, the prior Mouse Keys state is restored.

Key design points:

- **No driver, no signing.** Pure user-mode `SystemParametersInfoW` calls.
- **No numpad hijack.** We set `MKF_MOUSEKEYSON | MKF_AVAILABLE` but
  *deliberately omit* `MKF_REPLACENUMBERS`, so the numpad keeps typing
  digits as normal. The user never notices Mouse Keys is on.
- **Reversible.** `enable()` snapshots the current `MOUSEKEYS` struct via
  `SPI_GETMOUSEKEYS` and `disable()` writes it back. Snapshot lives in a
  process-local `Mutex<Option<MOUSEKEYS>>` (see `cursor_visibility.rs`).
- **Belt-and-braces nudge.** On every key event during CLX mode we send a
  zero-delta `MOUSEEVENTF_MOVE` via `SendInput`. This is the documented way
  to wake the cursor from `CURSOR_SUPPRESSED` and covers edge cases where
  Mouse Keys alone isn't enough.
- **Idempotent.** `enable()` early-returns if already enabled; `disable()`
  early-returns if no snapshot exists. Safe to call from edge transitions.

## Code map
| File | Role |
| --- | --- |
| `rs/adapters/windows/src/cursor_visibility.rs` | `enable()` / `disable()` / `nudge()` |
| `rs/adapters/windows/src/hook.rs` | Calls them on CLX-mode edge transitions and nudges on every key while held |
| `rs/adapters/windows/src/main.rs` | Final `disable()` on shutdown so Mouse Keys never leaks past process exit |
| `rs/adapters/windows/Cargo.toml` | Adds `Win32_UI_Accessibility` feature for `MOUSEKEYS` |

## Why Mouse Keys (and not the alternatives)
- **Just `SendInput` nudges** — unreliable on Win11 touch SKUs; suppression
  often persists until a *real* HID device reports motion.
- **`ShowCursor(TRUE)`** — per-thread, doesn't affect other processes,
  doesn't defeat suppression.
- **Interception driver (Oblita)** — works, but requires the user to
  install a third-party signed driver and reboot. Reserved as Stage 2
  fallback if Mouse Keys ever stops working.
- **Custom KMDF virtual HID mouse** — needs WDK + EV cert. Way too heavy
  for a feature whose entire job is "make the cursor visible".

## Verification
1. Disable touchpad / unplug mouse so cursor disappears from desktop.
2. Hold `CapsLock` (or `Space`) — cursor should appear immediately.
3. Press WASDQERF — cursor moves, clicks, scrolls visibly.
4. Release the trigger — cursor returns to its prior suppressed state.
5. Quit `clx-rust.exe` — confirm Mouse Keys is **off** in Settings →
   Accessibility → Mouse (it should match its pre-launch state exactly).

## Future
If a Win11 build ever ignores the Mouse Keys trick, fall back to bundling
Oblita's Interception driver and routing CLX motion through it — see
`docs/dev/cursor-visibility.md` Stage 2 in the original plan
(`~/.claude/plans/velvety-twirling-whistle.md`).
