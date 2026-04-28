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
                // Off-thread: close/kill can call out to AX/AppleScript and
                // block hundreds of ms — keep the event-tap callback fast so
                // the OS doesn't disable the tap.
                let p = Arc::clone(&self.platform);
                let ctrl_alt = mods.ctrl && mods.alt;
                let shift = mods.shift;
                std::thread::spawn(move || {
                    if ctrl_alt    { p.kill_window() }
                    else if shift  { p.close_window() }
                    else           { p.close_tab() }
                });
                true
            }
            KeyCode::C => {
                // arrange_windows iterates AX windows + animates resize — was
                // taking 1.5–2s synchronously and tripping the CGEventTap
                // 1s timeout, which fires emergency_stop and drops Space.
                let p = Arc::clone(&self.platform);
                let mode = if mods.shift { ArrangeMode::SideBySide } else { ArrangeMode::Stacked };
                std::thread::spawn(move || p.arrange_windows(mode));
                true
            }
            KeyCode::Period => {
                let p = Arc::clone(&self.platform);
                std::thread::spawn(move || p.restart());
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
