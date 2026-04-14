//! Push-to-talk voice input — event-driven via otoji subprocess.
//!
//! All STT runs in the otoji process. CLX sends SIGUSR1/SIGUSR2 to otoji
//! to mark PTT segment boundaries. Otoji emits `ptt_partial` and `ptt_final`
//! JSON events which the otoji reader thread dispatches to PttSession.
//!
//! Visual feedback at cursor:
//!   `-`       mic stream starting (not ready yet)
//!   `~`       mic ready, audio flowing (safe to speak)
//!   `text~`   partial (in-progress) transcription from otoji
//!   `text`    final committed transcription (~ removed)
//!
//! Modes:
//!   Hold      — press V to start, release to commit
//!   Locked    — double-tap V to enter, tap V again to commit + exit

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::key_code::KeyCode;
use crate::platform::Platform;

/// Delay before showing any placeholder — quick taps (<150ms) stay invisible.
const PLACEHOLDER_DELAY_MS: u64 = 150;

/// Max interval between two taps to count as double-click (ms).
const DOUBLE_TAP_MS: u64 = 500;

/// Result of `on_release`.
#[derive(Debug, PartialEq)]
pub enum PttRelease {
    /// Quick tap — caller should handle note-toggle.
    Tap,
    /// Hold released — PTT committed text.
    Hold,
    /// Double-tap detected or already in locked mode.
    Locked,
}

// ── PttSession ───────────────────────────────────────────────────────────────

pub struct PttSession {
    platform: Arc<dyn Platform>,
    /// Currently displayed text at cursor (placeholder, partial, or final).
    displayed: Arc<Mutex<String>>,
    /// Monotonic token — cancels stale deferred tasks.
    token_counter: AtomicU64,
    /// True when locked PTT mode is active (double-tap to enter).
    locked: AtomicBool,
    /// Timestamp of last tap release — for double-click detection.
    last_tap_time: Mutex<Option<Instant>>,
    /// True once otoji's mic is ready (first status/partial received).
    mic_ready: Arc<AtomicBool>,
    /// Signals that a PTT session is active (between press and release/commit).
    ptt_active: AtomicBool,
    /// Shared reference to OtojiBackend for getting pid.
    otoji: Arc<super::voice_otoji::OtojiBackend>,
}

impl PttSession {
    pub fn new(platform: Arc<dyn Platform>, otoji: Arc<super::voice_otoji::OtojiBackend>) -> Arc<Self> {
        Arc::new(Self {
            platform,
            displayed: Arc::new(Mutex::new(String::new())),
            token_counter: AtomicU64::new(0),
            locked: AtomicBool::new(false),
            last_tap_time: Mutex::new(None),
            mic_ready: Arc::new(AtomicBool::new(false)),
            ptt_active: AtomicBool::new(false),
            otoji,
        })
    }

    /// Mark mic as ready (called when otoji sends first status/partial).
    pub fn set_mic_ready(&self) {
        self.mic_ready.store(true, Ordering::Relaxed);
    }

    /// Feed is no longer needed — otoji handles audio internally.
    /// Kept as a no-op for backward compatibility during transition.
    pub fn feed(&self, _samples_16k: &[f32]) {
        // Set mic_ready on first non-empty feed (otoji's cpal callback is running).
        if !self.mic_ready.swap(true, Ordering::Relaxed) {
            eprintln!("[CLX] PTT: mic ready (via feed)");
        }
    }

    /// V key pressed. Returns `true` if consumed (lock-exit).
    pub fn on_press(self: &Arc<Self>) -> bool {
        if self.locked.load(Ordering::Relaxed) {
            eprintln!("[CLX] PTT: V pressed while locked → committing & unlocking");
            self.locked.store(false, Ordering::Relaxed);
            // Send SIGUSR2 to otoji → triggers ptt_final event.
            self.send_ptt_end();
            // Don't clear displayed here — ptt_final handler will do it.
            return true;
        }

        let was_ready = self.mic_ready.load(Ordering::Relaxed);
        let token = self.token_counter.fetch_add(1, Ordering::Relaxed) + 1;
        *self.displayed.lock().unwrap() = String::new();
        self.ptt_active.store(true, Ordering::Relaxed);

        // Try sending SIGUSR1 now. If otoji isn't running yet, the deferred
        // thread will retry after mic_ready confirms otoji is live.
        let sent_now = self.send_ptt_start();

        // Deferred placeholder + SIGUSR1 retry.
        let this = Arc::clone(self);
        std::thread::Builder::new()
            .name("ptt-init".into())
            .spawn(move || {
                // Wait for placeholder delay.
                std::thread::sleep(Duration::from_millis(PLACEHOLDER_DELAY_MS));
                if this.token_counter.load(Ordering::Relaxed) != token { return; }

                // Wait for mic_ready (= otoji is running and streaming).
                if !(was_ready || this.mic_ready.load(Ordering::Relaxed)) {
                    this.replace_displayed("-");
                    for _ in 0..100 { // up to 5s
                        std::thread::sleep(Duration::from_millis(50));
                        if this.token_counter.load(Ordering::Relaxed) != token { return; }
                        if this.mic_ready.load(Ordering::Relaxed) { break; }
                    }
                }
                if this.token_counter.load(Ordering::Relaxed) != token { return; }

                // Retry SIGUSR1 if it failed on press (otoji wasn't ready).
                if !sent_now {
                    this.send_ptt_start();
                }
                this.replace_displayed("~");
            })
            .ok();
        false
    }

