//! RDP / VM escape hatch — double-tap LCtrl+LAlt+LShift to send the focused
//! window to the back.
//!
//! A full-screen Remote Desktop (`mstsc`) or VM window swallows every
//! keystroke: Alt+Tab, the Win key and friends all travel to the *remote*
//! machine, so there is no keyboard route back to the local desktop. Modifier
//! keys pressed on their own, however, do nothing on the remote while the
//! local low-level hook still sees them — which makes a modifier-only gesture
//! the one safe escape hatch. Ported from the AHK original
//! (`Modules/CLX-WindowManager.ahk`, `CtrlShiftAlt弹起`).
//!
//! # Shape
//!
//! Hold all three of LCtrl / LAlt / LShift, release, repeat within
//! [`DOUBLE_TAP_WINDOW`]. This is the *look-back* form of double-tap (see
//! `lab/double-tap/`): the first release has no side effect at all, so nothing
//! has to be delayed or taken back, and no timer thread is involved — just an
//! `Instant` compared on the second release.
//!
//! Three guards keep it from firing by accident:
//! - all three modifiers must be down together before the first release arms it
//! - any non-modifier key pressed inside the chord cancels it (so
//!   Ctrl+Alt+Shift+X shortcuts are never swallowed), matching AHK's
//!   `A_PriorKey` check
//! - only the *left*-hand modifiers count, as in AHK's `<^<!<+`
//!
//! Nothing is ever suppressed: the modifiers reach the focused application
//! untouched, exactly like AHK's `~` prefix.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::key_code::KeyCode;

/// Maximum gap between the two chord releases. AHK used 200 ms and it is a
/// deliberate gesture, so the window stays tight — a slow double-tap is just
/// two harmless modifier presses.
pub const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(200);

/// The escape chord. Left-hand modifiers only (AHK: `<^<!<+`); the right-hand
/// ones stay free for ordinary shortcuts.
const CHORD: [KeyCode; 3] = [KeyCode::LCtrl, KeyCode::LAlt, KeyCode::LShift];

fn in_chord(key: KeyCode) -> bool {
    CHORD.contains(&key)
}

#[derive(Default)]
pub struct RdpEscape {
    /// All three modifiers have been held together and no other key has been
    /// pressed since — the next release counts as a tap.
    armed: AtomicBool,
    /// When the previous chord release happened, if it is still a candidate
    /// for the first half of a double-tap.
    last_release: Mutex<Option<Instant>>,
}

impl RdpEscape {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one key event. `held` is the engine's held-key set *after* this
    /// event was applied, and `prior` the last key pressed before it.
    ///
    /// Returns `true` when the double-tap completed; the caller performs the
    /// action. The event itself is never consumed.
    pub fn on_key_event(
        &self,
        code: KeyCode,
        pressed: bool,
        is_repeat: bool,
        prior: KeyCode,
        held: &HashSet<KeyCode>,
    ) -> bool {
        if pressed {
            if is_repeat {
                return false;
            }
            if in_chord(code) {
                // Arm once the third modifier joins the other two. Arming on
                // every chord key-down (not just a fixed order) means the
                // gesture works however the fingers land.
                if CHORD.iter().all(|k| held.contains(k)) {
                    self.armed.store(true, Ordering::Relaxed);
                }
            } else {
                // A real key inside the chord: this was a shortcut, not the
                // gesture. Disarm, and drop the pending first tap so the
                // shortcut's own modifier releases cannot complete one.
                self.armed.store(false, Ordering::Relaxed);
                *self.last_release.lock().unwrap() = None;
            }
            return false;
        }

        if !in_chord(code) || !self.armed.swap(false, Ordering::Relaxed) {
            return false;
        }
        // AHK's `A_PriorKey` guard: the last key pressed must itself be part
        // of the chord. Belt-and-braces with the key-down branch above.
        if !in_chord(prior) {
            *self.last_release.lock().unwrap() = None;
            return false;
        }

        let mut last = self.last_release.lock().unwrap();
        match *last {
            Some(t) if t.elapsed() < DOUBLE_TAP_WINDOW => {
                // Clear rather than roll forward, so a third tap needs a
                // fresh pair and one gesture never fires twice.
                *last = None;
                true
            }
            _ => {
                *last = Some(Instant::now());
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the detector the way the engine does: maintain the held set,
    /// track the prior key, and report whether the escape fired.
    struct Harness {
        esc: RdpEscape,
        held: HashSet<KeyCode>,
        prior: KeyCode,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                esc: RdpEscape::new(),
                held: HashSet::new(),
                prior: KeyCode::Unknown(0),
            }
        }

        fn key(&mut self, code: KeyCode, pressed: bool) -> bool {
            let is_repeat = if pressed {
                !self.held.insert(code)
            } else {
                self.held.remove(&code);
                false
            };
            let prior = self.prior;
            if pressed && !is_repeat {
                self.prior = code;
            }
            self.esc
                .on_key_event(code, pressed, is_repeat, prior, &self.held)
        }

        /// One full press-and-release of all three modifiers.
        fn tap_chord(&mut self) -> bool {
            for k in CHORD {
                self.key(k, true);
            }
            let mut fired = false;
            for k in CHORD {
                fired |= self.key(k, false);
            }
            fired
        }
    }

    #[test]
    fn double_tap_within_window_fires_once() {
        let mut h = Harness::new();
        assert!(!h.tap_chord());
        assert!(h.tap_chord());
    }

    #[test]
    fn single_tap_does_nothing() {
        let mut h = Harness::new();
        assert!(!h.tap_chord());
    }

    #[test]
    fn slow_second_tap_does_not_fire() {
        let mut h = Harness::new();
        assert!(!h.tap_chord());
        std::thread::sleep(DOUBLE_TAP_WINDOW + Duration::from_millis(60));
        assert!(!h.tap_chord());
    }

    #[test]
    fn triple_tap_fires_only_once() {
        let mut h = Harness::new();
        assert!(!h.tap_chord());
        assert!(h.tap_chord());
        assert!(!h.tap_chord());
    }

    #[test]
    fn key_pressed_inside_the_chord_cancels() {
        let mut h = Harness::new();
        // Ctrl+Alt+Shift+X — a real shortcut, not the gesture.
        for k in CHORD {
            h.key(k, true);
        }
        h.key(KeyCode::X, true);
        h.key(KeyCode::X, false);
        for k in CHORD {
            assert!(!h.key(k, false));
        }
        // And the aborted chord must not act as the first half of a pair.
        assert!(!h.tap_chord());
    }

    #[test]
    fn partial_chord_never_arms() {
        let mut h = Harness::new();
        for _ in 0..2 {
            h.key(KeyCode::LCtrl, true);
            h.key(KeyCode::LShift, true);
            h.key(KeyCode::LShift, false);
            assert!(!h.key(KeyCode::LCtrl, false));
        }
    }

    #[test]
    fn right_hand_modifiers_do_not_count() {
        let mut h = Harness::new();
        for _ in 0..2 {
            for k in [KeyCode::RCtrl, KeyCode::RAlt, KeyCode::RShift] {
                h.key(k, true);
            }
            for k in [KeyCode::RCtrl, KeyCode::RAlt, KeyCode::RShift] {
                assert!(!h.key(k, false));
            }
        }
    }

    #[test]
    fn auto_repeat_of_held_modifiers_is_ignored() {
        let mut h = Harness::new();
        assert!(!h.tap_chord());
        for k in CHORD {
            h.key(k, true);
            h.key(k, true); // auto-repeat key-down
        }
        let mut fired = false;
        for k in CHORD {
            fired |= h.key(k, false);
        }
        assert!(fired);
    }
}
