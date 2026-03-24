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
            // V is owned by voice module (Space+V = voice input).
            // Window transparency removed — was conflicting with voice.
            KeyCode::Period => {
                self.platform.restart();
                true
            }
            _ => false,
        }
    }

    pub fn on_key_up(&self, _key: KeyCode) -> bool {
        false
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::Z | KeyCode::X | KeyCode::C | KeyCode::Period)
    }
}