    /// Whether locked PTT mode is active.
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    /// V key released.
    pub fn on_release(self: &Arc<Self>) -> PttRelease {
        if self.locked.load(Ordering::Relaxed) {
            return PttRelease::Locked;
        }

        self.token_counter.fetch_add(1, Ordering::Relaxed);

        let displayed = self.displayed.lock().unwrap().clone();

        if displayed.is_empty() {
            // Quick tap — check for double-click.
            self.ptt_active.store(false, Ordering::Relaxed);
            self.send_ptt_end(); // cancel PTT segment in otoji

            let mut last = self.last_tap_time.lock().unwrap();
            let is_double = last
                .map(|t| t.elapsed().as_millis() < DOUBLE_TAP_MS as u128)
                .unwrap_or(false);
            if is_double {
                *last = None;
                eprintln!("[CLX] PTT: double-tap → entering locked mode");
                self.locked.store(true, Ordering::Relaxed);
                self.ptt_active.store(true, Ordering::Relaxed);
                *self.displayed.lock().unwrap() = String::new();
                // Start new PTT segment for locked mode.
                self.send_ptt_start();
                let this = Arc::clone(self);
                let token = self.token_counter.fetch_add(1, Ordering::Relaxed) + 1;
                std::thread::Builder::new()
                    .name("ptt-lock-init".into())
                    .spawn(move || {
                        std::thread::sleep(Duration::from_millis(PLACEHOLDER_DELAY_MS));
                        if this.token_counter.load(Ordering::Relaxed) != token { return; }
                        this.replace_displayed("~");
                    })
                    .ok();
                return PttRelease::Locked;
            }
            *last = Some(Instant::now());
            return PttRelease::Tap;
        }

        // Had displayed text → hold release. Send SIGUSR2 → otoji will emit ptt_final.
        *self.last_tap_time.lock().unwrap() = None;
        self.send_ptt_end();
        // Don't clear displayed yet — on_ptt_final will handle it.
        PttRelease::Hold
    }

    // ── otoji event handlers (called from otoji reader thread) ───────────

    /// Called when otoji emits a `ptt_partial` event.
    pub fn on_ptt_partial(&self, text: &str) {
        if !self.ptt_active.load(Ordering::Relaxed) { return; }
        if text.is_empty() { return; }
        let partial = format!("{}~", text);
        self.replace_displayed(&partial);
    }

    /// Called when otoji emits a `ptt_final` event.
    pub fn on_ptt_final(&self, text: &str) {
        let text = text.trim();
        let old = {
            let mut d = self.displayed.lock().unwrap();
            std::mem::take(&mut *d)
        };
        for _ in old.chars() {
            self.platform.key_tap(KeyCode::Backspace);
        }
        if !text.is_empty() {
            eprintln!("[CLX] PTT: final -> {:?}", text);
            self.platform.type_text(text);
        }

        // If locked mode, start a new PTT segment for the next utterance.
        if self.locked.load(Ordering::Relaxed) {
            self.send_ptt_start();
        } else {
            self.ptt_active.store(false, Ordering::Relaxed);
        }
    }

    // ── Internal ─────────────────────────────────────────────────────────

    /// Send SIGUSR1 to otoji. Returns true if sent, false if no pid available.
    fn send_ptt_start(&self) -> bool {
        if let Some(pid) = self.otoji.pid() {
            eprintln!("[CLX] PTT: sending SIGUSR1 (ptt_start) to otoji pid={pid}");
            Self::send_signal(pid, 10); // SIGUSR1
            true
        } else {
            eprintln!("[CLX] PTT: no otoji pid available for ptt_start (will retry)");
            false
        }
    }

    fn send_ptt_end(&self) {
        if let Some(pid) = self.otoji.pid() {
            eprintln!("[CLX] PTT: sending SIGUSR2 (ptt_end) to otoji pid={pid}");
            Self::send_signal(pid, 12); // SIGUSR2
        } else {
            eprintln!("[CLX] PTT: no otoji pid available for ptt_end");
        }
    }

    fn send_signal(pid: u32, sig: i32) {
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
            fn __error() -> *mut i32; // macOS errno
        }
        let ret = unsafe { kill(pid as i32, sig) };
        if ret != 0 {
            let errno = unsafe { *__error() };
            eprintln!("[CLX] PTT: kill({pid}, {sig}) FAILED: ret={ret}, errno={errno}");
        }
    }

    /// Replace displayed text at cursor using diff (common prefix optimization).
    fn replace_displayed(&self, new_text: &str) {
        let mut displayed = self.displayed.lock().unwrap();
        let old = &**displayed;

        let common: usize = old.chars().zip(new_text.chars())
            .take_while(|(a, b)| a == b)
            .count();

        let old_tail_chars = old.chars().count() - common;
        let new_tail: String = new_text.chars().skip(common).collect();

        for _ in 0..old_tail_chars {
            self.platform.key_tap(KeyCode::Backspace);
        }
        if !new_tail.is_empty() {
            self.platform.type_text(&new_tail);
        }

        *displayed = new_text.to_string();
    }
}
