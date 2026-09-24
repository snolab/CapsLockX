/// Platform-agnostic key code enum.
///
/// Each adapter maps its native key identifiers to these variants.
/// The `Unknown(u32)` variant passes raw values through unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // ── Trigger keys ──────────────────────────────────────────────────────────
    CapsLock,
    Space,
    Insert,
    ScrollLock,

    // ── Modifiers ─────────────────────────────────────────────────────────────
    Shift, // generic shift (for adapters that don't distinguish sides)
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LWin,
    RWin,

    // ── Navigation / editing ──────────────────────────────────────────────────
    Enter,
    Tab,
    Delete,
    Backspace,
    Left,
    Up,
    Right,
    Down,
    PageUp,
    PageDown,
    Home,
    End,

    // ── Alpha (A–Z) ───────────────────────────────────────────────────────────
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // ── Digits (0–9) ────────────────────────────────────────────────────────
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,

    // ── Punctuation / OEM ───────────────────────────────────────────────────
    Escape,
    BracketLeft,  // [
    BracketRight, // ]
    Backslash,    // \ (OEM_5)
    Period,       // . (OEM_PERIOD)
    Comma,        // , (OEM_COMMA)
    Slash,        // / (OEM_2)
    Minus,        // - (OEM_MINUS)
    Equal,        // = (OEM_PLUS); JIS boards carry ^ in this position

    // ── Function ──────────────────────────────────────────────────────────────
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // ── Media ─────────────────────────────────────────────────────────────────
    MediaPlay,
    MediaPrev,
    MediaNext,
    MediaStop,
    VolumeUp,
    VolumeDown,
    VolumeMute,

    // ── Fallback ──────────────────────────────────────────────────────────────
    Unknown(u32),
}

impl KeyCode {
    /// The key a single printable character names — `'c'` is C, `'4'` is D4,
    /// `'-'` is Minus. Used by the effect language, where `k c-c` means Ctrl+C
    /// and the key half is written as the character itself.
    pub fn from_char(c: char) -> Option<KeyCode> {
        let c = c.to_ascii_lowercase();
        Some(match c {
            'a'..='z' => Self::LETTERS[(c as u8 - b'a') as usize],
            '0'..='9' => Self::DIGITS[(c as u8 - b'0') as usize],
            '-' => KeyCode::Minus,
            '=' => KeyCode::Equal,
            ',' => KeyCode::Comma,
            '.' => KeyCode::Period,
            '/' => KeyCode::Slash,
            '[' => KeyCode::BracketLeft,
            ']' => KeyCode::BracketRight,
            '\\' => KeyCode::Backslash,
            _ => return None,
        })
    }

    /// `1` → F1. Only F1–F12 exist as variants, so anything beyond is `None`
    /// rather than a silently wrong key.
    pub fn from_fn_number(n: u8) -> Option<KeyCode> {
        Some(match n {
            1 => KeyCode::F1,
            2 => KeyCode::F2,
            3 => KeyCode::F3,
            4 => KeyCode::F4,
            5 => KeyCode::F5,
            6 => KeyCode::F6,
            7 => KeyCode::F7,
            8 => KeyCode::F8,
            9 => KeyCode::F9,
            10 => KeyCode::F10,
            11 => KeyCode::F11,
            12 => KeyCode::F12,
            _ => return None,
        })
    }

    const LETTERS: [KeyCode; 26] = [
        KeyCode::A,
        KeyCode::B,
        KeyCode::C,
        KeyCode::D,
        KeyCode::E,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::I,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::M,
        KeyCode::N,
        KeyCode::O,
        KeyCode::P,
        KeyCode::Q,
        KeyCode::R,
        KeyCode::S,
        KeyCode::T,
        KeyCode::U,
        KeyCode::V,
        KeyCode::W,
        KeyCode::X,
        KeyCode::Y,
        KeyCode::Z,
    ];
    const DIGITS: [KeyCode; 10] = [
        KeyCode::D0,
        KeyCode::D1,
        KeyCode::D2,
        KeyCode::D3,
        KeyCode::D4,
        KeyCode::D5,
        KeyCode::D6,
        KeyCode::D7,
        KeyCode::D8,
        KeyCode::D9,
    ];

    /// Returns true if this is a modifier key.
    pub fn is_modifier(self) -> bool {
        matches!(
            self,
            KeyCode::Shift
                | KeyCode::LShift
                | KeyCode::RShift
                | KeyCode::LCtrl
                | KeyCode::RCtrl
                | KeyCode::LAlt
                | KeyCode::RAlt
                | KeyCode::LWin
                | KeyCode::RWin
        )
    }
}

