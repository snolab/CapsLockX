/// CLX-WindowManager – window cycling (AccModel), tiling, close/kill.
///
/// Z uses AccModel2D for accelerating window cycling — hold Z to cycle
/// faster and faster, like HJKL cursor movement.
use std::sync::Arc;
use crate::acc_model::AccModel2D;
use crate::key_code::{KeyCode, Modifiers};
use crate::platform::{ArrangeMode, Platform};
use crate::state::{ClxState, SpeedConfig};

pub struct WindowManagerModule {
    platform: Arc<dyn Platform>,
    cycle: AccModel2D,
}

impl WindowManagerModule {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let speed = state.config.read().unwrap().speed.clone();
        let p = Arc::clone(&platform);
        let cycle = AccModel2D::new(
            Arc::new(move |dx, _dy, phase| {
                if phase == "MOVE" {
                    // dx > 0 = forward, dx < 0 = backward
                    for _ in 0..dx.abs() {
                        p.cycle_windows(if dx > 0 { 1 } else { -1 });
                    }
                }
            }),
            speed.cursor_speed * 0.5, // slower than cursor — windows are heavier
            speed.cursor_speed * 0.5,
            250.0,
        );
        Self { platform, cycle }
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.cycle.set_ratios(s.cursor_speed * 0.5, s.cursor_speed * 0.5, 250.0);
    }

    pub fn stop(&self) {
        self.cycle.stop();
    }

    pub fn tick(&self) {
        self.cycle.tick_once();
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        match key {
            KeyCode::Z => {
                if mods.shift { self.cycle.press_left() }  // backward
                else          { self.cycle.press_right() } // forward
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
            KeyCode::Period => {
                self.platform.restart();
                true
            }
            _ => false,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::Z => { self.cycle.stop(); true }
            _ => false,
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::Z | KeyCode::X | KeyCode::C | KeyCode::Period)
    }
}
