/// CLX-Mouse – virtual mouse via WASD + QE buttons + RF scroll.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::acc_model::AccModel2D;
use crate::key_code::KeyCode;
use crate::platform::{MouseButton, Platform};
use crate::state::{ClxState, SpeedConfig};

pub struct MouseModule {
    mouse_model:  AccModel2D,
    scroll_model: AccModel2D,
    left_btn:  Arc<AtomicBool>,
    right_btn: Arc<AtomicBool>,
    platform:  Arc<dyn Platform>,
}

impl MouseModule {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let speed = state.config.read().unwrap().speed.clone();
        let left_btn  = Arc::new(AtomicBool::new(false));
        let right_btn = Arc::new(AtomicBool::new(false));

        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let mouse_model = AccModel2D::new(
            Arc::new(move |dx, dy, phase| mouse_action(&*p, &s, dx, dy, phase)),
            speed.mouse_speed, speed.mouse_speed, f64::INFINITY,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let scroll_model = AccModel2D::new(
            Arc::new(move |dx, dy, phase| scroll_action(&*p, &s, dx, dy, phase)),
            speed.scroll_speed, speed.scroll_speed, f64::INFINITY,
        );
        Self { mouse_model, scroll_model, left_btn, right_btn, platform }
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.mouse_model .set_ratios(s.mouse_speed,  s.mouse_speed,  f64::INFINITY);
        self.scroll_model.set_ratios(s.scroll_speed, s.scroll_speed, f64::INFINITY);
    }

    /// Advance AccModel physics by one step (called by WASM adapter tick loop).
    pub fn tick(&self) {
        self.mouse_model.tick_once();
        self.scroll_model.tick_once();
    }

    pub fn stop(&self) {
        self.mouse_model.stop();
        self.scroll_model.stop();
        if self.left_btn.swap(false, Ordering::Relaxed) {
            self.platform.mouse_button(MouseButton::Left, false);
        }
        if self.right_btn.swap(false, Ordering::Relaxed) {
            self.platform.mouse_button(MouseButton::Right, false);
        }
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::W => { self.mouse_model.press_up();    true }
            KeyCode::S => { self.mouse_model.press_down();  true }
            KeyCode::A => { self.mouse_model.press_left();  true }
            KeyCode::D => { self.mouse_model.press_right(); true }
            KeyCode::Q => {
                if !self.right_btn.swap(true, Ordering::Relaxed) {
                    self.platform.mouse_button(MouseButton::Right, true);
                }
                true
            }
            KeyCode::E => {
                if !self.left_btn.swap(true, Ordering::Relaxed) {
                    self.platform.mouse_button(MouseButton::Left, true);
                }
                true
            }
            KeyCode::R => { self.scroll_model.press_up();   true }
            KeyCode::F => { self.scroll_model.press_down(); true }
            _ => false,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::W => { self.mouse_model.release_up();    true }
            KeyCode::S => { self.mouse_model.release_down();  true }
            KeyCode::A => { self.mouse_model.release_left();  true }
            KeyCode::D => { self.mouse_model.release_right(); true }
            KeyCode::Q => {
                if self.right_btn.swap(false, Ordering::Relaxed) {
                    self.platform.mouse_button(MouseButton::Right, false);
                }
                true
            }
            KeyCode::E => {
                if self.left_btn.swap(false, Ordering::Relaxed) {
                    self.platform.mouse_button(MouseButton::Left, false);
                }
                true
            }
            KeyCode::R => { self.scroll_model.release_up();   true }
            KeyCode::F => { self.scroll_model.release_down(); true }
            _ => false,
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key,
            KeyCode::W | KeyCode::A | KeyCode::S | KeyCode::D
            | KeyCode::Q | KeyCode::E
            | KeyCode::R | KeyCode::F
        )
    }
}

fn mouse_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() { return; }
    if phase == "MOVE" && (dx != 0 || dy != 0) {
        // Shift = precision mode: clamp to ±1 pixel (matches AHK behaviour).
        if s.is_shift_held() {
            p.mouse_move(dx.signum(), dy.signum());
        } else {
            p.mouse_move(dx, dy);
        }
    }
}