/// Snapshot of which modifier keys are currently held.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl Modifiers {
    pub fn from_held(held: &std::collections::HashSet<KeyCode>) -> Self {
        Self {
            shift: held.contains(&KeyCode::Shift)
                || held.contains(&KeyCode::LShift)
                || held.contains(&KeyCode::RShift),
            ctrl: held.contains(&KeyCode::LCtrl) || held.contains(&KeyCode::RCtrl),
            alt: held.contains(&KeyCode::LAlt) || held.contains(&KeyCode::RAlt),
            win: held.contains(&KeyCode::LWin) || held.contains(&KeyCode::RWin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn modifiers_are_modifier_keys() {
        for k in [
            KeyCode::Shift,
            KeyCode::LShift,
            KeyCode::RShift,
            KeyCode::LCtrl,
            KeyCode::RCtrl,
            KeyCode::LAlt,
            KeyCode::RAlt,
            KeyCode::LWin,
            KeyCode::RWin,
        ] {
            assert!(k.is_modifier(), "{:?} should be modifier", k);
        }
    }

    #[test]
    fn non_modifiers_are_not_modifier_keys() {
        for k in [
            KeyCode::Space,
            KeyCode::CapsLock,
            KeyCode::Insert,
            KeyCode::ScrollLock,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Delete,
            KeyCode::Backspace,
            KeyCode::Left,
            KeyCode::Up,
            KeyCode::Right,
            KeyCode::Down,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::A,
            KeyCode::Z,
            KeyCode::D0,
            KeyCode::D9,
            KeyCode::Escape,
            KeyCode::BracketLeft,
            KeyCode::BracketRight,
            KeyCode::Backslash,
            KeyCode::Period,
            KeyCode::Comma,
            KeyCode::F1,
            KeyCode::F12,
            KeyCode::MediaPlay,
            KeyCode::MediaPrev,
            KeyCode::MediaNext,
            KeyCode::MediaStop,
            KeyCode::VolumeUp,
            KeyCode::VolumeDown,
            KeyCode::VolumeMute,
            KeyCode::Unknown(42),
        ] {
            assert!(!k.is_modifier(), "{:?} should not be modifier", k);
        }
    }

    #[test]
    fn keycode_equality_and_hash() {
        let mut set = HashSet::new();
        set.insert(KeyCode::A);
        set.insert(KeyCode::A);
        set.insert(KeyCode::Unknown(7));
        set.insert(KeyCode::Unknown(7));
        set.insert(KeyCode::Unknown(8));
        assert_eq!(set.len(), 3);
        assert_eq!(KeyCode::A, KeyCode::A);
        assert_ne!(KeyCode::A, KeyCode::B);
        assert_ne!(KeyCode::Unknown(1), KeyCode::Unknown(2));
    }

    #[test]
    fn modifiers_default_all_false() {
        let m = Modifiers::default();
        assert!(!m.shift && !m.ctrl && !m.alt && !m.win);
    }

    #[test]
    fn modifiers_from_held_empty() {
        let held = HashSet::new();
        let m = Modifiers::from_held(&held);
        assert!(!m.shift && !m.ctrl && !m.alt && !m.win);
    }

    #[test]
    fn modifiers_from_held_generic_shift() {
        let mut h = HashSet::new();
        h.insert(KeyCode::Shift);
        assert!(Modifiers::from_held(&h).shift);
    }

    #[test]
    fn modifiers_from_held_left_and_right_shift() {
        let mut h = HashSet::new();
        h.insert(KeyCode::LShift);
        assert!(Modifiers::from_held(&h).shift);
        let mut h = HashSet::new();
        h.insert(KeyCode::RShift);
        assert!(Modifiers::from_held(&h).shift);
    }

    #[test]
    fn modifiers_from_held_ctrl_alt_win() {
        let mut h = HashSet::new();
        h.insert(KeyCode::LCtrl);
        h.insert(KeyCode::RAlt);
        h.insert(KeyCode::LWin);
        let m = Modifiers::from_held(&h);
        assert!(m.ctrl && m.alt && m.win && !m.shift);
    }

    #[test]
    fn modifiers_from_held_right_variants() {
        let mut h = HashSet::new();
        h.insert(KeyCode::RCtrl);
        h.insert(KeyCode::LAlt);
        h.insert(KeyCode::RWin);
        let m = Modifiers::from_held(&h);
        assert!(m.ctrl && m.alt && m.win);
    }

    #[test]
    fn modifiers_from_held_all_set() {
        let mut h = HashSet::new();
        for k in [KeyCode::Shift, KeyCode::LCtrl, KeyCode::LAlt, KeyCode::LWin] {
            h.insert(k);
        }
        let m = Modifiers::from_held(&h);
        assert!(m.shift && m.ctrl && m.alt && m.win);
    }

    #[test]
    fn keycode_clone_copy() {
        let a = KeyCode::A;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
