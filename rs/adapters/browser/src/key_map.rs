/// Browser `KeyboardEvent.code` (physical key) ↔ `KeyCode` mapping.
use capslockx_core::KeyCode;

/// Convert a browser `event.code` string to a `KeyCode`.
pub fn code_to_keycode(code: &str) -> KeyCode {
    match code {
        // ── Trigger keys ──────────────────────────────────────────────────────
        "CapsLock"     => KeyCode::CapsLock,
        "Space"        => KeyCode::Space,
        "Insert"       => KeyCode::Insert,
        "ScrollLock"   => KeyCode::ScrollLock,

        // ── Modifiers ─────────────────────────────────────────────────────────
        "ShiftLeft"    => KeyCode::LShift,
        "ShiftRight"   => KeyCode::RShift,
        "ControlLeft"  => KeyCode::LCtrl,
        "ControlRight" => KeyCode::RCtrl,
        "AltLeft"      => KeyCode::LAlt,
        "AltRight"     => KeyCode::RAlt,
        "MetaLeft"     => KeyCode::LWin,
        "MetaRight"    => KeyCode::RWin,

        // ── Navigation / editing ──────────────────────────────────────────────
        "Enter"        => KeyCode::Enter,
        "Tab"          => KeyCode::Tab,
        "Delete"       => KeyCode::Delete,
        "Backspace"    => KeyCode::Backspace,
        "ArrowLeft"    => KeyCode::Left,
        "ArrowUp"      => KeyCode::Up,
        "ArrowRight"   => KeyCode::Right,
        "ArrowDown"    => KeyCode::Down,
        "PageUp"       => KeyCode::PageUp,
        "PageDown"     => KeyCode::PageDown,
        "Home"         => KeyCode::Home,
        "End"          => KeyCode::End,

        // ── Digits ────────────────────────────────────────────────────────────
        "Digit0" => KeyCode::D0, "Digit1" => KeyCode::D1, "Digit2" => KeyCode::D2,
        "Digit3" => KeyCode::D3, "Digit4" => KeyCode::D4, "Digit5" => KeyCode::D5,
        "Digit6" => KeyCode::D6, "Digit7" => KeyCode::D7, "Digit8" => KeyCode::D8,
        "Digit9" => KeyCode::D9,

        // ── Alpha ─────────────────────────────────────────────────────────────
        "KeyA" => KeyCode::A, "KeyB" => KeyCode::B, "KeyC" => KeyCode::C,
        "KeyD" => KeyCode::D, "KeyE" => KeyCode::E, "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G, "KeyH" => KeyCode::H, "KeyI" => KeyCode::I,
        "KeyJ" => KeyCode::J, "KeyK" => KeyCode::K, "KeyL" => KeyCode::L,
        "KeyM" => KeyCode::M, "KeyN" => KeyCode::N, "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P, "KeyQ" => KeyCode::Q, "KeyR" => KeyCode::R,
        "KeyS" => KeyCode::S, "KeyT" => KeyCode::T, "KeyU" => KeyCode::U,
        "KeyV" => KeyCode::V, "KeyW" => KeyCode::W, "KeyX" => KeyCode::X,
        "KeyY" => KeyCode::Y, "KeyZ" => KeyCode::Z,

        // ── Function ──────────────────────────────────────────────────────────
        "F1"  => KeyCode::F1,  "F2"  => KeyCode::F2,  "F3"  => KeyCode::F3,
        "F4"  => KeyCode::F4,  "F5"  => KeyCode::F5,  "F6"  => KeyCode::F6,
        "F7"  => KeyCode::F7,  "F8"  => KeyCode::F8,  "F9"  => KeyCode::F9,
        "F10" => KeyCode::F10, "F11" => KeyCode::F11, "F12" => KeyCode::F12,

        // ── Media ─────────────────────────────────────────────────────────────
        "MediaPlayPause"        => KeyCode::MediaPlay,
        "MediaTrackPrevious"    => KeyCode::MediaPrev,
        "MediaTrackNext"        => KeyCode::MediaNext,
        "MediaStop"             => KeyCode::MediaStop,
        "AudioVolumeUp"         => KeyCode::VolumeUp,
        "AudioVolumeDown"       => KeyCode::VolumeDown,
        "AudioVolumeMute"       => KeyCode::VolumeMute,

        // ── Fallback: hash the code string to a stable u32 ───────────────────
        other => {
            let mut h: u32 = 5381;
            for b in other.bytes() {
                h = h.wrapping_mul(33).wrapping_add(b as u32);
            }
            KeyCode::Unknown(h)
        }
    }
}

