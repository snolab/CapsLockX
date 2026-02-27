//! LinuxPlatform – implements the `Platform` trait using evdev uinput.
//!
//! Call `init()` once at startup before any hook threads are spawned.
//! All output methods are thread-safe via the global `Mutex<VirtualDevice>`.

use std::sync::Mutex;

use evdev::{
    uinput::VirtualDeviceBuilder,
    AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
};
use once_cell::sync::OnceCell;

use capslockx_core::{KeyCode, Platform};
use capslockx_core::platform::{ArrangeMode, MouseButton};

use crate::key_map::keycode_to_evdev_key;

// ── Global virtual device ─────────────────────────────────────────────────────

static VDEV: OnceCell<Mutex<evdev::uinput::VirtualDevice>> = OnceCell::new();

// ── Initialisation ────────────────────────────────────────────────────────────

/// Create the uinput virtual device.  Must be called once before any
/// `LinuxPlatform` methods are invoked.
pub fn init() {
    let mut keys = AttributeSet::<Key>::new();

    // Trigger keys
    keys.insert(Key(58));   // KEY_CAPSLOCK
    keys.insert(Key(57));   // KEY_SPACE
    keys.insert(Key(110));  // KEY_INSERT
    keys.insert(Key(70));   // KEY_SCROLLLOCK

    // Modifiers
    keys.insert(Key(42));   // KEY_LEFTSHIFT
    keys.insert(Key(54));   // KEY_RIGHTSHIFT
    keys.insert(Key(29));   // KEY_LEFTCTRL
    keys.insert(Key(97));   // KEY_RIGHTCTRL
    keys.insert(Key(56));   // KEY_LEFTALT
    keys.insert(Key(100));  // KEY_RIGHTALT
    keys.insert(Key(125));  // KEY_LEFTMETA
    keys.insert(Key(126));  // KEY_RIGHTMETA

    // Navigation / editing
    keys.insert(Key(28));   // KEY_ENTER
    keys.insert(Key(15));   // KEY_TAB
    keys.insert(Key(111));  // KEY_DELETE
    keys.insert(Key(14));   // KEY_BACKSPACE
    keys.insert(Key(105));  // KEY_LEFT
    keys.insert(Key(103));  // KEY_UP
    keys.insert(Key(106));  // KEY_RIGHT
    keys.insert(Key(108));  // KEY_DOWN
    keys.insert(Key(104));  // KEY_PAGEUP
    keys.insert(Key(109));  // KEY_PAGEDOWN
    keys.insert(Key(102));  // KEY_HOME
    keys.insert(Key(107));  // KEY_END

    // Alpha A–Z
    for code in [30u16, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38,
                 50, 49, 24, 25, 16, 19, 31, 20, 22, 47, 17, 45, 21, 44] {
        keys.insert(Key(code));
    }

    // Function F1–F12
    for code in [59u16, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88] {
        keys.insert(Key(code));
    }

    // Media / volume
    keys.insert(Key(164));  // KEY_PLAYPAUSE
    keys.insert(Key(165));  // KEY_PREVIOUSSONG
    keys.insert(Key(163));  // KEY_NEXTSONG
    keys.insert(Key(166));  // KEY_STOPCD
    keys.insert(Key(115));  // KEY_VOLUMEUP
    keys.insert(Key(114));  // KEY_VOLUMEDOWN
    keys.insert(Key(113));  // KEY_MUTE

    // Mouse buttons
    keys.insert(Key(0x110));  // BTN_LEFT
    keys.insert(Key(0x111));  // BTN_RIGHT
    keys.insert(Key(0x112));  // BTN_MIDDLE

    // Relative axes for mouse movement and scrolling
    let mut axes = AttributeSet::<RelativeAxisType>::new();
    axes.insert(RelativeAxisType::REL_X);
    axes.insert(RelativeAxisType::REL_Y);
    axes.insert(RelativeAxisType::REL_WHEEL);
    axes.insert(RelativeAxisType::REL_HWHEEL);

    let vdev = VirtualDeviceBuilder::new()
        .expect("failed to open /dev/uinput (is the uinput module loaded?)")
        .name("CapsLockX")
        .with_keys(&keys)
        .expect("failed to declare keys on virtual device")
        .with_relative_axes(&axes)
        .expect("failed to declare relative axes on virtual device")
        .build()
        .expect("failed to build VirtualDevice");

    VDEV.set(Mutex::new(vdev))
        .expect("output::init() called more than once");

    eprintln!("[CLX] uinput virtual device ready");
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn with_vdev<F: FnOnce(&mut evdev::uinput::VirtualDevice)>(f: F) {
    let guard = VDEV.get().expect("output::init() was not called");
    let mut dev = guard.lock().unwrap();
    f(&mut dev);
}

/// Emit a key press or release + SYN_REPORT to uinput.
fn emit_key(key: Key, pressed: bool) {
    with_vdev(|dev| {
        let value = if pressed { 1 } else { 0 };
        let events = [
            InputEvent::new_now(EventType::KEY, key.code(), value),
            InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
        ];
        dev.emit(&events).unwrap_or_else(|e| eprintln!("[CLX] emit error: {e}"));
    });
}

/// Forward a raw event from the grabbed physical device to uinput.
/// Appends a SYN_REPORT so the consumer sees a complete event frame.
pub fn passthrough(event: InputEvent) {
    with_vdev(|dev| {
        let events = [
            event,
            InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
        ];
        dev.emit(&events).unwrap_or_else(|e| eprintln!("[CLX] passthrough emit error: {e}"));
    });
}

// ── LinuxPlatform ─────────────────────────────────────────────────────────────

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Platform for LinuxPlatform {
    // ── Keyboard output ───────────────────────────────────────────────────────

    fn key_down(&self, key: KeyCode) {
        if let Some(k) = keycode_to_evdev_key(key) {
            emit_key(k, true);
        }
    }

    fn key_up(&self, key: KeyCode) {
        if let Some(k) = keycode_to_evdev_key(key) {
            emit_key(k, false);
        }
    }

    // ── Mouse output ──────────────────────────────────────────────────────────

    fn mouse_move(&self, dx: i32, dy: i32) {
        with_vdev(|dev| {
            let events = [
                InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_X.0, dx),
                InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_Y.0, dy),
                InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
            ];
            dev.emit(&events).unwrap_or_else(|e| eprintln!("[CLX] mouse_move error: {e}"));
        });
    }

    fn scroll_v(&self, delta: i32) {
        with_vdev(|dev| {
            let events = [
                InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, delta),
                InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
            ];
            dev.emit(&events).unwrap_or_else(|e| eprintln!("[CLX] scroll_v error: {e}"));
        });
    }

    fn scroll_h(&self, delta: i32) {
        with_vdev(|dev| {
            let events = [
                InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_HWHEEL.0, delta),
                InputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
            ];
            dev.emit(&events).unwrap_or_else(|e| eprintln!("[CLX] scroll_h error: {e}"));
        });
    }

    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let key = match button {
            MouseButton::Left   => Key(0x110),  // BTN_LEFT
            MouseButton::Right  => Key(0x111),  // BTN_RIGHT
            MouseButton::Middle => Key(0x112),  // BTN_MIDDLE
        };
        emit_key(key, pressed);
    }

    // ── Window management: no-op (avoids X11/Wayland dependency) ─────────────
    // Default implementations from the Platform trait are used.
    // An x11rb/xcb backend can override these later.

    fn cycle_windows(&self, _dir: i32) {}
    fn arrange_windows(&self, _mode: ArrangeMode) {}
    fn close_tab(&self) { self.key_tap_ctrl(KeyCode::W); }
    fn close_window(&self) {}
    fn kill_window(&self) {}
    fn set_window_transparent(&self, _alpha: u8) {}
    fn restore_window(&self) {}
    fn toggle_window_topmost(&self) {}
}
