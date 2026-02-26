/// CLX-WindowManager – window cycling, tiling, close/kill, transparency.
///
/// This module only dispatches to the platform's window-management methods.
/// All actual Win32 / OS calls live in the adapter's `output.rs`.
use std::sync::Arc;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::{ArrangeMode, Platform};

pub struct WindowManagerModule {
    platform: Arc<dyn Platform>,
}

impl WindowManagerModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self { platform }
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        match key {
            KeyCode::Z => {
                if mods.shift { self.platform.cycle_windows(-1) }
                else          { self.platform.cycle_windows(1)  }
                true
            }
            KeyCode::X => {
                if mods.ctrl && mods.alt { self.platform.kill_window() }
                else if mods.shift       { self.platform.close_window() }
                else                     { self.platform.close_tab() }
                true
            }
            KeyCode::C => {
                if mods.shift {
                    self.platform.arrange_windows(ArrangeMode::SideBySide)
                } else {
                    self.platform.arrange_windows(ArrangeMode::Stacked)
                }
                true
            }
            KeyCode::V => {
                if mods.shift { self.platform.toggle_window_topmost() }
                else          { self.platform.set_window_transparent(100) }
                true
            }
            _ => false,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key == KeyCode::V {
            self.platform.restore_window();
            return true;
        }
        false
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::Z | KeyCode::X | KeyCode::C | KeyCode::V)
    }
}
