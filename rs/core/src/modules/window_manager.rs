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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ClxConfig;
    use crate::test_platform::{Call, MockPlatform};

    fn setup() -> (Arc<MockPlatform>, WindowManagerModule) {
        let mock = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        let module = WindowManagerModule::new(mock.clone(), state);
        (mock, module)
    }

    fn mods(shift: bool, ctrl: bool, alt: bool) -> Modifiers {
        let mut m = Modifiers::default();
        m.shift = shift;
        m.ctrl = ctrl;
        m.alt = alt;
        m
    }

    #[test]
    fn x_without_modifiers_closes_tab() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::X, &mods(false, false, false)));
        assert_eq!(mock.calls(), vec![Call::CloseTab]);
    }

    #[test]
    fn shift_x_closes_window() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::X, &mods(true, false, false)));
        assert_eq!(mock.calls(), vec![Call::CloseWindow]);
    }

    #[test]
    fn ctrl_alt_x_kills_window() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::X, &mods(false, true, true)));
        assert_eq!(mock.calls(), vec![Call::KillWindow]);
    }

    #[test]
    fn c_arranges_windows_stacked() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::C, &mods(false, false, false)));
        assert_eq!(mock.calls(), vec![Call::ArrangeWindows(ArrangeMode::Stacked)]);
    }

    #[test]
    fn shift_c_arranges_windows_side_by_side() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::C, &mods(true, false, false)));
        assert_eq!(mock.calls(), vec![Call::ArrangeWindows(ArrangeMode::SideBySide)]);
    }

    #[test]
    fn period_restarts_application() {
        let (mock, module) = setup();
        assert!(module.on_key_down(KeyCode::Period, &mods(false, false, false)));
        assert_eq!(mock.calls(), vec![Call::Restart]);
    }

    #[test]
    fn z_press_returns_true_and_release_returns_true() {
        let (_mock, module) = setup();
        assert!(module.on_key_down(KeyCode::Z, &mods(false, false, false)));
        assert!(module.on_key_up(KeyCode::Z));
    }

    #[test]
    fn shift_z_press_returns_true_for_backward_cycle() {
        let (_mock, module) = setup();
        assert!(module.on_key_down(KeyCode::Z, &mods(true, false, false)));
        module.stop();
    }

    #[test]
    fn unmapped_key_returns_false_for_down_and_up() {
        let (mock, module) = setup();
        assert!(!module.on_key_down(KeyCode::A, &mods(false, false, false)));
        assert!(!module.on_key_up(KeyCode::A));
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn is_mapped_key_recognises_handled_keys_only() {
        let (_, module) = setup();
        for k in [KeyCode::Z, KeyCode::X, KeyCode::C, KeyCode::Period] {
            assert!(module.is_mapped_key(k));
        }
        assert!(!module.is_mapped_key(KeyCode::A));
        assert!(!module.is_mapped_key(KeyCode::F1));
    }

    #[test]
    fn apply_speeds_updates_cycle_ratios_without_panicking() {
        let (_, module) = setup();
        let mut s = SpeedConfig::default();
        s.cursor_speed = 120.0;
        module.apply_speeds(&s);
    }

    #[test]
    fn tick_advances_cycle_without_panicking() {
        let (_, module) = setup();
        module.tick();
        module.stop();
    }
}
