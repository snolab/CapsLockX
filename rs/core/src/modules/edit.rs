use crate::acc_model::AccModel2D;
use crate::key_code::KeyCode;
use crate::platform::Platform;
use crate::state::{ClxState, SpeedConfig};
/// CLX-Edit – cursor & page navigation via HJKL / YUIO / GT / PN.
use std::sync::Arc;

pub struct EditModule {
    cursor: AccModel2D,
    page: AccModel2D,
    tab: AccModel2D,
    action: AccModel2D, // T=Delete (press_down), G=Enter (press_up)
}

impl EditModule {
    pub fn new(platform: Arc<dyn Platform>, state: Arc<ClxState>) -> Self {
        let speed = state.config.read().unwrap().speed.clone();
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let cursor = AccModel2D::new(
            Arc::new(move |dx, dy, phase| cursor_action(&*p, &s, dx, dy, phase)),
            speed.cursor_speed,
            speed.cursor_speed,
            250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let page = AccModel2D::new(
            Arc::new(move |dx, dy, phase| page_action(&*p, &s, dx, dy, phase)),
            speed.page_speed,
            speed.page_speed,
            250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let tab = AccModel2D::new(
            Arc::new(move |dx, dy, phase| tab_action(&*p, &s, dx, dy, phase)),
            speed.tab_speed,
            speed.tab_speed,
            250.0,
        );
        let (p, s) = (Arc::clone(&platform), Arc::clone(&state));
        let action = AccModel2D::new(
            Arc::new(move |_dx, dy, phase| action_action(&*p, &s, dy, phase)),
            speed.action_speed,
            speed.action_speed,
            250.0,
        );
        Self {
            cursor,
            page,
            tab,
            action,
        }
    }

    pub fn apply_speeds(&self, s: &SpeedConfig) {
        self.cursor
            .set_ratios(s.cursor_speed, s.cursor_speed, 250.0);
        self.page.set_ratios(s.page_speed, s.page_speed, 250.0);
        self.tab.set_ratios(s.tab_speed, s.tab_speed, 250.0);
        self.action
            .set_ratios(s.action_speed, s.action_speed, 250.0);
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
            KeyCode::H => {
                self.cursor.press_left();
                true
            }
            KeyCode::L => {
                self.cursor.press_right();
                true
            }
            KeyCode::K => {
                self.cursor.press_up();
                true
            }
            KeyCode::J => {
                self.cursor.press_down();
                true
            }
            KeyCode::Y => {
                self.page.press_left();
                true
            } // Home
            KeyCode::O => {
                self.page.press_right();
                true
            } // End
            KeyCode::U => {
                self.page.press_down();
                true
            } // PageDown
            KeyCode::I => {
                self.page.press_up();
                true
            } // PageUp
            KeyCode::G => {
                self.action.press_up();
                true
            }
            KeyCode::T => {
                self.action.press_down();
                true
            }
            KeyCode::P => {
                self.tab.press_up();
                true
            } // Shift+Tab
            KeyCode::N => {
                self.tab.press_down();
                true
            } // Tab
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
            KeyCode::H => {
                self.cursor.release_left();
                true
            }
            KeyCode::L => {
                self.cursor.release_right();
                true
            }
            KeyCode::K => {
                self.cursor.release_up();
                true
            }
            KeyCode::J => {
                self.cursor.release_down();
                true
            }
            KeyCode::Y => {
                self.page.release_left();
                true
            }
            KeyCode::O => {
                self.page.release_right();
                true
            }
            KeyCode::U => {
                self.page.release_down();
                true
            }
            KeyCode::I => {
                self.page.release_up();
                true
            }
            KeyCode::G => {
                self.action.release_up();
                true
            }
            KeyCode::T => {
                self.action.release_down();
                true
            }
            KeyCode::Enter | KeyCode::Backspace => true,
            KeyCode::P => {
                self.tab.release_up();
                true
            }
            KeyCode::N => {
                self.tab.release_down();
                true
            }
            _ => false,
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(
            key,
            KeyCode::H
                | KeyCode::J
                | KeyCode::K
                | KeyCode::L
                | KeyCode::Y
                | KeyCode::U
                | KeyCode::I
                | KeyCode::O
                | KeyCode::G
                | KeyCode::T
                | KeyCode::P
                | KeyCode::N
                | KeyCode::Enter
                | KeyCode::Backspace
        )
    }
}

// ── Action callbacks ──────────────────────────────────────────────────────────

/// Collect physically-held modifier keys into a list.
fn held_modifiers(p: &dyn Platform) -> Vec<KeyCode> {
    let mut mods = Vec::new();
    if p.is_key_physically_down(KeyCode::LShift) || p.is_key_physically_down(KeyCode::RShift) {
        mods.push(KeyCode::LShift);
    }
    if p.is_key_physically_down(KeyCode::LCtrl) || p.is_key_physically_down(KeyCode::RCtrl) {
        mods.push(KeyCode::LCtrl);
    }
    if p.is_key_physically_down(KeyCode::LAlt) || p.is_key_physically_down(KeyCode::RAlt) {
        mods.push(KeyCode::LAlt);
    }
    if p.is_key_physically_down(KeyCode::LWin) || p.is_key_physically_down(KeyCode::RWin) {
        mods.push(KeyCode::LWin);
    }
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
    if !s.is_clx_active() {
        return;
    }
    match phase {
        "H_MIDKEY" => {
            let key = if dx > 0 {
                KeyCode::Right
            } else {
                KeyCode::Left
            };
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
            if dy < 0 {
                tap_with_held_mods(p, KeyCode::Up, -dy);
            }
            if dy > 0 {
                tap_with_held_mods(p, KeyCode::Down, dy);
            }
            if dx < 0 {
                tap_with_held_mods(p, KeyCode::Left, -dx);
            }
            if dx > 0 {
                tap_with_held_mods(p, KeyCode::Right, dx);
            }
        }
        _ => {}
    }
}

fn page_action(p: &dyn Platform, s: &ClxState, dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" {
        return;
    }
    if dy < 0 {
        tap_with_held_mods(p, KeyCode::PageUp, -dy);
    }
    if dy > 0 {
        tap_with_held_mods(p, KeyCode::PageDown, dy);
    }
    if dx < 0 {
        tap_with_held_mods(p, KeyCode::Home, -dx);
    }
    if dx > 0 {
        tap_with_held_mods(p, KeyCode::End, dx);
    }
}

fn tab_action(p: &dyn Platform, s: &ClxState, _dx: i32, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" {
        return;
    }
    // N (dy>0) = Tab forward, P (dy<0) = Tab backward (Shift+Tab).
    // Any held modifiers (Ctrl, Alt, Cmd) pass through automatically.
    // So Ctrl+N = Ctrl+Tab (next tab in Chrome), Ctrl+P = Ctrl+Shift+Tab (prev tab).
    if dy > 0 {
        tap_with_held_mods(p, KeyCode::Tab, dy);
    }
    if dy < 0 {
        // P direction: add Shift to reverse Tab direction.
        let mut mods = held_modifiers(p);
        if !mods.contains(&KeyCode::LShift) {
            mods.push(KeyCode::LShift);
        }
        p.key_tap_with_mods(KeyCode::Tab, &mods, (-dy).min(128));
    }
}

