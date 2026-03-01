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
    Shift,    // generic shift (for adapters that don't distinguish sides)
    LShift, RShift,
    LCtrl,  RCtrl,
    LAlt,   RAlt,
    LWin,   RWin,

    // ── Navigation / editing ──────────────────────────────────────────────────
    Enter, Tab, Delete, Backspace,
    Left, Up, Right, Down,
    PageUp, PageDown, Home, End,

    // ── Alpha (A–Z) ───────────────────────────────────────────────────────────
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // ── Digits (0–9) ────────────────────────────────────────────────────────
    D0, D1, D2, D3, D4, D5, D6, D7, D8, D9,

    // ── Punctuation / OEM ───────────────────────────────────────────────────
    Escape,
    BracketLeft,   // [
    BracketRight,  // ]
    Backslash,     // \ (OEM_5)
    Period,        // . (OEM_PERIOD)

    // ── Function ──────────────────────────────────────────────────────────────
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // ── Media ─────────────────────────────────────────────────────────────────
    MediaPlay, MediaPrev, MediaNext, MediaStop,
    VolumeUp, VolumeDown, VolumeMute,

    // ── Fallback ──────────────────────────────────────────────────────────────
    Unknown(u32),
}

impl KeyCode {
    /// Returns true if this is a modifier key.
    pub fn is_modifier(self) -> bool {
        matches!(
            self,
            KeyCode::Shift
                | KeyCode::LShift | KeyCode::RShift
                | KeyCode::LCtrl  | KeyCode::RCtrl
                | KeyCode::LAlt   | KeyCode::RAlt
                | KeyCode::LWin   | KeyCode::RWin
        )
    }
}

/// Snapshot of which modifier keys are currently held.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl:  bool,
    pub alt:   bool,
    pub win:   bool,
}

impl Modifiers {
    pub fn from_held(held: &std::collections::HashSet<KeyCode>) -> Self {
        Self {
            shift: held.contains(&KeyCode::Shift)
                || held.contains(&KeyCode::LShift)
                || held.contains(&KeyCode::RShift),
            ctrl: held.contains(&KeyCode::LCtrl) || held.contains(&KeyCode::RCtrl),
            alt:  held.contains(&KeyCode::LAlt)  || held.contains(&KeyCode::RAlt),
            win:  held.contains(&KeyCode::LWin)  || held.contains(&KeyCode::RWin),
        }
    }
}
