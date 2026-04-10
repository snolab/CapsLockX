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
    action: AccModel2D,  // T=Delete (press_down), G=Enter (press_up)
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
            speed.page_speed, speed.page_speed, 250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let tab = AccModel2D::new(
            Arc::new(move |dx, dy, phase| tab_action(&*p, &s, dx, dy, phase)),
            speed.tab_speed, speed.tab_speed, 250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let action = AccModel2D::new(
            Arc::new(move |_dx, dy, phase| action_action(&*p, &s, dy, phase)),
            speed.action_speed, speed.action_speed, 250.0,
        );
        Self { cursor, page, tab, action }
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.cursor.set_ratios(s.cursor_speed, s.cursor_speed, 250.0);
        self.page  .set_ratios(s.page_speed,   s.page_speed,   250.0);
        self.tab   .set_ratios(s.tab_speed,    s.tab_speed,    250.0);
        self.action.set_ratios(s.action_speed, s.action_speed, 250.0);
    }

    pub fn stop(&self) {
        self.cursor.stop();
        self.page.stop();
        self.tab.stop();
        self.action.stop();
    }

    /// Advance AccModel physics by one step (called by WASM adapter tick loop).
    pub fn tick(&self) {
        self.cursor.tick_once();
        self.page.tick_once();
        self.tab.tick_once();
        self.action.tick_once();
    }

    pub fn on_key_down(&self, key: KeyCode, p: &dyn Platform) -> bool {
        match key {
            KeyCode::H => { self.cursor.press_left();  true }
            KeyCode::L => { self.cursor.press_right(); true }
            KeyCode::K => { self.cursor.press_up();    true }
            KeyCode::J => { self.cursor.press_down();  true }
            KeyCode::Y => { self.page.press_left();    true }  // Home
            KeyCode::O => { self.page.press_right();   true }  // End
            KeyCode::U => { self.page.press_down();    true }  // PageDown
            KeyCode::I => { self.page.press_up();      true }  // PageUp
            KeyCode::G => { self.action.press_up();   true }
            KeyCode::T => { self.action.press_down(); true }
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
            KeyCode::U => { self.page.release_down();    true }
            KeyCode::I => { self.page.release_up();      true }
            KeyCode::G => { self.action.release_up();   true }
            KeyCode::T => { self.action.release_down(); true }
            KeyCode::Enter | KeyCode::Backspace => true,
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

/// Collect physically-held modifier keys into a list.
fn held_modifiers(p: &dyn Platform) -> Vec<KeyCode> {
    let mut mods = Vec::new();
    if p.is_key_physically_down(KeyCode::LShift) || p.is_key_physically_down(KeyCode::RShift) { mods.push(KeyCode::LShift); }
    if p.is_key_physically_down(KeyCode::LCtrl)  || p.is_key_physically_down(KeyCode::RCtrl)  { mods.push(KeyCode::LCtrl); }
    if p.is_key_physically_down(KeyCode::LAlt)   || p.is_key_physically_down(KeyCode::RAlt)   { mods.push(KeyCode::LAlt); }
    if p.is_key_physically_down(KeyCode::LWin)   || p.is_key_physically_down(KeyCode::RWin)   { mods.push(KeyCode::LWin); }
    mods
}

/// Tap a key with all currently-held modifiers passed through.
/// Uses key_tap_with_mods which embeds flags atomically on macOS.
fn tap_with_held_mods(p: &dyn Platform, key: KeyCode, n: i32) {
    let mods = held_modifiers(p);
    if mods.is_empty() {
        p.key_tap_n(key, n);
    } else {
        p.key_tap_with_mods(key, &mods, n);
    }
}

fn cursor_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() { return; }
    match phase {
        "H_MIDKEY" => {
            let key = if dx > 0 { KeyCode::Right } else { KeyCode::Left };
            tap_with_held_mods(p, key, 1);
        }
        "V_MIDKEY" => {
            let key = if dy > 0 { KeyCode::Down } else { KeyCode::Up };
            let mods = held_modifiers(p);
            if mods.is_empty() {
                p.key_tap(key);
                p.key_tap(KeyCode::Home);
            } else {
                tap_with_held_mods(p, key, 1);
            }
        }
        "MOVE" => {
            if dy < 0 { tap_with_held_mods(p, KeyCode::Up,    -dy); }
            if dy > 0 { tap_with_held_mods(p, KeyCode::Down,   dy); }
            if dx < 0 { tap_with_held_mods(p, KeyCode::Left,  -dx); }
            if dx > 0 { tap_with_held_mods(p, KeyCode::Right,  dx); }
        }
        _ => {}
    }
}

fn page_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" { return; }
    if dy < 0 { tap_with_held_mods(p, KeyCode::PageUp,   -dy); }
    if dy > 0 { tap_with_held_mods(p, KeyCode::PageDown,  dy); }
    if dx < 0 { tap_with_held_mods(p, KeyCode::Home,     -dx); }
    if dx > 0 { tap_with_held_mods(p, KeyCode::End,       dx); }
}

fn tab_action(p: &dyn Platform, s: &ClxState, _dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" { return; }
    // N (dy>0) = Tab forward, P (dy<0) = Tab backward (Shift+Tab).
    // Any held modifiers (Ctrl, Alt, Cmd) pass through automatically.
    // So Ctrl+N = Ctrl+Tab (next tab in Chrome), Ctrl+P = Ctrl+Shift+Tab (prev tab).
    if dy > 0 {
        tap_with_held_mods(p, KeyCode::Tab, dy);
    }
    if dy < 0 {
        // P direction: add Shift to reverse Tab direction.
        let mut mods = held_modifiers(p);
        if !mods.contains(&KeyCode::LShift) { mods.push(KeyCode::LShift); }
        p.key_tap_with_mods(KeyCode::Tab, &mods, (-dy).min(128));
    }
}

fn action_action(p: &dyn Platform, s: &ClxState, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" { return; }
    if dy < 0 { tap_with_held_mods(p, KeyCode::Enter,  -dy); }  // G (preserves Ctrl/Shift/etc.)
    if dy > 0 { tap_with_held_mods(p, KeyCode::Delete,  dy); }  // T
}