fn action_action(p: &dyn Platform, s: &ClxState, dy: i32, phase: &str) {
    if !s.is_clx_active() || phase != "MOVE" {
        return;
    }
    if dy < 0 {
        tap_with_held_mods(p, KeyCode::Enter, -dy);
    } // G (preserves Ctrl/Shift/etc.)
    if dy > 0 {
        tap_with_held_mods(p, KeyCode::Delete, dy);
    } // T
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acc_model::set_external_tick;
    use crate::state::{ClxConfig, ClxState, SpeedConfig};
    use crate::test_platform::{Call, MockPlatform};
    use std::sync::Arc;
    use std::sync::Once;
    use std::thread::sleep;
    use std::time::Duration;

    static EXTERNAL_TICK_INIT: Once = Once::new();

    fn enable_external_tick() {
        EXTERNAL_TICK_INIT.call_once(|| set_external_tick(true));
    }

    fn make_module(active: bool) -> (Arc<MockPlatform>, Arc<ClxState>, EditModule) {
        enable_external_tick();
        let mock = Arc::new(MockPlatform::new());
        let state = Arc::new(ClxState::new(ClxConfig::default()));
        if active {
            state.enter_fn_mode();
        }
        let m = EditModule::new(mock.clone() as Arc<dyn Platform>, state.clone());
        (mock, state, m)
    }

    fn drive_until_move(module: &EditModule, mock: &MockPlatform) {
        for _ in 0..20 {
            module.tick();
            if mock.calls().iter().any(|c| matches!(c, Call::KeyDown(_))) {
                return;
            }
            sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn is_mapped_key_covers_all_documented_keys() {
        let (_m, _s, e) = make_module(false);
        for k in [
            KeyCode::H,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::Y,
            KeyCode::U,
            KeyCode::I,
            KeyCode::O,
            KeyCode::G,
            KeyCode::T,
            KeyCode::P,
            KeyCode::N,
            KeyCode::Enter,
            KeyCode::Backspace,
        ] {
            assert!(e.is_mapped_key(k), "{:?} should be mapped", k);
        }
    }

    #[test]
    fn is_mapped_key_rejects_unmapped_keys() {
        let (_m, _s, e) = make_module(false);
        for k in [
            KeyCode::A,
            KeyCode::Z,
            KeyCode::Space,
            KeyCode::CapsLock,
            KeyCode::F1,
        ] {
            assert!(!e.is_mapped_key(k), "{:?} should NOT be mapped", k);
        }
    }

    #[test]
    fn enter_keydown_taps_end_end_enter() {
        let (mock, _s, e) = make_module(false);
        assert!(e.on_key_down(KeyCode::Enter, &*mock));
        let calls = mock.calls();
        let taps: Vec<KeyCode> = calls
            .iter()
            .filter_map(|c| match c {
                Call::KeyDown(k) => Some(*k),
                _ => None,
            })
            .collect();
        assert_eq!(taps, vec![KeyCode::End, KeyCode::End, KeyCode::Enter]);
    }

    #[test]
    fn backspace_keydown_performs_full_delete_line_sequence() {
        let (mock, _s, e) = make_module(false);
        assert!(e.on_key_down(KeyCode::Backspace, &*mock));
        let calls = mock.calls();
        let downs: Vec<KeyCode> = calls
            .iter()
            .filter_map(|c| match c {
                Call::KeyDown(k) => Some(*k),
                _ => None,
            })
            .collect();
        assert_eq!(
            downs,
            vec![
                KeyCode::Home,
                KeyCode::Home,
                KeyCode::End,
                KeyCode::Home,
                KeyCode::Home,
                KeyCode::LShift,
                KeyCode::End,
                KeyCode::LShift,
                KeyCode::End,
                KeyCode::LShift,
                KeyCode::Right,
                KeyCode::Delete,
            ]
        );
    }

    #[test]
    fn unmapped_keys_return_false_on_key_down() {
        let (mock, _s, e) = make_module(false);
        assert!(!e.on_key_down(KeyCode::A, &*mock));
        assert!(!e.on_key_down(KeyCode::Space, &*mock));
        assert!(!e.on_key_down(KeyCode::F1, &*mock));
    }

    #[test]
    fn unmapped_keys_return_false_on_key_up() {
        let (_mock, _s, e) = make_module(false);
        assert!(!e.on_key_up(KeyCode::A));
        assert!(!e.on_key_up(KeyCode::Space));
    }

    #[test]
    fn cursor_keys_are_accepted_on_down_and_up() {
        let (mock, _s, e) = make_module(false);
        for k in [KeyCode::H, KeyCode::J, KeyCode::K, KeyCode::L] {
            assert!(e.on_key_down(k, &*mock), "down {:?}", k);
            assert!(e.on_key_up(k), "up {:?}", k);
        }
    }

    #[test]
    fn page_keys_are_accepted_on_down_and_up() {
        let (mock, _s, e) = make_module(false);
        for k in [KeyCode::Y, KeyCode::U, KeyCode::I, KeyCode::O] {
            assert!(e.on_key_down(k, &*mock));
            assert!(e.on_key_up(k));
        }
    }

    #[test]
    fn action_keys_g_and_t_are_accepted() {
        let (mock, _s, e) = make_module(false);
        assert!(e.on_key_down(KeyCode::G, &*mock));
        assert!(e.on_key_up(KeyCode::G));
        assert!(e.on_key_down(KeyCode::T, &*mock));
        assert!(e.on_key_up(KeyCode::T));
    }

    #[test]
    fn tab_keys_n_and_p_are_accepted() {
        let (mock, _s, e) = make_module(false);
        assert!(e.on_key_down(KeyCode::N, &*mock));
        assert!(e.on_key_up(KeyCode::N));
        assert!(e.on_key_down(KeyCode::P, &*mock));
        assert!(e.on_key_up(KeyCode::P));
    }

    #[test]
    fn enter_and_backspace_keyup_return_true() {
        let (_mock, _s, e) = make_module(false);
        assert!(e.on_key_up(KeyCode::Enter));
        assert!(e.on_key_up(KeyCode::Backspace));
    }

    #[test]
    fn apply_speeds_and_stop_and_tick_do_not_panic() {
        let (_mock, _s, e) = make_module(false);
        let cfg = SpeedConfig {
            cursor_speed: 100.0,
            page_speed: 50.0,
            tab_speed: 25.0,
            action_speed: 10.0,
            mouse_speed: 1.0,
            scroll_speed: 1.0,
        };
        e.apply_speeds(&cfg);
        e.tick();
        e.stop();
    }

    #[test]
    fn cursor_action_emits_right_arrow_when_active_and_l_pressed() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::L, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::L);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::Right))),
            "expected a Right key tap, got: {:?}",
            calls
        );
    }

    #[test]
    fn cursor_action_does_nothing_when_clx_inactive() {
        let (mock, _s, e) = make_module(false);
        assert!(e.on_key_down(KeyCode::L, &*mock));
        for _ in 0..5 {
            e.tick();
            sleep(Duration::from_millis(10));
        }
        e.on_key_up(KeyCode::L);
        assert!(
            mock.calls().is_empty(),
            "no platform calls when inactive, got {:?}",
            mock.calls()
        );
    }

    #[test]
    fn page_action_emits_end_when_active_and_o_pressed() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::O, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::O);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::End))),
            "expected an End key tap, got: {:?}",
            calls
        );
    }

    #[test]
    fn tab_action_emits_tab_when_active_and_n_pressed() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::N, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::N);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::Tab))),
            "expected a Tab key tap, got: {:?}",
            calls
        );
    }

    #[test]
    fn tab_action_p_direction_emits_shift_tab() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::P, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::P);
        let calls = mock.calls();
        let has_shift = calls
            .iter()
            .any(|c| matches!(c, Call::KeyDown(KeyCode::LShift)));
        let has_tab = calls
            .iter()
            .any(|c| matches!(c, Call::KeyDown(KeyCode::Tab)));
        assert!(has_shift && has_tab, "expected Shift+Tab, got: {:?}", calls);
    }

    #[test]
    fn action_action_g_emits_enter() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::G, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::G);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::Enter))),
            "expected Enter, got: {:?}",
            calls
        );
    }

    #[test]
    fn action_action_t_emits_delete() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::T, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::T);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::Delete))),
            "expected Delete, got: {:?}",
            calls
        );
    }

    #[test]
    fn cursor_action_vertical_emits_up_or_down_and_home_when_no_mods() {
        let (mock, _s, e) = make_module(true);
        assert!(e.on_key_down(KeyCode::J, &*mock));
        drive_until_move(&e, &mock);
        e.on_key_up(KeyCode::J);
        let calls = mock.calls();
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::KeyDown(KeyCode::Down))),
            "expected Down, got: {:?}",
            calls
        );
    }
}
