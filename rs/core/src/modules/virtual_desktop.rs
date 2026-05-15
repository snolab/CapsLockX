/// CLX-VirtualDesktop – switch / move windows between virtual desktops.
///
/// CLX + 1–9  → switch to desktop 1–9
/// CLX + 0    → switch to desktop 10
/// CLX + Shift + 1–9/0 → move active window to that desktop
use std::sync::Arc;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

pub struct VirtualDesktopModule {
    platform: Arc<dyn Platform>,
}

impl VirtualDesktopModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self { platform }
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        let idx = match key {
            KeyCode::D1 => 1,
            KeyCode::D2 => 2,
            KeyCode::D3 => 3,
            KeyCode::D4 => 4,
            KeyCode::D5 => 5,
            KeyCode::D6 => 6,
            KeyCode::D7 => 7,
            KeyCode::D8 => 8,
            KeyCode::D9 => 9,
            KeyCode::D0 => 10,
            _ => return false,
        };
        if mods.shift {
            self.platform.move_window_to_desktop(idx);
        } else {
            self.platform.switch_to_desktop(idx);
        }
        true
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key,
            KeyCode::D0 | KeyCode::D1 | KeyCode::D2 | KeyCode::D3 | KeyCode::D4 |
            KeyCode::D5 | KeyCode::D6 | KeyCode::D7 | KeyCode::D8 | KeyCode::D9
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    fn setup() -> (Arc<MockPlatform>, VirtualDesktopModule) {
        let mock = Arc::new(MockPlatform::new());
        let module = VirtualDesktopModule::new(mock.clone());
        (mock, module)
    }

    fn no_mods() -> Modifiers { Modifiers::default() }
    fn shift_mods() -> Modifiers {
        let mut m = Modifiers::default();
        m.shift = true;
        m
    }

    #[test]
    fn digits_one_through_nine_switch_to_matching_desktop() {
        let cases = [
            (KeyCode::D1, 1u32), (KeyCode::D2, 2), (KeyCode::D3, 3),
            (KeyCode::D4, 4), (KeyCode::D5, 5), (KeyCode::D6, 6),
            (KeyCode::D7, 7), (KeyCode::D8, 8), (KeyCode::D9, 9),
        ];
        for (key, idx) in cases {
            let (mock, module) = setup();
            assert!(module.on_key_down(key, &no_mods()));
            assert_eq!(mock.calls(), vec![Call::SwitchToDesktop(idx)]);
        }
    }

    #[test]
    fn digit_zero_switches_to_desktop_ten() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::D0, &no_mods()));
        assert_eq!(mock.calls(), vec![Call::SwitchToDesktop(10)]);
    }

    #[test]
    fn shift_modifier_moves_window_instead_of_switching() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::D3, &shift_mods()));
        assert_eq!(mock.calls(), vec![Call::MoveWindowToDesktop(3)]);
    }

    #[test]
    fn shift_with_zero_moves_window_to_desktop_ten() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::D0, &shift_mods()));
        assert_eq!(mock.calls(), vec![Call::MoveWindowToDesktop(10)]);
    }

    #[test]
    fn unmapped_key_returns_false_and_emits_nothing() {
        let (mock, module) = setup();
        assert!(!module.on_key_down(KeyCode::A, &no_mods()));
        assert!(!module.on_key_down(KeyCode::F1, &shift_mods()));
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn is_mapped_key_recognises_all_digits_and_rejects_others() {
        let (_, module) = setup();
        for k in [KeyCode::D0, KeyCode::D1, KeyCode::D2, KeyCode::D3, KeyCode::D4,
                  KeyCode::D5, KeyCode::D6, KeyCode::D7, KeyCode::D8, KeyCode::D9] {
            assert!(module.is_mapped_key(k));
        }
        assert!(!module.is_mapped_key(KeyCode::A));
        assert!(!module.is_mapped_key(KeyCode::F1));
    }
}
