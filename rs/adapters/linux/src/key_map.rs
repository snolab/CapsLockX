//! evdev Key ↔ KeyCode mapping (Linux only).
//!
//! Numeric codes come from linux/input-event-codes.h.
//! Mirror of rs/adapters/windows/src/vk.rs for the Linux side.

use evdev::Key;
use capslockx_core::KeyCode;

/// Convert an evdev [`Key`] to the platform-agnostic [`KeyCode`].
pub fn evdev_key_to_keycode(key: Key) -> KeyCode {
    match key.code() {
        // ── Trigger keys ──────────────────────────────────────────────────────
        58  => KeyCode::CapsLock,
        57  => KeyCode::Space,
        110 => KeyCode::Insert,
        70  => KeyCode::ScrollLock,

        // ── Modifiers ─────────────────────────────────────────────────────────
        42  => KeyCode::LShift,
        54  => KeyCode::RShift,
        29  => KeyCode::LCtrl,
        97  => KeyCode::RCtrl,
        56  => KeyCode::LAlt,
        100 => KeyCode::RAlt,
        125 => KeyCode::LWin,
        126 => KeyCode::RWin,

        // ── Navigation / editing ──────────────────────────────────────────────
        28  => KeyCode::Enter,
        15  => KeyCode::Tab,
        111 => KeyCode::Delete,
        14  => KeyCode::Backspace,
        105 => KeyCode::Left,
        103 => KeyCode::Up,
        106 => KeyCode::Right,
        108 => KeyCode::Down,
        104 => KeyCode::PageUp,
        109 => KeyCode::PageDown,
        102 => KeyCode::Home,
        107 => KeyCode::End,

        // ── Alpha (A–Z) ───────────────────────────────────────────────────────
        30 => KeyCode::A,  48 => KeyCode::B,  46 => KeyCode::C,
        32 => KeyCode::D,  18 => KeyCode::E,  33 => KeyCode::F,
        34 => KeyCode::G,  35 => KeyCode::H,  23 => KeyCode::I,
        36 => KeyCode::J,  37 => KeyCode::K,  38 => KeyCode::L,
        50 => KeyCode::M,  49 => KeyCode::N,  24 => KeyCode::O,
        25 => KeyCode::P,  16 => KeyCode::Q,  19 => KeyCode::R,
        31 => KeyCode::S,  20 => KeyCode::T,  22 => KeyCode::U,
        47 => KeyCode::V,  17 => KeyCode::W,  45 => KeyCode::X,
        21 => KeyCode::Y,  44 => KeyCode::Z,

        // ── Function keys ─────────────────────────────────────────────────────
        59 => KeyCode::F1,  60 => KeyCode::F2,  61 => KeyCode::F3,
        62 => KeyCode::F4,  63 => KeyCode::F5,  64 => KeyCode::F6,
        65 => KeyCode::F7,  66 => KeyCode::F8,  67 => KeyCode::F9,
        68 => KeyCode::F10, 87 => KeyCode::F11, 88 => KeyCode::F12,

        // ── Media / volume ────────────────────────────────────────────────────
        164 => KeyCode::MediaPlay,
        165 => KeyCode::MediaPrev,
        163 => KeyCode::MediaNext,
        166 => KeyCode::MediaStop,
        115 => KeyCode::VolumeUp,
        114 => KeyCode::VolumeDown,
        113 => KeyCode::VolumeMute,

        other => KeyCode::Unknown(other as u32),
    }
}

/// Convert a [`KeyCode`] to the evdev [`Key`] used for uinput output.
/// Returns `None` for `KeyCode::Unknown`.
pub fn keycode_to_evdev_key(key: KeyCode) -> Option<Key> {
    let code: u16 = match key {
        // ── Trigger keys ──────────────────────────────────────────────────────
        KeyCode::CapsLock   => 58,
        KeyCode::Space      => 57,
        KeyCode::Insert     => 110,
        KeyCode::ScrollLock => 70,

        // ── Modifiers ─────────────────────────────────────────────────────────
        KeyCode::Shift  => 42,   // generic → left shift
        KeyCode::LShift => 42,
        KeyCode::RShift => 54,
        KeyCode::LCtrl  => 29,
        KeyCode::RCtrl  => 97,
        KeyCode::LAlt   => 56,
        KeyCode::RAlt   => 100,
        KeyCode::LWin   => 125,
        KeyCode::RWin   => 126,

        // ── Navigation / editing ──────────────────────────────────────────────
        KeyCode::Enter     => 28,
        KeyCode::Tab       => 15,
        KeyCode::Delete    => 111,
        KeyCode::Backspace => 14,
        KeyCode::Left      => 105,
        KeyCode::Up        => 103,
        KeyCode::Right     => 106,
        KeyCode::Down      => 108,
        KeyCode::PageUp    => 104,
        KeyCode::PageDown  => 109,
        KeyCode::Home      => 102,
        KeyCode::End       => 107,

        // ── Alpha (A–Z) ───────────────────────────────────────────────────────
        KeyCode::A => 30,  KeyCode::B => 48,  KeyCode::C => 46,
        KeyCode::D => 32,  KeyCode::E => 18,  KeyCode::F => 33,
        KeyCode::G => 34,  KeyCode::H => 35,  KeyCode::I => 23,
        KeyCode::J => 36,  KeyCode::K => 37,  KeyCode::L => 38,
        KeyCode::M => 50,  KeyCode::N => 49,  KeyCode::O => 24,
        KeyCode::P => 25,  KeyCode::Q => 16,  KeyCode::R => 19,
        KeyCode::S => 31,  KeyCode::T => 20,  KeyCode::U => 22,
        KeyCode::V => 47,  KeyCode::W => 17,  KeyCode::X => 45,
        KeyCode::Y => 21,  KeyCode::Z => 44,

        // ── Function keys ─────────────────────────────────────────────────────
        KeyCode::F1  => 59,  KeyCode::F2  => 60,  KeyCode::F3  => 61,
        KeyCode::F4  => 62,  KeyCode::F5  => 63,  KeyCode::F6  => 64,
        KeyCode::F7  => 65,  KeyCode::F8  => 66,  KeyCode::F9  => 67,
        KeyCode::F10 => 68,  KeyCode::F11 => 87,  KeyCode::F12 => 88,

        // ── Media / volume ────────────────────────────────────────────────────
        KeyCode::MediaPlay  => 164,
        KeyCode::MediaPrev  => 165,
        KeyCode::MediaNext  => 163,
        KeyCode::MediaStop  => 166,
        KeyCode::VolumeUp   => 115,
        KeyCode::VolumeDown => 114,
        KeyCode::VolumeMute => 113,

        KeyCode::Unknown(_) => return None,
    };
    Some(Key(code))
}
