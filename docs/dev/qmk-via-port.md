# Porting CapsLockX to QMK / VIA — Feasibility Notes

## TL;DR
**Partial port is feasible and interesting; full port is not.** Roughly the
input-remapping ~60% of CapsLockX could live in keyboard firmware (QMK +
VIA Custom UI), giving you a CLX-on-any-OS-with-zero-install experience.
The "smart" features (LLM agent, voice STT, window manager, virtual
desktops, brainstorm) require host APIs that don't exist on a microcontroller
and must stay host-side. A **hybrid model** — firmware handles fast-path
input remapping, a tiny host companion handles smart features via Raw HID —
is the most promising direction.

## Architectural mapping

### What lives natively in QMK ✅

| CapsLockX Module | QMK Equivalent | Notes |
|---|---|---|
| Trigger-key state machine (CapsLock / Space hold → CLX mode) | Layer + `LT()` / `MO()` / Tap-Hold | CLX's `CM_FN`/`CM_CLX` bitmask maps cleanly to two QMK layers. `Space` becomes `LT(CLX, KC_SPC)` with PERMISSIVE_HOLD + IGNORE_MOD_TAP_INTERRUPT tuned. |
| `CLX-Edit` (HJKL cursor, YUIO PgUp/Dn/Home/End, G=Enter, T=Del) | Per-key keymap on the CLX layer | Trivial — pure keycode mapping. |
| `CLX-MediaKeys` (F5–F11 → media/volume) | `KC_MEDIA_*`, `KC_AUDIO_*` consumer keys | Trivial. |
| `CLX-Mouse` WASD movement + QE buttons + RF scroll | QMK Mouse Keys (`KC_MS_*`) **or** custom `pointing_device` driver | Stock Mouse Keys is jerky; CLX's `AccModel2D` (exp+polynomial accel + damping) is the *whole point*. Reimplementing it in C is ~150 LoC and runs fine on STM32/RP2040. |
| `CLX-EnterWithoutBreak`, `CLX-PinchZoom`, `CLX-Lianki` | Per-key macros / tap dance | Mostly trivial macros. |
| Speeds / toggles (cursor_speed, mouse_speed, etc.) | **VIA Custom UI** sliders → EEPROM, read by AccModel | The killer feature: live-tuning physics from a GUI without reflashing. |

### What CANNOT live in QMK ❌

| CapsLockX Module | Why it can't move to firmware |
|---|---|
| `CLX-Brainstorm` (Space+B LLM chat) | Needs HTTPS, JSON, multi-MB streaming. No network stack on a keyboard. |
| `CLX-Agent` (Space+M LLM controls computer) | Needs to read screen pixels, AX tree, send arbitrary input. Firmware can't see the host. |
| `Voice` (Space+V STT) | Needs microphone capture, ONNX/whisper inference, DSP. ~1GB models. Impossible. |
| `WindowManager` (Z/X/C) | Needs OS window enumeration. Firmware can only send keystrokes. |
| `VirtualDesktop` (1-9) | Same — these are *macros* of OS shortcuts; technically possible as keymap macros but the *cycle/tile* logic is host-side. |
| `NodeEval` (JS scripting) | rquickjs / boa won't fit. |
| `LaptopKeyboardFix` | Specific to disabling the host's built-in keyboard via OS API. |

### The grey zone 🌗

| Feature | Firmware? | Notes |
|---|---|---|
| Window-cycle key (Z) | Half | Firmware can send `Alt+Tab` / `Alt+Shift+Tab`, but the smart "next window matching app" logic needs the host. |
| Virtual desktop 1-9 | Half | Firmware sends `Win+Ctrl+→`, but "jump to desktop N" needs host bookkeeping. |
| Locale-aware shortcuts | Half | Firmware can switch keymaps per OS via VIA layers, but locale detection is host-side. |
| Mouse acceleration profile | Yes | `AccModel2D` ports cleanly. The `cpal`/`audio_capture` deps go away. |

## Three concrete porting strategies

### Strategy A — "QMK-only fork" (clean, limited)
Build a separate `clx-firmware/` repo using QMK userspace. Hand-port `mouse.rs` →
`cmouse.c` and the layer state machine. Drop everything host-side. Ship as
a downloadable `.hex` for popular boards (Planck, Corne, Ergodox). VIA Custom
UI exposes 6 sliders (cursor speed, mouse speed, scroll speed, accel exponent,
damping, max speed).

- ✅ Zero install on host. Works on Linux/macOS/Windows/iPad/Android.
- ✅ No driver, no signing, no permission dialogs.
- ❌ Loses ~40% of features (the cool ones).
- ❌ Hand-maintained C duplicate of Rust logic — drift risk.

### Strategy B — "Hybrid: firmware + host companion via Raw HID" (recommended)
Firmware does the fast-path subset from Strategy A. Additionally, it exposes
a Raw HID interface (`RAW_ENABLE = yes` in QMK). A small host companion
(`clx-companion`, ~10MB Rust binary, no Tauri) listens on Raw HID and
provides the smart features:

```
keyboard ─Raw HID─→ clx-companion ─OS APIs─→ host
   │                       │
   │  "agent please"       │  spawns LLM agent loop
   │  "brainstorm Y"       │  streams overlay
   │  "window cycle"       │  enumerates windows
   │  "vdesk 3"            │  switches desktop
   │  "voice toggle"       │  starts STT
```

Firmware sends *intent* (a 1-byte command + payload), companion does the
heavy lift. Configuration syncs both ways: VIA Custom UI for firmware
params, companion's tray menu for LLM keys / voice models.

