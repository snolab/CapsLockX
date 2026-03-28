//! macOS virtual key code ↔ KeyCode mapping.
//!
//! Key codes come from Carbon's Events.h / HIToolbox (unchanged in modern macOS).
//! Mirror of rs/adapters/linux/src/key_map.rs for the macOS side.

use capslockx_core::KeyCode;

/// Convert a macOS virtual key code (`CGKeyCode`, u16) to the platform-agnostic [`KeyCode`].
pub fn cg_keycode_to_keycode(code: u16) -> KeyCode {
    match code {
        // ── Trigger keys ──────────────────────────────────────────────────────
        0x39 => KeyCode::CapsLock,   // kVK_CapsLock
        0x31 => KeyCode::Space,      // kVK_Space

        // ── Modifiers ─────────────────────────────────────────────────────────
        0x38 => KeyCode::LShift,     // kVK_Shift
        0x3C => KeyCode::RShift,     // kVK_RightShift
        0x3B => KeyCode::LCtrl,      // kVK_Control
        0x3E => KeyCode::RCtrl,      // kVK_RightControl
        0x3A => KeyCode::LAlt,       // kVK_Option
        0x3D => KeyCode::RAlt,       // kVK_RightOption
        0x37 => KeyCode::LWin,       // kVK_Command
        0x36 => KeyCode::RWin,       // kVK_RightCommand

        // ── Navigation / editing ──────────────────────────────────────────────
        0x24 => KeyCode::Enter,      // kVK_Return
        0x30 => KeyCode::Tab,        // kVK_Tab
        0x75 => KeyCode::Delete,     // kVK_ForwardDelete
        0x33 => KeyCode::Backspace,  // kVK_Delete (backspace on Mac)
        0x7B => KeyCode::Left,       // kVK_LeftArrow
        0x7E => KeyCode::Up,         // kVK_UpArrow
        0x7C => KeyCode::Right,      // kVK_RightArrow
        0x7D => KeyCode::Down,       // kVK_DownArrow
        0x74 => KeyCode::PageUp,     // kVK_PageUp
        0x79 => KeyCode::PageDown,   // kVK_PageDown
        0x73 => KeyCode::Home,       // kVK_Home
        0x77 => KeyCode::End,        // kVK_End
        0x35 => KeyCode::Escape,     // kVK_Escape

        // ── Alpha (A–Z) ───────────────────────────────────────────────────────
        0x00 => KeyCode::A,  0x0B => KeyCode::B,  0x08 => KeyCode::C,
        0x02 => KeyCode::D,  0x0E => KeyCode::E,  0x03 => KeyCode::F,
        0x05 => KeyCode::G,  0x04 => KeyCode::H,  0x22 => KeyCode::I,
        0x26 => KeyCode::J,  0x28 => KeyCode::K,  0x25 => KeyCode::L,
        0x2E => KeyCode::M,  0x2D => KeyCode::N,  0x1F => KeyCode::O,
        0x23 => KeyCode::P,  0x0C => KeyCode::Q,  0x0F => KeyCode::R,
        0x01 => KeyCode::S,  0x11 => KeyCode::T,  0x20 => KeyCode::U,
        0x09 => KeyCode::V,  0x0D => KeyCode::W,  0x07 => KeyCode::X,
        0x10 => KeyCode::Y,  0x06 => KeyCode::Z,

        // ── Digits (0–9) ──────────────────────────────────────────────────────
        0x1D => KeyCode::D0,  0x12 => KeyCode::D1,  0x13 => KeyCode::D2,
        0x14 => KeyCode::D3,  0x15 => KeyCode::D4,  0x17 => KeyCode::D5,
        0x16 => KeyCode::D6,  0x1A => KeyCode::D7,  0x1C => KeyCode::D8,
        0x19 => KeyCode::D9,

        // ── Punctuation ───────────────────────────────────────────────────────
        0x21 => KeyCode::BracketLeft,   // kVK_ANSI_LeftBracket
        0x1E => KeyCode::BracketRight,  // kVK_ANSI_RightBracket
        0x2A => KeyCode::Backslash,     // kVK_ANSI_Backslash
        0x2F => KeyCode::Period,        // kVK_ANSI_Period
        0x2B => KeyCode::Comma,         // kVK_ANSI_Comma

        // ── Function keys ─────────────────────────────────────────────────────
        0x7A => KeyCode::F1,  0x78 => KeyCode::F2,  0x63 => KeyCode::F3,
        0x76 => KeyCode::F4,  0x60 => KeyCode::F5,  0x61 => KeyCode::F6,
        0x62 => KeyCode::F7,  0x64 => KeyCode::F8,  0x65 => KeyCode::F9,
        0x6D => KeyCode::F10, 0x67 => KeyCode::F11, 0x6F => KeyCode::F12,

        other => KeyCode::Unknown(other as u32),
    }
}

