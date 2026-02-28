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
