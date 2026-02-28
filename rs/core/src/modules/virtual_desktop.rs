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