- ✅ Keeps every CapsLockX feature.
- ✅ Companion is much smaller than current `clx-rust.exe` — no input
  hook, no tray, no Tauri webview, just a Raw HID listener + the LLM/voice/wm modules.
- ✅ Input latency drops to firmware level (~1ms) for the fast path.
- ✅ Permissions surface shrinks: companion needs accessibility for window
  management but no input-monitoring (the keyboard sends keys directly).
- ❌ Need to maintain firmware *and* companion.
- ❌ Companion still needs OS-specific code (the same modules we have today).

### Strategy C — "Compile core to embedded Rust" (research)
Theoretically, `rs/core/` could target `thumbv6m-none-eabi` for RP2040. The
engine + state machine + modules/{mouse,edit,media} are mostly `no_std`-able.
QMK has experimental Rust interop via the `qmk-rs` project (very alpha as of
2025). The pieces:

- `acc_model.rs` — pure math, `no_std` after stripping `std::time::Instant`.
- `engine.rs` + `state.rs` — `no_std` after replacing `RwLock` with `critical_section`.
- `modules/edit.rs`, `mouse.rs`, `media.rs` — port platform calls to a `Platform` impl that talks to QMK's `register_code` / `pointing_device_set_report`.
- Drop everything else (`agent`, `brainstorm`, `voice`, `task_manager`, `tts`, `cloud_stt`, `llm_client`, `local_*`, `audio_capture`).

- ✅ Single source of truth for the input logic. No drift.
- ✅ Same `AccModel2D` Rust code on host *and* firmware.
- ❌ QMK is a C codebase; Rust interop is bleeding-edge and mostly unproven.
- ❌ Build complexity: cargo + qmk Makefile + linker scripts.
- ❌ Probably 2-3× the effort of Strategy A for marginal gain over Strategy B.

## VIA Custom UI sketch
For Strategy A or B, expose these via VIA's `via.json` + raw HID custom values:

```jsonc
{
  "menus": [{
    "label": "CapsLockX",
    "content": [
      { "label": "Cursor speed",  "type": "range", "options": [1,20], "id":[0,1] },
      { "label": "Mouse speed",   "type": "range", "options": [1,30], "id":[0,2] },
      { "label": "Scroll speed",  "type": "range", "options": [1,15], "id":[0,3] },
      { "label": "Accel curve",   "type": "range", "options": [10,40], "id":[0,4] },
      { "label": "Damping",       "type": "range", "options": [50,99], "id":[0,5] },
      { "label": "Trigger: Space hold-to-CLX", "type": "toggle", "id":[0,6] },
      { "label": "Trigger: CapsLock hold-to-CLX", "type": "toggle", "id":[0,7] }
    ]
  }]
}
```

Values persist in EEPROM via QMK's `eeconfig_user`.

## Hardware targets that make sense
- **RP2040 boards** (Raspberry Pi Pico, KB2040, ProMicro RP2040) — cheap, fast,
  enough flash, native USB, well supported by QMK.
- **STM32F4** (ProMicro F4, Bluepill) — plenty of cycles for AccModel.
- **Avoid AVR/ATmega32u4** — Pro Micro classic. Tight on flash; mouse accel
  math will fight for space with the keymap.

## Recommended next step
If you want to actually pursue this: do **Strategy B** in two phases.

1. **Phase 1 — Firmware MVP.** Fork QMK, set up a userspace folder
   `users/snomiao/clx/`, port the layer state machine + AccModel mouse to C
   for one keyboard you actually own. Get VIA Custom UI sliders working.
   Ship a `.hex`. ~1-2 weeks of focused work.
2. **Phase 2 — Companion handshake.** Add `RAW_ENABLE = yes`. Define a
   minimal protocol (`u8 cmd, u8 len, u8[len] payload`). Strip the current
   `rs/adapters/windows/` down to a Raw-HID-listener binary that reuses the
   existing `agent`, `brainstorm`, `voice`, `window_manager`, `virtual_desktop`
   modules from `rs/core/`. The hook + SendInput path goes away entirely on
   that keyboard.

## Things this would *eliminate* from the current Windows adapter
- `WH_KEYBOARD_LL` hook (the keyboard sends keys directly)
- `SendInput` for the fast path (firmware already sent them)
- Cursor visibility hack (a real HID mouse means no `CURSOR_SUPPRESSED`)
- Elevation prompt (no global hook = no admin needed)
- Single-instance kill logic (no shared input state to fight over)
- The whole tray icon (companion can be headless or use a tiny tray)

That last bullet is the most attractive thing about this whole exercise:
**a CapsLockX-equipped keyboard would just work, on any computer, with
nothing installed beyond the optional companion for smart features.**

## Open questions / things I haven't checked
- Whether QMK Mouse Keys + report rate can match CLX's current 16ms tick.
  Should be fine on RP2040 (USB HS 1ms reports), needs measurement on AVR.
- Whether VIA Custom UI's value-set IPC is fast enough for live tuning,
  or if we'd need raw-HID for that too.
- License compatibility: QMK is GPLv2, CapsLockX is GPLv3. Userspace folder
  in QMK can be GPLv3 if kept self-contained; merging upstream would need
  relicensing or a clean reimplementation. Probably keep it as a personal
  fork.
- Whether `boa_engine` / `rquickjs` could ever fit on hardware (no — but
  could a tiny custom DSL replace `CLX-NodeEval` for firmware-side scripting?).
