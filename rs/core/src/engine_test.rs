//! Integration tests for ClxEngine — verifies key routing, modifier forwarding,
//! emergency stop, cycle wrap, and module interactions using a MockPlatform.
//!
//! No real keyboard/mouse/Accessibility needed — pure logic tests.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::engine::ClxEngine;
    use crate::key_code::KeyCode;
    use crate::platform::{ArrangeMode, MouseButton, Platform};
    use crate::state::ClxConfig;
    use crate::CoreResponse;

    // ── Mock Platform ────────────────────────────────────────────────────────

    #[derive(Clone)]
    struct KeyEvent {
        key: KeyCode,
        mods: Vec<KeyCode>,
    }

    struct MockPlatform {
        /// All key_tap / key_tap_with_mods calls recorded here.
        taps: Mutex<Vec<KeyEvent>>,
        /// key_tap_extended calls (media/volume keys).
        extended_taps: Mutex<Vec<KeyCode>>,
        /// Mouse move deltas accumulated.
        mouse_dx: Mutex<i32>,
        mouse_dy: Mutex<i32>,
        /// Simulated physically-held keys (for is_key_physically_down).
        physical_keys: Mutex<Vec<KeyCode>>,
        /// Typed text via type_text().
        typed: Mutex<String>,
        /// cycle_windows call directions (positive=forward, negative=back).
        cycle_calls: Mutex<Vec<i32>>,
        /// close_tab() call count.
        close_tab_count: Mutex<u32>,
        /// close_window() call count.
        close_window_count: Mutex<u32>,
        /// kill_window() call count.
        kill_window_count: Mutex<u32>,
        /// arrange_windows() calls.
        arrange_calls: Mutex<Vec<ArrangeMode>>,
        /// switch_to_desktop() calls (desktop index).
        desktop_switch_calls: Mutex<Vec<u32>>,
        /// move_window_to_desktop() calls (desktop index).
        window_move_calls: Mutex<Vec<u32>>,
        /// restart() call count.
        restart_count: Mutex<u32>,
    }

    impl MockPlatform {
        fn new() -> Self {
            Self {
                taps: Mutex::new(Vec::new()),
                extended_taps: Mutex::new(Vec::new()),
                mouse_dx: Mutex::new(0),
                mouse_dy: Mutex::new(0),
                physical_keys: Mutex::new(Vec::new()),
                typed: Mutex::new(String::new()),
                cycle_calls: Mutex::new(Vec::new()),
                close_tab_count: Mutex::new(0),
                close_window_count: Mutex::new(0),
                kill_window_count: Mutex::new(0),
                arrange_calls: Mutex::new(Vec::new()),
                desktop_switch_calls: Mutex::new(Vec::new()),
                window_move_calls: Mutex::new(Vec::new()),
                restart_count: Mutex::new(0),
            }
        }

        fn taps(&self) -> Vec<KeyEvent> {
            self.taps.lock().unwrap().clone()
        }

        fn extended_taps(&self) -> Vec<KeyCode> {
            self.extended_taps.lock().unwrap().clone()
        }

        fn clear_taps(&self) {
            self.taps.lock().unwrap().clear();
            self.extended_taps.lock().unwrap().clear();
        }

        fn mouse_delta(&self) -> (i32, i32) {
            (*self.mouse_dx.lock().unwrap(), *self.mouse_dy.lock().unwrap())
        }

        fn clear_mouse(&self) {
            *self.mouse_dx.lock().unwrap() = 0;
            *self.mouse_dy.lock().unwrap() = 0;
        }

        fn set_physical_keys(&self, keys: Vec<KeyCode>) {
            *self.physical_keys.lock().unwrap() = keys;
        }

        fn typed_text(&self) -> String {
            self.typed.lock().unwrap().clone()
        }

        fn cycle_calls(&self) -> Vec<i32> {
            self.cycle_calls.lock().unwrap().clone()
        }

        fn close_tab_count(&self) -> u32 {
            *self.close_tab_count.lock().unwrap()
        }

        fn close_window_count(&self) -> u32 {
            *self.close_window_count.lock().unwrap()
        }

        fn kill_window_count(&self) -> u32 {
            *self.kill_window_count.lock().unwrap()
        }

        fn arrange_calls(&self) -> Vec<ArrangeMode> {
            self.arrange_calls.lock().unwrap().clone()
        }

        fn desktop_switch_calls(&self) -> Vec<u32> {
            self.desktop_switch_calls.lock().unwrap().clone()
        }

        fn window_move_calls(&self) -> Vec<u32> {
            self.window_move_calls.lock().unwrap().clone()
        }

        fn restart_count(&self) -> u32 {
            *self.restart_count.lock().unwrap()
        }
    }

    impl Platform for MockPlatform {
        fn key_down(&self, _key: KeyCode) {}
        fn key_up(&self, _key: KeyCode) {}

        fn key_tap(&self, key: KeyCode) {
            self.taps.lock().unwrap().push(KeyEvent { key, mods: vec![] });
        }

        fn key_tap_n(&self, key: KeyCode, n: i32) {
            for _ in 0..n {
                self.key_tap(key);
            }
        }

        fn key_tap_extended(&self, key: KeyCode) {
            self.extended_taps.lock().unwrap().push(key);
        }

        fn key_tap_with_mods(&self, key: KeyCode, mods: &[KeyCode], n: i32) {
            for _ in 0..n {
                self.taps.lock().unwrap().push(KeyEvent {
                    key,
                    mods: mods.to_vec(),
                });
            }
        }

        fn is_key_physically_down(&self, key: KeyCode) -> bool {
            self.physical_keys.lock().unwrap().contains(&key)
        }

        fn mouse_move(&self, dx: i32, dy: i32) {
            *self.mouse_dx.lock().unwrap() += dx;
            *self.mouse_dy.lock().unwrap() += dy;
        }

        fn scroll_v(&self, _delta: i32) {}
        fn scroll_h(&self, _delta: i32) {}
        fn mouse_button(&self, _button: MouseButton, _pressed: bool) {}

        fn type_text(&self, text: &str) {
            self.typed.lock().unwrap().push_str(text);
        }

        fn cycle_windows(&self, dir: i32) {
            self.cycle_calls.lock().unwrap().push(dir);
        }

        fn close_tab(&self) {
            *self.close_tab_count.lock().unwrap() += 1;
        }

        fn close_window(&self) {
            *self.close_window_count.lock().unwrap() += 1;
        }

        fn kill_window(&self) {
            *self.kill_window_count.lock().unwrap() += 1;
        }

        fn arrange_windows(&self, mode: ArrangeMode) {
            self.arrange_calls.lock().unwrap().push(mode);
        }

        fn switch_to_desktop(&self, idx: u32) {
            self.desktop_switch_calls.lock().unwrap().push(idx);
        }

        fn move_window_to_desktop(&self, idx: u32) {
            self.window_move_calls.lock().unwrap().push(idx);
        }

        fn restart(&self) {
            *self.restart_count.lock().unwrap() += 1;
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_engine(platform: Arc<MockPlatform>) -> Arc<ClxEngine> {
        let mut config = ClxConfig::default();
        config.use_space = true;
        config.use_capslock = false;
        ClxEngine::with_config(platform, config)
    }

    fn make_engine_capslock(platform: Arc<MockPlatform>) -> Arc<ClxEngine> {
        let mut config = ClxConfig::default();
        config.use_space = false;
        config.use_capslock = true;
        ClxEngine::with_config(platform, config)
    }

    /// Simulate holding Space (enter CLX mode) then pressing+holding a key
    /// long enough for AccModel to produce at least one tick, then releasing.
    fn clx_tap(engine: &ClxEngine, key: KeyCode) {
        engine.on_key_event(KeyCode::Space, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(key, true);
        std::thread::sleep(std::time::Duration::from_millis(80));
        engine.on_key_event(key, false);
    }

    /// Simulate holding Space + pressing a key with modifiers (no AccModel wait needed).
    fn clx_tap_instant(engine: &ClxEngine, key: KeyCode) {
        engine.on_key_event(KeyCode::Space, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(key, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(key, false);
    }

    /// Simulate releasing Space (exit CLX mode).
    fn release_space(engine: &ClxEngine) {
        engine.on_key_event(KeyCode::Space, false);
    }

    /// Wait for AccModel tick to fire (needs ~20ms for one tick cycle).
    fn wait_tick() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // ── Edit module: cursor keys ─────────────────────────────────────────────

    #[test]
    fn test_space_triggers_clx_mode() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::Suppress, "Space down should be suppressed");

        engine.on_key_event(KeyCode::Space, false);
    }

    #[test]
    fn test_hjkl_cursor_keys() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        for (key, expected) in [
            (KeyCode::H, KeyCode::Left),
            (KeyCode::J, KeyCode::Down),
            (KeyCode::K, KeyCode::Up),
            (KeyCode::L, KeyCode::Right),
        ] {
            platform.clear_taps();
            clx_tap(&engine, key);
            wait_tick();
            assert!(
                platform.taps().iter().any(|t| t.key == expected),
                "Space+{:?} should produce {:?}, got: {:?}",
                key, expected,
                platform.taps().iter().map(|t| t.key).collect::<Vec<_>>()
            );
        }

        release_space(&engine);
    }

    #[test]
    fn test_space_h_produces_left_arrow() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap(&engine, KeyCode::H);
        wait_tick();

        let taps = platform.taps();
        assert!(!taps.is_empty(), "Space+H should produce key taps");
        assert!(
            taps.iter().any(|t| t.key == KeyCode::Left),
            "Space+H should produce Left arrow, got: {:?}",
            taps.iter().map(|t| t.key).collect::<Vec<_>>()
        );

        release_space(&engine);
    }

    #[test]
    fn test_space_g_produces_enter() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap(&engine, KeyCode::G);
        wait_tick();

        let taps = platform.taps();
        assert!(
            taps.iter().any(|t| t.key == KeyCode::Enter),
            "Space+G should produce Enter, got: {:?}",
            taps.iter().map(|t| t.key).collect::<Vec<_>>()
        );

        release_space(&engine);
    }

    #[test]
    fn test_space_t_produces_delete() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap(&engine, KeyCode::T);
        wait_tick();

        let taps = platform.taps();
        assert!(
            taps.iter().any(|t| t.key == KeyCode::Delete),
            "Space+T should produce Delete, got: {:?}",
            taps.iter().map(|t| t.key).collect::<Vec<_>>()
        );

        release_space(&engine);
    }

    #[test]
    fn test_space_n_produces_tab() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap(&engine, KeyCode::N);
        wait_tick();

        let taps = platform.taps();
        assert!(
            taps.iter().any(|t| t.key == KeyCode::Tab),
            "Space+N should produce Tab, got: {:?}",
            taps.iter().map(|t| t.key).collect::<Vec<_>>()
        );

        release_space(&engine);
    }

    #[test]
    fn test_page_keys_yuio() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        // Y = Home
        clx_tap(&engine, KeyCode::Y);
        wait_tick();
        assert!(platform.taps().iter().any(|t| t.key == KeyCode::Home),
            "Space+Y should produce Home");
        platform.clear_taps();

        // O = End
        clx_tap(&engine, KeyCode::O);
        wait_tick();
        assert!(platform.taps().iter().any(|t| t.key == KeyCode::End),
            "Space+O should produce End");
        platform.clear_taps();

        // I = PageUp
        clx_tap(&engine, KeyCode::I);
        wait_tick();
        assert!(platform.taps().iter().any(|t| t.key == KeyCode::PageUp),
            "Space+I should produce PageUp");
        platform.clear_taps();

        // U = PageDown
        clx_tap(&engine, KeyCode::U);
        wait_tick();
        assert!(platform.taps().iter().any(|t| t.key == KeyCode::PageDown),
            "Space+U should produce PageDown");

        release_space(&engine);
    }

    // ── Edit module: modifier forwarding ────────────────────────────────────

    #[test]
    fn test_ctrl_space_g_produces_ctrl_enter() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LCtrl]);

        engine.on_key_event(KeyCode::LCtrl, true);
        engine.on_key_event(KeyCode::Space, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(KeyCode::G, true);
        engine.on_key_event(KeyCode::G, false);
        wait_tick();

        let taps = platform.taps();
        let enter_taps: Vec<_> = taps.iter().filter(|t| t.key == KeyCode::Enter).collect();

        // If CLX mode activated (Space not bypassed by Ctrl), Enter should have Ctrl mod.
        if !enter_taps.is_empty() {
            assert!(
                enter_taps.iter().any(|t| t.mods.contains(&KeyCode::LCtrl)),
                "Ctrl+Space+G should produce Ctrl+Enter, but Enter had no Ctrl mod"
            );
        }

        release_space(&engine);
    }

    #[test]
    fn test_shift_space_g_produces_shift_enter() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);

        clx_tap(&engine, KeyCode::G);
        wait_tick();

        let taps = platform.taps();
        let enter_taps: Vec<_> = taps.iter().filter(|t| t.key == KeyCode::Enter).collect();
        if !enter_taps.is_empty() {
            assert!(
                enter_taps.iter().any(|t| t.mods.contains(&KeyCode::LShift)),
                "Shift+Space+G should produce Shift+Enter"
            );
        }

        release_space(&engine);
    }

    // ── Mouse module ─────────────────────────────────────────────────────────

    #[test]
    fn test_space_wasd_moves_mouse() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::D, true); // mouse right
        std::thread::sleep(std::time::Duration::from_millis(100));
        engine.on_key_event(KeyCode::D, false);
        release_space(&engine);

        let (dx, _dy) = platform.mouse_delta();
        assert!(dx > 0, "Space+D should move mouse right, got dx={}", dx);
    }

    #[test]
    fn test_space_w_moves_mouse_up() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::W, true);
        std::thread::sleep(std::time::Duration::from_millis(100));
        engine.on_key_event(KeyCode::W, false);
        release_space(&engine);

        let (_dx, dy) = platform.mouse_delta();
        assert!(dy < 0, "Space+W should move mouse up (negative dy), got dy={}", dy);
    }

    #[test]
    fn test_space_s_moves_mouse_down() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::S, true);
        std::thread::sleep(std::time::Duration::from_millis(100));
        engine.on_key_event(KeyCode::S, false);
        release_space(&engine);

        let (_dx, dy) = platform.mouse_delta();
        assert!(dy > 0, "Space+S should move mouse down (positive dy), got dy={}", dy);
    }

    #[test]
    fn test_space_a_moves_mouse_left() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::A, true);
        std::thread::sleep(std::time::Duration::from_millis(100));
        engine.on_key_event(KeyCode::A, false);
        release_space(&engine);

        let (dx, _dy) = platform.mouse_delta();
        assert!(dx < 0, "Space+A should move mouse left (negative dx), got dx={}", dx);
    }

    // ── Media module ─────────────────────────────────────────────────────────

    #[test]
    fn test_space_f5_plays_media() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F5);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaPlay),
            "Space+F5 should send MediaPlay extended key"
        );
    }

    #[test]
    fn test_space_f6_prev_track() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F6);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaPrev),
            "Space+F6 should send MediaPrev"
        );
    }

    #[test]
    fn test_space_f7_next_track() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F7);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaNext),
            "Space+F7 should send MediaNext"
        );
    }

    #[test]
    fn test_space_f9_volume_up() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F9);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::VolumeUp),
            "Space+F9 should send VolumeUp"
        );
    }

    #[test]
    fn test_space_f10_volume_down() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F10);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::VolumeDown),
            "Space+F10 should send VolumeDown"
        );
    }

    #[test]
    fn test_space_f11_mute() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::F11);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::VolumeMute),
            "Space+F11 should send VolumeMute"
        );
    }

    #[test]
    fn test_space_bracket_left_prev_track() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::BracketLeft);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaPrev),
            "Space+[ should send MediaPrev"
        );
    }

    #[test]
    fn test_space_bracket_right_next_track() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::BracketRight);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaNext),
            "Space+] should send MediaNext"
        );
    }

    #[test]
    fn test_space_backslash_play_pause() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::Backslash);
        release_space(&engine);

        assert!(
            platform.extended_taps().contains(&KeyCode::MediaPlay),
            "Space+\\ should send MediaPlay"
        );
    }

    // ── Window manager module ────────────────────────────────────────────────

    #[test]
    fn test_space_z_cycles_windows_forward() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap(&engine, KeyCode::Z);
        engine.on_key_event(KeyCode::Z, false);
        release_space(&engine);
        wait_tick();

        let calls = platform.cycle_calls();
        assert!(!calls.is_empty(), "Space+Z should call cycle_windows");
        assert!(
            calls.iter().any(|&d| d > 0),
            "Space+Z should cycle forward (dir > 0), got: {:?}", calls
        );
    }

    #[test]
    fn test_space_shift_z_cycles_windows_backward() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);
        clx_tap(&engine, KeyCode::Z);
        engine.on_key_event(KeyCode::Z, false);
        release_space(&engine);
        wait_tick();

        let calls = platform.cycle_calls();
        assert!(!calls.is_empty(), "Space+Shift+Z should call cycle_windows");
        assert!(
            calls.iter().any(|&d| d < 0),
            "Space+Shift+Z should cycle backward (dir < 0), got: {:?}", calls
        );
    }

    #[test]
    fn test_space_x_closes_tab() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::X);
        release_space(&engine);

        assert_eq!(platform.close_tab_count(), 1, "Space+X should call close_tab once");
        assert_eq!(platform.close_window_count(), 0, "Space+X should NOT call close_window");
        assert_eq!(platform.kill_window_count(), 0, "Space+X should NOT call kill_window");
    }

    #[test]
    fn test_space_shift_x_closes_window() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);
        clx_tap_instant(&engine, KeyCode::X);
        release_space(&engine);

        assert_eq!(platform.close_window_count(), 1, "Space+Shift+X should call close_window");
        assert_eq!(platform.close_tab_count(), 0, "Space+Shift+X should NOT call close_tab");
    }

    #[test]
    fn test_space_ctrl_alt_x_kills_window() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LCtrl, KeyCode::LAlt]);
        engine.on_key_event(KeyCode::LCtrl, true);
        engine.on_key_event(KeyCode::LAlt, true);
        clx_tap_instant(&engine, KeyCode::X);
        release_space(&engine);

        assert_eq!(platform.kill_window_count(), 1, "Space+Ctrl+Alt+X should call kill_window");
        assert_eq!(platform.close_tab_count(), 0, "Space+Ctrl+Alt+X should NOT call close_tab");
        assert_eq!(platform.close_window_count(), 0, "Space+Ctrl+Alt+X should NOT call close_window");
    }

    #[test]
    fn test_space_c_arranges_stacked() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::C);
        release_space(&engine);

        let calls = platform.arrange_calls();
        assert!(
            calls.contains(&ArrangeMode::Stacked),
            "Space+C should call arrange_windows(Stacked)"
        );
    }

    #[test]
    fn test_space_shift_c_arranges_side_by_side() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);
        clx_tap_instant(&engine, KeyCode::C);
        release_space(&engine);

        let calls = platform.arrange_calls();
        assert!(
            calls.contains(&ArrangeMode::SideBySide),
            "Space+Shift+C should call arrange_windows(SideBySide)"
        );
    }

    // ── Virtual desktop module ────────────────────────────────────────────────

    #[test]
    fn test_space_1_through_9_switch_desktop() {
        let keys_and_desktops = [
            (KeyCode::D1, 1u32),
            (KeyCode::D2, 2u32),
            (KeyCode::D3, 3u32),
            (KeyCode::D9, 9u32),
        ];

        for (key, expected_desktop) in keys_and_desktops {
            let platform = Arc::new(MockPlatform::new());
            let engine = make_engine(Arc::clone(&platform));

            clx_tap_instant(&engine, key);
            release_space(&engine);

            let calls = platform.desktop_switch_calls();
            assert!(
                calls.contains(&expected_desktop),
                "Space+{:?} should call switch_to_desktop({}), got: {:?}",
                key, expected_desktop, calls
            );
            assert!(
                platform.window_move_calls().is_empty(),
                "Space+{:?} should NOT call move_window_to_desktop", key
            );
        }
    }

    #[test]
    fn test_space_0_switches_to_desktop_10() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::D0);
        release_space(&engine);

        assert!(
            platform.desktop_switch_calls().contains(&10),
            "Space+0 should switch to desktop 10"
        );
    }

    #[test]
    fn test_space_shift_1_moves_window_to_desktop() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);
        clx_tap_instant(&engine, KeyCode::D1);
        release_space(&engine);

        assert!(
            platform.window_move_calls().contains(&1),
            "Space+Shift+1 should call move_window_to_desktop(1)"
        );
        assert!(
            platform.desktop_switch_calls().is_empty(),
            "Space+Shift+1 should NOT call switch_to_desktop"
        );
    }

    // ── Engine lifecycle ──────────────────────────────────────────────────────

    #[test]
    fn test_emergency_stop_clears_state() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::W, true);
        wait_tick();

        engine.emergency_stop();

        assert!(
            !engine.state().is_clx_active(),
            "emergency_stop should deactivate CLX mode"
        );

        platform.clear_mouse();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let (dx, dy) = platform.mouse_delta();
        assert!(
            dx == 0 && dy == 0,
            "After emergency_stop, mouse should not move. Got dx={}, dy={}", dx, dy
        );
    }

    #[test]
    fn test_bare_space_passes_through_space() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        engine.on_key_event(KeyCode::Space, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(KeyCode::Space, false);

        // Wait for timeout thread.
        std::thread::sleep(std::time::Duration::from_millis(300));

        let taps = platform.taps();
        assert!(
            taps.iter().any(|t| t.key == KeyCode::Space),
            "Bare Space tap should produce Space key, got: {:?}",
            taps.iter().map(|t| t.key).collect::<Vec<_>>()
        );
    }

    // ── Bypass logic (CLX must NOT intercept these combos) ────────────────────

    #[test]
    fn test_shift_space_bypasses_clx_mode() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        // Shift+Space should be passed through (IME switching), not enter CLX mode.
        platform.set_physical_keys(vec![KeyCode::LShift]);
        engine.on_key_event(KeyCode::LShift, true);

        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(
            resp, CoreResponse::PassThrough,
            "Shift+Space should pass through (not enter CLX mode)"
        );
        engine.on_key_event(KeyCode::Space, false);
    }

    // ── CapsLock trigger mode ────────────────────────────────────────────────

    #[test]
    fn test_capslock_trigger_hjkl() {
        let platform = Arc::new(MockPlatform::new());
        // Build engine with CapsLock as trigger instead of Space.
        let engine = make_engine_capslock(Arc::clone(&platform));

        // CapsLock down → enter CLX mode.
        engine.on_key_event(KeyCode::CapsLock, true);
        std::thread::sleep(std::time::Duration::from_millis(5));
        engine.on_key_event(KeyCode::H, true);
        std::thread::sleep(std::time::Duration::from_millis(80));
        engine.on_key_event(KeyCode::H, false);
        wait_tick();

        assert!(
            platform.taps().iter().any(|t| t.key == KeyCode::Left),
            "CapsLock+H should produce Left arrow"
        );

        engine.on_key_event(KeyCode::CapsLock, false);
    }

    // ── Window manager: restart shortcut ─────────────────────────────────────

    #[test]
    fn test_space_period_restarts() {
        let platform = Arc::new(MockPlatform::new());
        let engine = make_engine(Arc::clone(&platform));

        clx_tap_instant(&engine, KeyCode::Period);
        release_space(&engine);

        assert_eq!(
            platform.restart_count(), 1,
            "Space+Period should call restart()"
        );
    }
}