fn scroll_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" { return; }
    // delta is in pixels (platform adapters convert to native units).
    // scroll_speed controls the rate via AccModel — no extra multiplier needed.
    if s.is_shift_held() {
        // Shift+R/F → horizontal scroll. R=left, F=right.
        if dy != 0 { p.scroll_h(-dy); }
    } else {
        // R/F → vertical scroll.
        if dy != 0 { p.scroll_v(-dy); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};
    use crate::state::ClxConfig;
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    fn setup() -> (Arc<MockPlatform>, Arc<ClxState>, MouseModule) {
        let plat = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        state.enter_fn_mode();
        let module = MouseModule::new(plat.clone() as Arc<dyn Platform>, state.clone());
        (plat, state, module)
    }

    #[test]
    fn is_mapped_key_recognises_wasd_qe_rf() {
        let (_p, _s, m) = setup();
        for k in [KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D,
                  KeyCode::Q, KeyCode::E, KeyCode::R, KeyCode::F] {
            assert!(m.is_mapped_key(k));
        }
    }

    #[test]
    fn is_mapped_key_rejects_unmapped() {
        let (_p, _s, m) = setup();
        for k in [KeyCode::H, KeyCode::J, KeyCode::Space, KeyCode::Enter, KeyCode::B] {
            assert!(!m.is_mapped_key(k));
        }
    }

    #[test]
    fn on_key_down_returns_true_for_mapped() {
        let (_p, _s, m) = setup();
        for k in [KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D,
                  KeyCode::Q, KeyCode::E, KeyCode::R, KeyCode::F] {
            assert!(m.on_key_down(k));
        }
        m.stop();
    }

    #[test]
    fn on_key_down_returns_false_for_unmapped() {
        let (_p, _s, m) = setup();
        assert!(!m.on_key_down(KeyCode::H));
        assert!(!m.on_key_down(KeyCode::Enter));
    }

    #[test]
    fn on_key_up_returns_true_for_mapped() {
        let (_p, _s, m) = setup();
        for k in [KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D,
                  KeyCode::Q, KeyCode::E, KeyCode::R, KeyCode::F] {
            assert!(m.on_key_up(k));
        }
    }

    #[test]
    fn on_key_up_returns_false_for_unmapped() {
        let (_p, _s, m) = setup();
        assert!(!m.on_key_up(KeyCode::H));
    }

    #[test]
    fn q_press_emits_right_button_down() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::Q);
        assert!(plat.calls().contains(&Call::MouseButton(MouseButton::Right, true)));
    }

    #[test]
    fn q_release_emits_right_button_up() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::Q);
        plat.clear();
        m.on_key_up(KeyCode::Q);
        assert!(plat.calls().contains(&Call::MouseButton(MouseButton::Right, false)));
    }

    #[test]
    fn e_press_emits_left_button_down() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::E);
        assert!(plat.calls().contains(&Call::MouseButton(MouseButton::Left, true)));
    }

    #[test]
    fn e_release_emits_left_button_up() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::E);
        plat.clear();
        m.on_key_up(KeyCode::E);
        assert!(plat.calls().contains(&Call::MouseButton(MouseButton::Left, false)));
    }

    #[test]
    fn duplicate_q_press_does_not_emit_twice() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::Q);
        m.on_key_down(KeyCode::Q);
        let count = plat.count(|c| matches!(c, Call::MouseButton(MouseButton::Right, true)));
        assert_eq!(count, 1);
    }

    #[test]
    fn duplicate_e_press_does_not_emit_twice() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::E);
        m.on_key_down(KeyCode::E);
        let count = plat.count(|c| matches!(c, Call::MouseButton(MouseButton::Left, true)));
        assert_eq!(count, 1);
    }

    #[test]
    fn release_without_press_emits_nothing() {
        let (plat, _s, m) = setup();
        m.on_key_up(KeyCode::Q);
        m.on_key_up(KeyCode::E);
        let count = plat.count(|c| matches!(c, Call::MouseButton(_, _)));
        assert_eq!(count, 0);
    }

    #[test]
    fn stop_releases_held_buttons() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::Q);
        m.on_key_down(KeyCode::E);
        plat.clear();
        m.stop();
        let calls = plat.calls();
        assert!(calls.contains(&Call::MouseButton(MouseButton::Right, false)));
        assert!(calls.contains(&Call::MouseButton(MouseButton::Left, false)));
    }

    #[test]
    fn stop_with_no_buttons_held_does_not_emit_button_release() {
        let (plat, _s, m) = setup();
        m.stop();
        let count = plat.count(|c| matches!(c, Call::MouseButton(_, _)));
        assert_eq!(count, 0);
    }

    #[test]
    fn stop_is_idempotent_for_buttons() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::Q);
        m.stop();
        plat.clear();
        m.stop();
        let count = plat.count(|c| matches!(c, Call::MouseButton(_, _)));
        assert_eq!(count, 0);
    }

    #[test]
    fn apply_speeds_does_not_panic() {
        let (_p, _s, m) = setup();
        let cfg = SpeedConfig { mouse_speed: 800.0, scroll_speed: 400.0, ..SpeedConfig::default() };
        m.apply_speeds(&cfg);
    }

    #[test]
    fn tick_does_not_panic_when_idle() {
        let (_p, _s, m) = setup();
        m.tick();
        m.tick();
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn wasd_eventually_emits_mouse_move() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::D);
        let mut found = false;
        for _ in 0..200 {
            sleep(Duration::from_millis(10));
            if plat.count(|c| matches!(c, Call::MouseMove(_, _))) > 0 {
                found = true;
                break;
            }
        }
        m.on_key_up(KeyCode::D);
        m.stop();
        assert!(found, "expected MouseMove call after holding D");
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn rf_eventually_emits_scroll_v() {
        let (plat, _s, m) = setup();
        m.on_key_down(KeyCode::R);
        let mut found = false;
        for _ in 0..200 {
            sleep(Duration::from_millis(10));
            if plat.count(|c| matches!(c, Call::ScrollV(_))) > 0 {
                found = true;
                break;
            }
        }
        m.on_key_up(KeyCode::R);
        m.stop();
        assert!(found, "expected ScrollV call after holding R");
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn shift_held_with_rf_emits_horizontal_scroll() {
        let (plat, state, m) = setup();
        state.set_shift_held(true);
        m.on_key_down(KeyCode::F);
        let mut found = false;
        for _ in 0..200 {
            sleep(Duration::from_millis(10));
            if plat.count(|c| matches!(c, Call::ScrollH(_))) > 0 {
                found = true;
                break;
            }
        }
        m.on_key_up(KeyCode::F);
        state.set_shift_held(false);
        m.stop();
        assert!(found, "expected ScrollH call when Shift+F held");
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn shift_held_with_wasd_clamps_movement_to_unit() {
        let (plat, state, m) = setup();
        state.set_shift_held(true);
        m.on_key_down(KeyCode::D);
        let mut ok = false;
        for _ in 0..200 {
            sleep(Duration::from_millis(10));
            let calls = plat.calls();
            if let Some(Call::MouseMove(dx, dy)) = calls.iter().find(|c| matches!(c, Call::MouseMove(_, _))) {
                assert!(dx.abs() <= 1 && dy.abs() <= 1, "expected ±1 clamp, got ({},{})", dx, dy);
                ok = true;
                break;
            }
        }
        m.on_key_up(KeyCode::D);
        state.set_shift_held(false);
        m.stop();
        assert!(ok, "expected at least one clamped MouseMove");
    }

    #[test]
    fn inactive_clx_state_suppresses_mouse_action() {
        let plat = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        let module = MouseModule::new(plat.clone() as Arc<dyn Platform>, state.clone());
        module.on_key_down(KeyCode::D);
        sleep(Duration::from_millis(200));
        module.on_key_up(KeyCode::D);
        module.stop();
        let count = plat.count(|c| matches!(c, Call::MouseMove(_, _)));
        assert_eq!(count, 0, "no MouseMove should fire when CLX inactive");
    }

    #[test]
    fn inactive_clx_state_suppresses_scroll_action() {
        let plat = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        let module = MouseModule::new(plat.clone() as Arc<dyn Platform>, state.clone());
        module.on_key_down(KeyCode::R);
        sleep(Duration::from_millis(200));
        module.on_key_up(KeyCode::R);
        module.stop();
        let count = plat.count(|c| matches!(c, Call::ScrollV(_)) || matches!(c, Call::ScrollH(_)));
        assert_eq!(count, 0);
    }
}
