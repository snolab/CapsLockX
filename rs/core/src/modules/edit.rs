/// CLX-Edit – cursor & page navigation via HJKL / YUIO / GT / PN.
use std::sync::Arc;
use crate::acc_model::AccModel2D;
use crate::key_code::KeyCode;
use crate::platform::Platform;
use crate::state::{ClxState, SpeedConfig};

pub struct EditModule {
    cursor: AccModel2D,
    page:   AccModel2D,
    tab:    AccModel2D,
}

impl EditModule {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let speed = state.config.read().unwrap().speed.clone();
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let cursor = AccModel2D::new(
            Arc::new(move |dx, dy, phase| cursor_action(&*p, &s, dx, dy, phase)),
            speed.cursor_speed, speed.cursor_speed, 250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let page = AccModel2D::new(
            Arc::new(move |dx, dy, phase| page_action(&*p, &s, dx, dy, phase)),
            speed.cursor_speed, speed.cursor_speed, 250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let tab = AccModel2D::new(
            Arc::new(move |dx, dy, phase| tab_action(&*p, &s, dx, dy, phase)),
            speed.cursor_speed, speed.cursor_speed, 250.0,
        );
        Self { cursor, page, tab }
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.cursor.set_ratios(s.cursor_speed, s.cursor_speed, 250.0);
        self.page  .set_ratios(s.cursor_speed, s.cursor_speed, 250.0);
        self.tab   .set_ratios(s.cursor_speed, s.cursor_speed, 250.0);
    }

    pub fn stop(&self) {
        self.cursor.stop();
        self.page.stop();
        self.tab.stop();
    }

    /// Advance AccModel physics by one step (called by WASM adapter tick loop).
    pub fn tick(&self) {
        self.cursor.tick_once();
        self.page.tick_once();
        self.tab.tick_once();
    }

    pub fn on_key_down(&self, key: KeyCode, p: &dyn Platform) -> bool {
        match key {
            KeyCode::H => { self.cursor.press_left();  true }
            KeyCode::L => { self.cursor.press_right(); true }
            KeyCode::K => { self.cursor.press_up();    true }
            KeyCode::J => { self.cursor.press_down();  true }
            KeyCode::Y => { self.page.press_left();    true }  // Home
            KeyCode::O => { self.page.press_right();   true }  // End
            KeyCode::U => { self.page.press_up();      true }  // PageUp
            KeyCode::I => { self.page.press_down();    true }  // PageDown
            KeyCode::G => { p.key_tap(KeyCode::Enter);  true }
            KeyCode::T => { p.key_tap(KeyCode::Delete); true }
            KeyCode::P => { self.tab.press_up();       true }  // Shift+Tab
            KeyCode::N => { self.tab.press_down();     true }  // Tab
            // EnterWithoutBreak: End → Enter (newline without splitting line)
            KeyCode::Enter => {
                p.key_tap(KeyCode::End);
                p.key_tap(KeyCode::End);
                p.key_tap(KeyCode::Enter);
                true
            }
            // Delete entire line: Home Home → Shift+End Shift+Right → Delete
            KeyCode::Backspace => {
                p.key_tap(KeyCode::Home);
                p.key_tap(KeyCode::Home);
                p.key_tap(KeyCode::End);
                p.key_tap(KeyCode::Home);
                p.key_tap(KeyCode::Home);
                p.key_tap_shifted(KeyCode::End);
                p.key_tap_shifted(KeyCode::End);
                p.key_tap_shifted(KeyCode::Right);
                p.key_tap(KeyCode::Delete);
                true
            }
            _ => false,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::H => { self.cursor.release_left();  true }
            KeyCode::L => { self.cursor.release_right(); true }
            KeyCode::K => { self.cursor.release_up();    true }
            KeyCode::J => { self.cursor.release_down();  true }
            KeyCode::Y => { self.page.release_left();    true }
            KeyCode::O => { self.page.release_right();   true }
            KeyCode::U => { self.page.release_up();      true }
            KeyCode::I => { self.page.release_down();    true }
            KeyCode::G | KeyCode::T | KeyCode::Enter | KeyCode::Backspace => true,
            KeyCode::P => { self.tab.release_up();       true }
            KeyCode::N => { self.tab.release_down();     true }
            _ => false,
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(
            key,
            KeyCode::H | KeyCode::J | KeyCode::K | KeyCode::L
            | KeyCode::Y | KeyCode::U | KeyCode::I | KeyCode::O
            | KeyCode::G | KeyCode::T
            | KeyCode::P | KeyCode::N
            | KeyCode::Enter | KeyCode::Backspace
        )
    }
}

// ── Action callbacks ──────────────────────────────────────────────────────────

fn cursor_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() { return; }
    match phase {
        "横中键" => {
            if dx > 0 { p.key_tap(KeyCode::Right); } else { p.key_tap(KeyCode::Left); }
        }
        "纵中键" => {
            if dy > 0 { p.key_tap(KeyCode::Down); } else { p.key_tap(KeyCode::Up); }
            p.key_tap(KeyCode::Home);
        }
        "移动" => {
            if dy < 0 { p.key_tap_n(KeyCode::Up,    -dy); }
            if dy > 0 { p.key_tap_n(KeyCode::Down,   dy); }
            if dx < 0 { p.key_tap_n(KeyCode::Left,  -dx); }
            if dx > 0 { p.key_tap_n(KeyCode::Right,  dx); }
        }
        _ => {}
    }
}

fn page_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "移动" { return; }
    if dy < 0 { p.key_tap_n(KeyCode::PageUp,   -dy); }
    if dy > 0 { p.key_tap_n(KeyCode::PageDown,  dy); }
    if dx < 0 { p.key_tap_n(KeyCode::Home,     -dx); }
    if dx > 0 { p.key_tap_n(KeyCode::End,       dx); }
}

fn tab_action(p: &dyn Platform, s: &ClxState, _dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "移动" { return; }
    if dy < 0 { p.key_tap_shifted_n(KeyCode::Tab, -dy); }
    if dy > 0 { p.key_tap_n(KeyCode::Tab,          dy); }
}
