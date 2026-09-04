use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;
use crate::state::ClxState;
/// CLX-FnRow – the number row becomes F1–F12 while BOTH triggers are held.
///
/// In the gesture notation (see `lab/chord-fn-row/#notation`):
///
/// ```text
///   (capslock+space)[ 1 ]        -> F1        ... [ = ] -> F12
///   (capslock+space+shift)[ 1 ]  -> Shift+F1  (modifiers decorate for free)
///   (capslock+space)]            -> toggle the CLX lock, unchanged
///   space[ 3 ]                   -> virtual desktop 3, unchanged
/// ```
///
/// The layer is gated on the *chord* — both CapsLock and Space physically
/// down — which is a different gesture from a single trigger, so the virtual
/// desktop bindings on the same keys are untouched.
///
/// Modifiers need no handling here: the engine never injects or suppresses
/// Shift/Ctrl/Alt/Win, so a physically-held modifier decorates the injected
/// F-key by itself. `(capslock+space+alt)[ 4 ]` really is Alt+F4.
use std::sync::Arc;

pub struct FnRowModule {
    platform: Arc<dyn Platform>,
    state: Arc<ClxState>,
}

impl FnRowModule {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        Self { platform, state }
    }

    /// Map a number-row key to its function key, layout permitting.
    ///
    /// `Equal` is the US position for F12; JIS boards have `^` (VK_OEM_7)
    /// there instead, which the Windows adapter also maps to `Equal`.
    fn f_key(key: KeyCode) -> Option<KeyCode> {
        Some(match key {
            KeyCode::D1 => KeyCode::F1,
            KeyCode::D2 => KeyCode::F2,
            KeyCode::D3 => KeyCode::F3,
            KeyCode::D4 => KeyCode::F4,
            KeyCode::D5 => KeyCode::F5,
            KeyCode::D6 => KeyCode::F6,
            KeyCode::D7 => KeyCode::F7,
            KeyCode::D8 => KeyCode::F8,
            KeyCode::D9 => KeyCode::F9,
            KeyCode::D0 => KeyCode::F10,
            KeyCode::Minus => KeyCode::F11,
            KeyCode::Equal => KeyCode::F12,
            _ => return None,
        })
    }

    pub fn on_key_down(&self, key: KeyCode, _mods: &Modifiers) -> bool {
        if !self.state.is_chord_active() {
            return false;
        }
        match Self::f_key(key) {
            Some(f) => {
                self.platform.key_tap(f);
                true
            }
            None => false,
        }
    }

    /// Chord-aware on purpose. Claiming `Minus`/`Equal` unconditionally would
    /// make the engine swallow the auto-repeat of a bare `clx[ - ]`, which
    /// should still type a stream of `-`.
    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        self.state.is_chord_active() && Self::f_key(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ClxConfig;
    use crate::test_platform::{Call, MockPlatform};

    fn setup(chord: bool) -> (Arc<MockPlatform>, FnRowModule) {
        let mock = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        state.set_chord_active(chord);
        (mock.clone(), FnRowModule::new(mock, state))
    }

    fn taps(p: &MockPlatform, k: KeyCode) -> usize {
        p.count(|c| matches!(c, Call::KeyDown(x) if *x == k))
    }

    #[test]
    fn number_row_maps_to_function_keys_while_chorded() {
        let cases = [
            (KeyCode::D1, KeyCode::F1),
            (KeyCode::D9, KeyCode::F9),
            (KeyCode::D0, KeyCode::F10),
            (KeyCode::Minus, KeyCode::F11),
            (KeyCode::Equal, KeyCode::F12),
        ];
        for (from, to) in cases {
            let (mock, m) = setup(true);
            assert!(m.on_key_down(from, &Modifiers::default()), "{:?}", from);
            assert!(taps(&mock, to) >= 1, "{:?} should emit {:?}", from, to);
        }
    }

    #[test]
    fn does_nothing_without_the_chord() {
        let (mock, m) = setup(false);
        assert!(!m.on_key_down(KeyCode::D1, &Modifiers::default()));
        assert!(mock.calls().is_empty());
        assert!(!m.is_mapped_key(KeyCode::D1));
        assert!(!m.is_mapped_key(KeyCode::Minus));
    }

    #[test]
    fn unrelated_keys_are_not_claimed() {
        let (_, m) = setup(true);
        assert!(!m.on_key_down(KeyCode::H, &Modifiers::default()));
        assert!(!m.is_mapped_key(KeyCode::H));
    }
}
