# Input Injection APIs

## Keyboard

### macOS
- **CGEventCreateKeyboardEvent** + **CGEventPost** at `kCGHIDEventTap`
- `CGEventKeyboardSetUnicodeString` for arbitrary Unicode
- Requires Accessibility permission
- Already implemented in CapsLockX (`rs/adapters/macos/src/output.rs`)

### Windows
- **SendInput** with `INPUT_KEYBOARD` + `KEYBDINPUT`
- Use `KEYEVENTF_SCANCODE` for game compatibility (DirectInput reads scancodes)
- Already implemented in CapsLockX (`rs/adapters/windows/`)

### Rust crates
- `enigo` — cross-platform, simple API
- `rdev` — cross-platform listen + simulate
- **Reuse CapsLockX's existing injection code** (battle-tested)

## Mouse

### macOS
- **CGEventCreateMouseEvent** — full mouse event (move, click, drag)
- **CGWarpMouseCursorPosition** — instant teleport (no move event generated)
- Supports float coordinates (sub-pixel at API level)
- Scroll: `CGEventSetIntegerValueField` with `kCGScrollWheelEventDeltaAxis1`

### Windows
- **SendInput** with `INPUT_MOUSE` + `MOUSEINPUT`
- `MOUSEEVENTF_ABSOLUTE` — coordinates in 0-65535 normalized range
- `MOUSEEVENTF_MOVE_NOCOALESCE` — prevent coalescing (important for fast injection)
- `MOUSEEVENTF_MOVE` — relative movement

### Smooth curves (lerp)
- Executor thread interpolates between current and target position
- Bezier or linear interpolation, configurable
- Update rate: 120-240 Hz (sufficient for 60fps games)
- Use `spin_sleep` crate for sub-ms timing between updates

## Gamepad / Controller

### Windows — ViGEm
- **ViGEmBus** kernel driver + **ViGEmClient** userspace library
- Creates virtual Xbox 360 or DualShock 4 controllers
- Appears as real hardware to XInput and DirectInput
- Analog sticks: `i16` (-32768 to 32767) for X/Y axes
- Triggers: `u8` (0-255)
- Rust crate: `vigem-client` or FFI wrapper

### macOS — IOHIDUserDevice
- Create virtual HID device with gamepad report descriptor
- Define axes (LX, LY, RX, RY), triggers, buttons, D-pad
- Visible to GameController framework and Steam Input
- Requires DriverKit entitlements for distribution
- Alternative: **Karabiner-VirtualHIDDevice** approach

### Analog stick simulation
- Values are continuous floats mapped to hardware range
- Circular dead zone handling (games apply 10-20% dead zone)
- Normalize diagonal vectors to unit circle
- Update at 60+ Hz for smooth movement

## MIDI

### macOS — CoreMIDI
- `MIDIClientCreate` → `MIDISourceCreate` (virtual output port)
- `MIDISend` with `MIDIPacketList`
- Timestamps via `mach_absolute_time` (nanosecond resolution)
- IAC Driver provides built-in virtual MIDI bus

### Windows
- WinMM: `midiOutOpen` → `midiOutShortMsg` (legacy, 1-10ms latency)
- Virtual ports: **loopMIDI** (free) or **virtualMIDI SDK**

### Rust crates
- **`midir`** — cross-platform MIDI I/O, virtual ports on macOS/Linux
- **`wmidi`** — strongly-typed MIDI message construction
- **`midi-msg`** — message parsing/construction

## Timing & Precision

### Achievable precision
- macOS: `mach_absolute_time` — ~40ns resolution (Apple Silicon)
- Windows: `QueryPerformanceCounter` — ~100ns resolution
- Practical input injection: **sub-millisecond** with spin-wait

### Rust crates for timing
- **`spin_sleep`** — sleep + spin-wait hybrid, sub-ms precision
- **`quanta`** — minimal-overhead high-res timer
- `std::time::Instant` — uses platform-native timers

### Real-time thread setup
- macOS: `pthread_setschedparam` with `SCHED_FIFO`
- Windows: `AvSetMmThreadCharacteristics("Games")` via MMCSS
- Pre-allocate all event structs outside hot path
- Lock memory pages to prevent page faults

## Screen Capture (Feedback Loop)

### macOS — ScreenCaptureKit (12.3+)
- `SCStream` delivers `CMSampleBuffer` frames via delegate
- Hardware-accelerated (Metal-backed)
- Latency: 1-3 frames (16-50ms at 60fps)
- Rust: `screencapturekit-rs` crate

### Windows — DXGI Desktop Duplication
- `IDXGIOutputDuplication::AcquireNextFrame`
- GPU-resident `ID3D11Texture2D`
- Latency: ~1 frame (sub-16ms)
- Rust: `captrs` or `xcap` crates

### Pipeline for LLM feedback
1. Capture frame at 2-10 fps (not every frame — LLM can't keep up)
2. Downscale to 256x256 or smaller
3. Encode as JPEG (fast, small)
4. Send to Gemini Live API or vision model
5. Response feeds back into agent command stream
