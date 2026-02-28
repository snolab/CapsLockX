/// Windows VK code ↔ KeyCode mapping.
use capslockx_core::KeyCode;

pub fn vk_to_keycode(vk: u32) -> KeyCode {
    match vk {
        0x14 => KeyCode::CapsLock,
        0x20 => KeyCode::Space,
        0x2D => KeyCode::Insert,
        0x91 => KeyCode::ScrollLock,

        0x10 => KeyCode::Shift,
        0xA0 => KeyCode::LShift,
        0xA1 => KeyCode::RShift,
        0xA2 => KeyCode::LCtrl,
        0xA3 => KeyCode::RCtrl,
        0xA4 => KeyCode::LAlt,
        0xA5 => KeyCode::RAlt,
        0x5B => KeyCode::LWin,
        0x5C => KeyCode::RWin,

        0x0D => KeyCode::Enter,
        0x09 => KeyCode::Tab,
        0x2E => KeyCode::Delete,
        0x08 => KeyCode::Backspace,
        0x25 => KeyCode::Left,
        0x26 => KeyCode::Up,
        0x27 => KeyCode::Right,
        0x28 => KeyCode::Down,
        0x21 => KeyCode::PageUp,
        0x22 => KeyCode::PageDown,
        0x24 => KeyCode::Home,
        0x23 => KeyCode::End,

        0x30 => KeyCode::D0, 0x31 => KeyCode::D1, 0x32 => KeyCode::D2,
        0x33 => KeyCode::D3, 0x34 => KeyCode::D4, 0x35 => KeyCode::D5,
        0x36 => KeyCode::D6, 0x37 => KeyCode::D7, 0x38 => KeyCode::D8,
        0x39 => KeyCode::D9,

        0x41 => KeyCode::A,  0x42 => KeyCode::B,  0x43 => KeyCode::C,
        0x44 => KeyCode::D,  0x45 => KeyCode::E,  0x46 => KeyCode::F,
        0x47 => KeyCode::G,  0x48 => KeyCode::H,  0x49 => KeyCode::I,
        0x4A => KeyCode::J,  0x4B => KeyCode::K,  0x4C => KeyCode::L,
        0x4D => KeyCode::M,  0x4E => KeyCode::N,  0x4F => KeyCode::O,
        0x50 => KeyCode::P,  0x51 => KeyCode::Q,  0x52 => KeyCode::R,
        0x53 => KeyCode::S,  0x54 => KeyCode::T,  0x55 => KeyCode::U,
        0x56 => KeyCode::V,  0x57 => KeyCode::W,  0x58 => KeyCode::X,
        0x59 => KeyCode::Y,  0x5A => KeyCode::Z,

        0x70 => KeyCode::F1,  0x71 => KeyCode::F2,  0x72 => KeyCode::F3,
        0x73 => KeyCode::F4,  0x74 => KeyCode::F5,  0x75 => KeyCode::F6,
        0x76 => KeyCode::F7,  0x77 => KeyCode::F8,  0x78 => KeyCode::F9,
        0x79 => KeyCode::F10, 0x7A => KeyCode::F11, 0x7B => KeyCode::F12,

        0xB3 => KeyCode::MediaPlay,
        0xB1 => KeyCode::MediaPrev,
        0xB0 => KeyCode::MediaNext,
        0xB2 => KeyCode::MediaStop,
        0xAF => KeyCode::VolumeUp,
        0xAE => KeyCode::VolumeDown,
        0xAD => KeyCode::VolumeMute,

        other => KeyCode::Unknown(other),
    }
}

pub fn keycode_to_vk(key: KeyCode) -> u16 {
    match key {
        KeyCode::CapsLock   => 0x14,
        KeyCode::Space      => 0x20,
        KeyCode::Insert     => 0x2D,
        KeyCode::ScrollLock => 0x91,

        KeyCode::Shift  => 0x10,
        KeyCode::LShift => 0xA0,
        KeyCode::RShift => 0xA1,
        KeyCode::LCtrl  => 0xA2,
        KeyCode::RCtrl  => 0xA3,
        KeyCode::LAlt   => 0xA4,
        KeyCode::RAlt   => 0xA5,
        KeyCode::LWin   => 0x5B,
        KeyCode::RWin   => 0x5C,

        KeyCode::Enter     => 0x0D,
        KeyCode::Tab       => 0x09,
        KeyCode::Delete    => 0x2E,
        KeyCode::Backspace => 0x08,
        KeyCode::Left      => 0x25,
        KeyCode::Up        => 0x26,
        KeyCode::Right     => 0x27,
        KeyCode::Down      => 0x28,
        KeyCode::PageUp    => 0x21,
        KeyCode::PageDown  => 0x22,
        KeyCode::Home      => 0x24,
        KeyCode::End       => 0x23,

        KeyCode::D0 => 0x30, KeyCode::D1 => 0x31, KeyCode::D2 => 0x32,
        KeyCode::D3 => 0x33, KeyCode::D4 => 0x34, KeyCode::D5 => 0x35,
        KeyCode::D6 => 0x36, KeyCode::D7 => 0x37, KeyCode::D8 => 0x38,
        KeyCode::D9 => 0x39,

        KeyCode::A => 0x41, KeyCode::B => 0x42, KeyCode::C => 0x43,
        KeyCode::D => 0x44, KeyCode::E => 0x45, KeyCode::F => 0x46,
        KeyCode::G => 0x47, KeyCode::H => 0x48, KeyCode::I => 0x49,
        KeyCode::J => 0x4A, KeyCode::K => 0x4B, KeyCode::L => 0x4C,
        KeyCode::M => 0x4D, KeyCode::N => 0x4E, KeyCode::O => 0x4F,
        KeyCode::P => 0x50, KeyCode::Q => 0x51, KeyCode::R => 0x52,
        KeyCode::S => 0x53, KeyCode::T => 0x54, KeyCode::U => 0x55,
        KeyCode::V => 0x56, KeyCode::W => 0x57, KeyCode::X => 0x58,
        KeyCode::Y => 0x59, KeyCode::Z => 0x5A,

        KeyCode::F1  => 0x70, KeyCode::F2  => 0x71, KeyCode::F3  => 0x72,
        KeyCode::F4  => 0x73, KeyCode::F5  => 0x74, KeyCode::F6  => 0x75,
        KeyCode::F7  => 0x76, KeyCode::F8  => 0x77, KeyCode::F9  => 0x78,
        KeyCode::F10 => 0x79, KeyCode::F11 => 0x7A, KeyCode::F12 => 0x7B,

        KeyCode::MediaPlay  => 0xB3,
        KeyCode::MediaPrev  => 0xB1,
        KeyCode::MediaNext  => 0xB0,
        KeyCode::MediaStop  => 0xB2,
        KeyCode::VolumeUp   => 0xAF,
        KeyCode::VolumeDown => 0xAE,
        KeyCode::VolumeMute => 0xAD,

        KeyCode::Unknown(v) => v as u16,
    }
}