/// Convert a [`KeyCode`] to a macOS virtual key code (`CGKeyCode`).
/// Returns `None` for `KeyCode::Unknown`.
pub fn keycode_to_cg_keycode(key: KeyCode) -> Option<u16> {
    let code: u16 = match key {
        // ── Trigger keys ──────────────────────────────────────────────────────
        KeyCode::CapsLock   => 0x39,
        KeyCode::Space      => 0x31,
        KeyCode::Insert     => return None,  // no Insert key on Mac
        KeyCode::ScrollLock => return None,  // no ScrollLock on Mac

        // ── Modifiers ─────────────────────────────────────────────────────────
        KeyCode::Shift  => 0x38,   // generic → left shift
        KeyCode::LShift => 0x38,
        KeyCode::RShift => 0x3C,
        KeyCode::LCtrl  => 0x3B,
        KeyCode::RCtrl  => 0x3E,
        KeyCode::LAlt   => 0x3A,
        KeyCode::RAlt   => 0x3D,
        KeyCode::LWin   => 0x37,
        KeyCode::RWin   => 0x36,

        // ── Navigation / editing ──────────────────────────────────────────────
        KeyCode::Enter     => 0x24,
        KeyCode::Tab       => 0x30,
        KeyCode::Delete    => 0x75,
        KeyCode::Backspace => 0x33,
        KeyCode::Left      => 0x7B,
        KeyCode::Up        => 0x7E,
        KeyCode::Right     => 0x7C,
        KeyCode::Down      => 0x7D,
        KeyCode::PageUp    => 0x74,
        KeyCode::PageDown  => 0x79,
        KeyCode::Home      => 0x73,
        KeyCode::End       => 0x77,
        KeyCode::Escape    => 0x35,

        // ── Alpha (A–Z) ───────────────────────────────────────────────────────
        KeyCode::A => 0x00,  KeyCode::B => 0x0B,  KeyCode::C => 0x08,
        KeyCode::D => 0x02,  KeyCode::E => 0x0E,  KeyCode::F => 0x03,
        KeyCode::G => 0x05,  KeyCode::H => 0x04,  KeyCode::I => 0x22,
        KeyCode::J => 0x26,  KeyCode::K => 0x28,  KeyCode::L => 0x25,
        KeyCode::M => 0x2E,  KeyCode::N => 0x2D,  KeyCode::O => 0x1F,
        KeyCode::P => 0x23,  KeyCode::Q => 0x0C,  KeyCode::R => 0x0F,
        KeyCode::S => 0x01,  KeyCode::T => 0x11,  KeyCode::U => 0x20,
        KeyCode::V => 0x09,  KeyCode::W => 0x0D,  KeyCode::X => 0x07,
        KeyCode::Y => 0x10,  KeyCode::Z => 0x06,

        // ── Digits (0–9) ──────────────────────────────────────────────────────
        KeyCode::D0 => 0x1D,  KeyCode::D1 => 0x12,  KeyCode::D2 => 0x13,
        KeyCode::D3 => 0x14,  KeyCode::D4 => 0x15,  KeyCode::D5 => 0x17,
        KeyCode::D6 => 0x16,  KeyCode::D7 => 0x1A,  KeyCode::D8 => 0x1C,
        KeyCode::D9 => 0x19,

        // ── Punctuation ───────────────────────────────────────────────────────
        KeyCode::BracketLeft  => 0x21,
        KeyCode::BracketRight => 0x1E,
        KeyCode::Backslash    => 0x2A,
        KeyCode::Period       => 0x2F,
        KeyCode::Comma        => 0x2B,

        // ── Function keys ─────────────────────────────────────────────────────
        KeyCode::F1  => 0x7A,  KeyCode::F2  => 0x78,  KeyCode::F3  => 0x63,
        KeyCode::F4  => 0x76,  KeyCode::F5  => 0x60,  KeyCode::F6  => 0x61,
        KeyCode::F7  => 0x62,  KeyCode::F8  => 0x64,  KeyCode::F9  => 0x65,
        KeyCode::F10 => 0x6D,  KeyCode::F11 => 0x67,  KeyCode::F12 => 0x6F,

        // ── Media / volume (use NX system-defined keys, not CGEvent) ─────────
        KeyCode::MediaPlay  => return None,
        KeyCode::MediaPrev  => return None,
        KeyCode::MediaNext  => return None,
        KeyCode::MediaStop  => return None,
        KeyCode::VolumeUp   => return None,
        KeyCode::VolumeDown => return None,
        KeyCode::VolumeMute => return None,

        KeyCode::Unknown(_) => return None,
    };
    Some(code)
}
