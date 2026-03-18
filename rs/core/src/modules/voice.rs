/// CLX-Voice – V key toggles voice listening (toggle / hold modes).
///
/// State machine:
///   - on_key_down(V): Idle→Listening (start), Listening→Idle (toggle off)
///   - on_key_up(V):   if held >300ms → stop (hold mode); else keep listening (toggle mode)
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::key_code::KeyCode;
use crate::platform::Platform;

const STATE_IDLE: u8 = 0;
const STATE_LISTENING: u8 = 1;
#[allow(dead_code)]
const STATE_STOPPING: u8 = 2;

/// Hold threshold: if V is held longer than this, releasing V stops listening.
const HOLD_THRESHOLD_MS: u128 = 300;

pub struct VoiceModule {
    state: Arc<AtomicU8>,
    press_time: Mutex<Option<Instant>>,
    #[allow(dead_code)]
    platform: Arc<dyn Platform>,
}

impl VoiceModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STATE_IDLE)),
            press_time: Mutex::new(None),
            platform,
        }
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        let current = self.state.load(Ordering::Relaxed);
        match current {
            STATE_IDLE => {
                self.state.store(STATE_LISTENING, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = Some(Instant::now());
                start_listening();
                true
            }
            STATE_LISTENING => {
                self.state.store(STATE_IDLE, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = None;
                stop_listening();
                true
            }
            _ => true,
        }
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }

        let current = self.state.load(Ordering::Relaxed);
        if current == STATE_LISTENING {
            let held_long = self
                .press_time
                .lock()
                .unwrap()
                .map(|t| t.elapsed().as_millis() >= HOLD_THRESHOLD_MS)
                .unwrap_or(false);

            if held_long {
                // Hold mode: release stops listening.
                self.state.store(STATE_IDLE, Ordering::Relaxed);
                *self.press_time.lock().unwrap() = None;
                stop_listening();
            }
            // Toggle mode (<300ms): keep listening, do nothing.
        }
        true
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    /// Called when CLX mode deactivates – ensure we stop listening.
    pub fn stop(&self) {
        let prev = self.state.swap(STATE_IDLE, Ordering::Relaxed);
        *self.press_time.lock().unwrap() = None;
        if prev == STATE_LISTENING {
            stop_listening();
        }
    }

    pub fn is_listening(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STATE_LISTENING
    }
}

fn start_listening() {
    eprintln!("[CLX] voice: start listening");
}

fn stop_listening() {
    eprintln!("[CLX] voice: stop listening");
}