/// Map a `KeyCode` to `(key, code)` strings for `KeyboardEvent` dispatch.
///
/// `key`  is the logical value  (e.g. `"ArrowLeft"`, `"a"`).
/// `code` is the physical value (e.g. `"ArrowLeft"`, `"KeyA"`).
pub fn keycode_to_event_strs(key: KeyCode) -> (&'static str, &'static str) {
    match key {
        KeyCode::Enter     => ("Enter",     "Enter"),
        KeyCode::Tab       => ("Tab",       "Tab"),
        KeyCode::Delete    => ("Delete",    "Delete"),
        KeyCode::Backspace => ("Backspace", "Backspace"),
        KeyCode::Left      => ("ArrowLeft",  "ArrowLeft"),
        KeyCode::Up        => ("ArrowUp",    "ArrowUp"),
        KeyCode::Right     => ("ArrowRight", "ArrowRight"),
        KeyCode::Down      => ("ArrowDown",  "ArrowDown"),
        KeyCode::PageUp    => ("PageUp",     "PageUp"),
        KeyCode::PageDown  => ("PageDown",   "PageDown"),
        KeyCode::Home      => ("Home",       "Home"),
        KeyCode::End       => ("End",        "End"),
        KeyCode::Space     => (" ",          "Space"),
        KeyCode::CapsLock  => ("CapsLock",   "CapsLock"),
        KeyCode::Insert    => ("Insert",     "Insert"),
        KeyCode::ScrollLock => ("ScrollLock", "ScrollLock"),

        KeyCode::Shift | KeyCode::LShift => ("Shift",   "ShiftLeft"),
        KeyCode::RShift                  => ("Shift",   "ShiftRight"),
        KeyCode::LCtrl                   => ("Control", "ControlLeft"),
        KeyCode::RCtrl                   => ("Control", "ControlRight"),
        KeyCode::LAlt                    => ("Alt",     "AltLeft"),
        KeyCode::RAlt                    => ("Alt",     "AltRight"),
        KeyCode::LWin                    => ("Meta",    "MetaLeft"),
        KeyCode::RWin                    => ("Meta",    "MetaRight"),

        KeyCode::D0 => ("0", "Digit0"), KeyCode::D1 => ("1", "Digit1"),
        KeyCode::D2 => ("2", "Digit2"), KeyCode::D3 => ("3", "Digit3"),
        KeyCode::D4 => ("4", "Digit4"), KeyCode::D5 => ("5", "Digit5"),
        KeyCode::D6 => ("6", "Digit6"), KeyCode::D7 => ("7", "Digit7"),
        KeyCode::D8 => ("8", "Digit8"), KeyCode::D9 => ("9", "Digit9"),

        KeyCode::A => ("a", "KeyA"), KeyCode::B => ("b", "KeyB"),
        KeyCode::C => ("c", "KeyC"), KeyCode::D => ("d", "KeyD"),
        KeyCode::E => ("e", "KeyE"), KeyCode::F => ("f", "KeyF"),
        KeyCode::G => ("g", "KeyG"), KeyCode::H => ("h", "KeyH"),
        KeyCode::I => ("i", "KeyI"), KeyCode::J => ("j", "KeyJ"),
        KeyCode::K => ("k", "KeyK"), KeyCode::L => ("l", "KeyL"),
        KeyCode::M => ("m", "KeyM"), KeyCode::N => ("n", "KeyN"),
        KeyCode::O => ("o", "KeyO"), KeyCode::P => ("p", "KeyP"),
        KeyCode::Q => ("q", "KeyQ"), KeyCode::R => ("r", "KeyR"),
        KeyCode::S => ("s", "KeyS"), KeyCode::T => ("t", "KeyT"),
        KeyCode::U => ("u", "KeyU"), KeyCode::V => ("v", "KeyV"),
        KeyCode::W => ("w", "KeyW"), KeyCode::X => ("x", "KeyX"),
        KeyCode::Y => ("y", "KeyY"), KeyCode::Z => ("z", "KeyZ"),

        KeyCode::F1  => ("F1",  "F1"),  KeyCode::F2  => ("F2",  "F2"),
        KeyCode::F3  => ("F3",  "F3"),  KeyCode::F4  => ("F4",  "F4"),
        KeyCode::F5  => ("F5",  "F5"),  KeyCode::F6  => ("F6",  "F6"),
        KeyCode::F7  => ("F7",  "F7"),  KeyCode::F8  => ("F8",  "F8"),
        KeyCode::F9  => ("F9",  "F9"),  KeyCode::F10 => ("F10", "F10"),
        KeyCode::F11 => ("F11", "F11"), KeyCode::F12 => ("F12", "F12"),

        KeyCode::MediaPlay  => ("MediaPlayPause",     "MediaPlayPause"),
        KeyCode::MediaPrev  => ("MediaTrackPrevious", "MediaTrackPrevious"),
        KeyCode::MediaNext  => ("MediaTrackNext",     "MediaTrackNext"),
        KeyCode::MediaStop  => ("MediaStop",          "MediaStop"),
        KeyCode::VolumeUp   => ("AudioVolumeUp",      "AudioVolumeUp"),
        KeyCode::VolumeDown => ("AudioVolumeDown",    "AudioVolumeDown"),
        KeyCode::VolumeMute => ("AudioVolumeMute",    "AudioVolumeMute"),

        _ => ("Unidentified", ""),
    }
}
