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
    if phase == "移动" && (dx != 0 || dy != 0) {
        p.mouse_move(dx, dy);
    }
}

fn scroll_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "移动" { return; }
    if dy != 0 { p.scroll_v(-dy * 3); }
    if dx != 0 { p.scroll_h(dx * 3); }
}
