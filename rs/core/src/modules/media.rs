/// CLX-MediaKeys – F5–F11 → media / volume keys.
/// CLX-Lianki   – [ ] \ → prev / next / play-pause.
use std::sync::Arc;
use crate::key_code::KeyCode;
use crate::platform::Platform;

pub struct MediaModule {
    platform: Arc<dyn Platform>,
}

impl MediaModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self { platform }
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        match key {
            // F-key media controls
            KeyCode::F5  => { self.platform.key_tap_extended(KeyCode::MediaPlay);   true }
            KeyCode::F6  => { self.platform.key_tap_extended(KeyCode::MediaPrev);   true }
            KeyCode::F7  => { self.platform.key_tap_extended(KeyCode::MediaNext);   true }
            KeyCode::F8  => { self.platform.key_tap_extended(KeyCode::MediaStop);   true }
            KeyCode::F9  => { self.platform.key_tap_extended(KeyCode::VolumeUp);    true }
            KeyCode::F10 => { self.platform.key_tap_extended(KeyCode::VolumeDown);  true }
            KeyCode::F11 => { self.platform.key_tap_extended(KeyCode::VolumeMute);  true }
            // Lianki shortcuts
            KeyCode::BracketLeft  => { self.platform.key_tap_extended(KeyCode::MediaPrev); true }
            KeyCode::BracketRight => { self.platform.key_tap_extended(KeyCode::MediaNext); true }
            KeyCode::Backslash    => { self.platform.key_tap_extended(KeyCode::MediaPlay); true }
            _ => false,
        }
    }

    pub fn on_key_up(&self, _key: KeyCode) -> bool { false }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::F5 | KeyCode::F6 | KeyCode::F7 | KeyCode::F8
                    | KeyCode::F9 | KeyCode::F10 | KeyCode::F11
                    | KeyCode::BracketLeft | KeyCode::BracketRight | KeyCode::Backslash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    fn setup() -> (Arc<MockPlatform>, MediaModule) {
        let mock = Arc::new(MockPlatform::new());
        let module = MediaModule::new(mock.clone());
        (mock, module)
    }

    #[test]
    fn f_keys_dispatch_to_media_controls() {
        let cases = [
            (KeyCode::F5,  KeyCode::MediaPlay),
            (KeyCode::F6,  KeyCode::MediaPrev),
            (KeyCode::F7,  KeyCode::MediaNext),
            (KeyCode::F8,  KeyCode::MediaStop),
            (KeyCode::F9,  KeyCode::VolumeUp),
            (KeyCode::F10, KeyCode::VolumeDown),
            (KeyCode::F11, KeyCode::VolumeMute),
        ];
        for (input, expected) in cases {
            let (mock, module) = setup();
            assert!(module.on_key_down(input));
            assert_eq!(mock.calls(), vec![Call::KeyTapExtended(expected)]);
        }
    }

    #[test]
    fn lianki_brackets_dispatch_to_media_controls() {
        let cases = [
            (KeyCode::BracketLeft,  KeyCode::MediaPrev),
            (KeyCode::BracketRight, KeyCode::MediaNext),
            (KeyCode::Backslash,    KeyCode::MediaPlay),
        ];
        for (input, expected) in cases {
            let (mock, module) = setup();
            assert!(module.on_key_down(input));
            assert_eq!(mock.calls(), vec![Call::KeyTapExtended(expected)]);
        }
    }

    #[test]
    fn unmapped_key_returns_false_and_emits_nothing() {
        let (mock, module) = setup();
        assert!(!module.on_key_down(KeyCode::A));
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn key_up_always_returns_false_without_dispatch() {
        let (mock, module) = setup();
        assert!(!module.on_key_up(KeyCode::F5));
        assert!(!module.on_key_up(KeyCode::A));
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn is_mapped_key_matches_all_handled_keys() {
        let (_, module) = setup();
        for k in [KeyCode::F5, KeyCode::F6, KeyCode::F7, KeyCode::F8,
                  KeyCode::F9, KeyCode::F10, KeyCode::F11,
                  KeyCode::BracketLeft, KeyCode::BracketRight, KeyCode::Backslash] {
            assert!(module.is_mapped_key(k));
        }
        assert!(!module.is_mapped_key(KeyCode::A));
        assert!(!module.is_mapped_key(KeyCode::F4));
    }
}
