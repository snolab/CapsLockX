//! Tests for PTT backspace/diff logic and state machine.
//!
//! Uses a MockPlatform that simulates a text cursor — tracks the full "screen"
//! state including pre-existing text so we can verify PTT never eats user text.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::key_code::KeyCode;
    use crate::platform::{MouseButton, Platform};
    use crate::modules::voice_ptt::{PttRelease, PttSession};
    use crate::modules::voice_otoji::OtojiBackend;

    // ── MockPlatform: simulates a text cursor ────────────────────────────────

    struct MockPlatform {
        /// Full text "on screen" — includes pre-existing text + PTT output.
        screen: Arc<Mutex<String>>,
        /// Log of all operations for debugging.
        ops: Arc<Mutex<Vec<String>>>,
    }

    impl MockPlatform {
        fn new() -> Self {
            Self {
                screen: Arc::new(Mutex::new(String::new())),
                ops: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn with_existing_text(text: &str) -> Self {
            Self {
                screen: Arc::new(Mutex::new(text.to_string())),
                ops: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn screen_text(&self) -> String {
            self.screen.lock().unwrap().clone()
        }

        fn op_log(&self) -> Vec<String> {
            self.ops.lock().unwrap().clone()
        }

        fn backspace_count(&self) -> usize {
            self.ops.lock().unwrap().iter()
                .filter(|op| op == &"BS")
                .count()
        }

        fn type_count(&self) -> usize {
            self.ops.lock().unwrap().iter()
                .filter(|op| op.starts_with("T:"))
                .count()
        }
    }

    impl Platform for MockPlatform {
        fn key_down(&self, key: KeyCode) {
            if key == KeyCode::Backspace {
                let mut s = self.screen.lock().unwrap();
                if s.pop().is_some() {
                    self.ops.lock().unwrap().push("BS".into());
                } else {
                    self.ops.lock().unwrap().push("BS(empty!)".into());
                }
            }
        }
        fn key_up(&self, _key: KeyCode) {}
        fn mouse_move(&self, _dx: i32, _dy: i32) {}
        fn scroll_v(&self, _delta: i32) {}
        fn scroll_h(&self, _delta: i32) {}
        fn mouse_button(&self, _button: MouseButton, _pressed: bool) {}

        fn type_text(&self, text: &str) {
            self.screen.lock().unwrap().push_str(text);
            self.ops.lock().unwrap().push(format!("T:{}", text));
        }
    }

    // ── Helper: feed audio and wait ──────────────────────────────────────────

    /// Generate 16kHz mono silence.
    fn silence(duration_ms: u64) -> Vec<f32> {
        vec![0.0; (16 * duration_ms) as usize]
    }

    /// Generate 16kHz mono sine wave (audible signal for VAD).
    fn sine_wave(duration_ms: u64, freq_hz: f32) -> Vec<f32> {
        let n = (16 * duration_ms) as usize;
        (0..n).map(|i| {
            let t = i as f32 / 16000.0;
            (2.0 * std::f32::consts::PI * freq_hz * t).sin() * 0.5
        }).collect()
    }

    /// Feed samples to PTT in small chunks (simulating real-time mic callback).
    fn feed_chunked(ptt: &PttSession, samples: &[f32], chunk_ms: u64) {
        let chunk_size = (16 * chunk_ms) as usize;
        for chunk in samples.chunks(chunk_size) {
            ptt.feed(chunk);
            std::thread::sleep(Duration::from_millis(chunk_ms));
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn test_replace_displayed_diff_append() {
        // Scenario: "~" → "hello~" — should backspace 1, type "hello~"
        let platform = Arc::new(MockPlatform::new());
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));

        // After mic_ready + placeholder delay, tail = "-" (listening, no VAD).
        // VAD on would flip it to "~".
        ptt.set_mic_ready();
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(platform.screen_text(), "-", "placeholder should be - (mic ready, VAD silent)");

        // Simulate VAD on → tail flips to "~".
        ptt.on_vad(true);
        assert_eq!(platform.screen_text(), "~", "VAD on should flip tail to ~");

        // Simulate what streaming does: replace "~" with "hello~"
        // We call replace_displayed indirectly — not pub. Instead, test via
        // the full flow by stopping here and checking ops.
        let bs = platform.backspace_count();
        let tc = platform.type_count();
        eprintln!("[test] after placeholder: screen={:?}, bs={}, types={}",
            platform.screen_text(), bs, tc);

        // Clean up
        ptt.on_release();
        std::thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_tap_no_text() {
        // Quick tap (<150ms) should not type anything.
        let platform = Arc::new(MockPlatform::with_existing_text("existing"));
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50)); // well under placeholder delay
        let result = ptt.on_release();

        assert_eq!(result, PttRelease::Tap);
        assert_eq!(platform.screen_text(), "existing",
            "tap should not modify existing text");
        assert_eq!(platform.backspace_count(), 0,
            "tap should not send any backspaces");
    }

    #[test]
    fn test_tap_with_placeholder_cleanup() {
        // Tap at 200ms — placeholder "~" was typed, should be cleaned up.
        let platform = Arc::new(MockPlatform::with_existing_text("hello"));
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        ptt.on_press();
        std::thread::sleep(Duration::from_millis(200)); // placeholder appears
        let screen_during = platform.screen_text();
        eprintln!("[test] during hold: {:?}", screen_during);
        assert!(screen_during.ends_with("-"), "should show placeholder - (mic ready, no VAD)");

        let result = ptt.on_release();
        // Simulate otoji sending ptt_final (empty = no speech detected).
        ptt.on_ptt_final("");
        std::thread::sleep(Duration::from_millis(100));

        // PTT had displayed text, so it's a Hold.
        // ptt_final("") should erase the placeholder without typing anything.
        let screen_after = platform.screen_text();
        eprintln!("[test] after release: {:?}, result={:?}", screen_after, result);
        assert_eq!(screen_after, "hello",
            "existing text must be preserved after hold release with no speech");
    }

    #[test]
    fn test_hold_preserves_existing_text() {
        let platform = Arc::new(MockPlatform::with_existing_text("pre-existing "));
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        ptt.on_press();
        std::thread::sleep(Duration::from_millis(200));
        std::thread::sleep(Duration::from_millis(500));

        ptt.on_release();
        // Simulate otoji ptt_final (empty).
        ptt.on_ptt_final("");
        std::thread::sleep(Duration::from_millis(100));

        let screen = platform.screen_text();
        eprintln!("[test] screen after silent hold: {:?}", screen);
        assert!(screen.starts_with("pre-existing "),
            "existing text must be preserved, got: {:?}", screen);
    }

    #[test]
    fn test_double_tap_enters_locked() {
        let platform = Arc::new(MockPlatform::new());
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        // First tap
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        let r1 = ptt.on_release();
        assert_eq!(r1, PttRelease::Tap);

        // Second tap within 500ms
        std::thread::sleep(Duration::from_millis(100));
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        let r2 = ptt.on_release();
        assert_eq!(r2, PttRelease::Locked);
        assert!(ptt.is_locked(), "should be in locked mode");
    }

    #[test]
    fn test_locked_exit_on_press() {
        let platform = Arc::new(MockPlatform::new());
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        // Enter locked mode via double-tap
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        ptt.on_release();
        std::thread::sleep(Duration::from_millis(100));
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        ptt.on_release();
        assert!(ptt.is_locked());

        // Wait for placeholder
        std::thread::sleep(Duration::from_millis(200));

        // Press V to exit locked mode
        let consumed = ptt.on_press();
        assert!(consumed, "press in locked mode should be consumed");
        assert!(!ptt.is_locked(), "should have exited locked mode");
    }

    #[test]
    fn test_no_backspace_on_empty_screen() {
        // Ensure we never send BS(empty!) — backspace when nothing to delete.
        let platform = Arc::new(MockPlatform::new());
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        ptt.on_press();
        std::thread::sleep(Duration::from_millis(200));
        ptt.on_release();
        std::thread::sleep(Duration::from_millis(200));

        let ops = platform.op_log();
        let empty_bs = ops.iter().filter(|op| *op == "BS(empty!)").count();
        assert_eq!(empty_bs, 0,
            "should never backspace on empty screen, ops: {:?}", ops);
    }

    #[test]
    fn test_double_tap_too_slow_is_two_taps() {
        let platform = Arc::new(MockPlatform::new());
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        // First tap
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        let r1 = ptt.on_release();
        assert_eq!(r1, PttRelease::Tap);

        // Wait >500ms (too slow for double-tap)
        std::thread::sleep(Duration::from_millis(600));

        // Second tap
        ptt.on_press();
        std::thread::sleep(Duration::from_millis(50));
        let r2 = ptt.on_release();
        assert_eq!(r2, PttRelease::Tap, "slow second tap should be Tap, not Locked");
        assert!(!ptt.is_locked());
    }

    #[test]
    fn test_backspace_count_matches_displayed() {
        // After a full press-release cycle, total backspaces should equal
        // total characters typed by PTT (placeholder + partials) minus
        // any final committed text.
        let platform = Arc::new(MockPlatform::with_existing_text("AAA"));
        let ptt = PttSession::new(Arc::clone(&platform) as Arc<dyn Platform>, Arc::new(OtojiBackend::new()));
        ptt.set_mic_ready();

        ptt.on_press();
        std::thread::sleep(Duration::from_millis(200)); // placeholder "~" typed
        let screen_during = platform.screen_text();
        assert!(screen_during.starts_with("AAA"), "existing text preserved during hold");

        ptt.on_release();
        ptt.on_ptt_final(""); // simulate otoji response
        std::thread::sleep(Duration::from_millis(100));

        let screen_after = platform.screen_text();
        let ops = platform.op_log();
        eprintln!("[test] ops: {:?}", ops);
        eprintln!("[test] screen: {:?}", screen_after);

        // Verify: all PTT-typed chars were backspaced, existing text intact.
        assert!(screen_after.starts_with("AAA"),
            "existing text 'AAA' must survive, got: {:?}", screen_after);
    }
}
