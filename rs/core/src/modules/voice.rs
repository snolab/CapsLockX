//! CLX+V — hand voice input to the otoji app.
//!
//! CapsLockX no longer records, transcribes, polishes or speaks anything:
//! all of that lives in otoji (https://github.com/snomiao/otoji). This module
//! only turns the CLX+V gesture into an otoji command and delivers it through
//! [`Platform::otoji_send`]:
//!
//! - **tap** (released within [`HOLD_THRESHOLD`]) → `toggle` (otoji on/off)
//! - **hold** → `ptt-start` once the threshold passes, `ptt-end` on release
//!
//! Delivery can block for seconds (the adapter may have to launch otoji), so
//! a single worker thread owns it; the hook thread only posts key edges. One
//! thread also keeps `ptt-start` / `ptt-end` in order.
//!
//! When otoji is missing the user gets a short overlay instead: an install
//! hint on macOS (a second CLX+V while it is showing opens the download page),
//! or "not available yet" where otoji doesn't ship.

use crate::key_code::KeyCode;
use crate::platform::{OtojiCommand, OtojiSendResult, Platform};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A press held at least this long is push-to-talk; shorter is a tap.
pub const HOLD_THRESHOLD: Duration = Duration::from_millis(300);

/// Where to get otoji.
pub const OTOJI_INSTALL_URL: &str = "https://github.com/snomiao/otoji/releases";

/// How long the "install otoji" hint stays up; a CLX+V inside this window
/// opens [`OTOJI_INSTALL_URL`].
const HINT_DURATION: Duration = Duration::from_secs(6);

enum Edge {
    Down,
    Up,
}

pub struct VoiceModule {
    platform: Arc<dyn Platform>,
    held: Mutex<bool>,
    /// Lazily started worker (keeps `Modules::new` free of thread spawns that
    /// only matter once the user actually presses CLX+V).
    tx: Mutex<Option<Sender<Edge>>>,
    /// When the install hint was last shown (shared with the worker).
    hint_shown: Arc<Mutex<Option<Instant>>>,
}

impl VoiceModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self {
            platform,
            held: Mutex::new(false),
            tx: Mutex::new(None),
            hint_shown: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        key == KeyCode::V
    }

    pub fn on_key_down(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }
        let mut held = self.held.lock().unwrap();
        if !*held {
            // Auto-repeat downs are swallowed by the `held` latch.
            *held = true;
            self.post(Edge::Down);
        }
        true
    }

    pub fn on_key_up(&self, key: KeyCode) -> bool {
        if key != KeyCode::V {
            return false;
        }
        self.release();
        true
    }

    /// CLX mode ended (trigger released) — the V key-up may never reach us.
    pub fn stop(&self) {
        self.release();
    }

    fn release(&self) {
        let mut held = self.held.lock().unwrap();
        if *held {
            *held = false;
            self.post(Edge::Up);
        }
    }

    fn post(&self, edge: Edge) {
        let mut slot = self.tx.lock().unwrap();
        if slot.is_none() {
            let (tx, rx) = mpsc::channel();
            let platform = Arc::clone(&self.platform);
            let hint = Arc::clone(&self.hint_shown);
            let spawned = std::thread::Builder::new()
                .name("clx-voice-otoji".into())
                .spawn(move || worker(rx, platform, hint));
            if spawned.is_err() {
                eprintln!("[CLX] voice: failed to start otoji worker");
                return;
            }
            *slot = Some(tx);
        }
        if let Some(tx) = slot.as_ref() {
            let _ = tx.send(edge);
        }
    }
}

/// Turns key edges into otoji commands (tap vs. hold) and delivers them.
fn worker(rx: Receiver<Edge>, platform: Arc<dyn Platform>, hint: Arc<Mutex<Option<Instant>>>) {
    while let Ok(edge) = rx.recv() {
        if !matches!(edge, Edge::Down) {
            continue; // stray release
        }
        match rx.recv_timeout(HOLD_THRESHOLD) {
            Ok(_) => {
                // Released quickly: a tap.
                deliver(&platform, &hint, OtojiCommand::Toggle);
            }
            Err(RecvTimeoutError::Timeout) => {
                // Still held: push-to-talk until the release arrives.
                let reachable = deliver(&platform, &hint, OtojiCommand::PttStart);
                loop {
                    match rx.recv() {
                        Ok(Edge::Up) => break,
                        Ok(Edge::Down) => continue,
                        Err(_) => return,
                    }
                }
                if reachable {
                    deliver(&platform, &hint, OtojiCommand::PttEnd);
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// Send one command and surface failures. Returns whether otoji received it.
fn deliver(platform: &Arc<dyn Platform>, hint: &Mutex<Option<Instant>>, cmd: OtojiCommand) -> bool {
    let result = platform.otoji_send(cmd);
    eprintln!("[CLX] voice: otoji {} → {:?}", cmd.as_str(), result);
    match result {
        OtojiSendResult::Sent | OtojiSendResult::Launched => {
            *hint.lock().unwrap() = None;
            true
        }
        OtojiSendResult::NotInstalled => {
            let mut shown = hint.lock().unwrap();
            if shown.is_some_and(|t| t.elapsed() < HINT_DURATION) {
                // Second CLX+V while the hint is up: take them to the download.
                *shown = None;
                drop(shown);
                platform.hide_brainstorm_overlay();
                platform.open_url(OTOJI_INSTALL_URL);
            } else {
                *shown = Some(Instant::now());
                drop(shown);
                flash(
                    platform,
                    &format!(
                        "🎙 Voice input now lives in otoji.\n\n\
                         otoji isn't installed. Get it from:\n{OTOJI_INSTALL_URL}\n\n\
                         Press CLX+V again to open the download page."
                    ),
                    HINT_DURATION,
                );
            }
            false
        }
        OtojiSendResult::NoResponse => {
            flash(
                platform,
                "🎙 otoji didn't respond.\n\nCLX+V needs an otoji version with the tray \
                 control socket — please update otoji.",
                Duration::from_secs(5),
            );
            false
        }
        OtojiSendResult::Unsupported => {
            flash(
                platform,
                "🎙 Voice input (CLX+V) isn't available on this platform yet.\n\
                 It moved to otoji, which currently ships for macOS only.",
                Duration::from_secs(4),
            );
            false
        }
    }
}

/// Bumped per flash so an older message's timer doesn't hide a newer one.
static FLASH_GEN: AtomicU64 = AtomicU64::new(0);

/// Show a message in the overlay, then hide it after `dur` (off-thread).
/// Adapters without an overlay make this a no-op, so also log it.
fn flash(platform: &Arc<dyn Platform>, text: &str, dur: Duration) {
    eprintln!("[CLX] voice: {}", text.replace('\n', " "));
    platform.show_brainstorm_overlay(text);
    let gen = FLASH_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let platform = Arc::clone(platform);
    let _ = std::thread::Builder::new()
        .name("clx-voice-flash".into())
        .spawn(move || {
            std::thread::sleep(dur);
            if FLASH_GEN.load(Ordering::Relaxed) == gen {
                platform.hide_brainstorm_overlay();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    fn otoji_calls(mock: &MockPlatform) -> Vec<OtojiCommand> {
        mock.calls()
            .into_iter()
            .filter_map(|c| match c {
                Call::OtojiSend(cmd) => Some(cmd),
                _ => None,
            })
            .collect()
    }

    fn wait_for(mock: &MockPlatform, n: usize) -> Vec<OtojiCommand> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let got = otoji_calls(mock);
            if got.len() >= n || Instant::now() > deadline {
                return got;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn only_v_is_mapped() {
        let m = VoiceModule::new(Arc::new(MockPlatform::new()));
        assert!(m.is_mapped_key(KeyCode::V));
        assert!(!m.is_mapped_key(KeyCode::B));
        assert!(!m.on_key_down(KeyCode::B));
        assert!(!m.on_key_up(KeyCode::B));
    }

    #[test]
    fn tap_toggles() {
        let mock = Arc::new(MockPlatform::new());
        let m = VoiceModule::new(mock.clone());
        assert!(m.on_key_down(KeyCode::V));
        assert!(m.on_key_up(KeyCode::V));
        assert_eq!(wait_for(&mock, 1), vec![OtojiCommand::Toggle]);
    }

    #[test]
    fn hold_is_push_to_talk() {
        let mock = Arc::new(MockPlatform::new());
        let m = VoiceModule::new(mock.clone());
        m.on_key_down(KeyCode::V);
        // Auto-repeat while held must not restart anything.
        m.on_key_down(KeyCode::V);
        std::thread::sleep(HOLD_THRESHOLD + Duration::from_millis(150));
        assert_eq!(otoji_calls(&mock), vec![OtojiCommand::PttStart]);
        m.on_key_up(KeyCode::V);
        assert_eq!(
            wait_for(&mock, 2),
            vec![OtojiCommand::PttStart, OtojiCommand::PttEnd]
        );
    }

    #[test]
    fn stop_releases_a_held_v() {
        let mock = Arc::new(MockPlatform::new());
        let m = VoiceModule::new(mock.clone());
        m.on_key_down(KeyCode::V);
        m.stop();
        assert_eq!(wait_for(&mock, 1), vec![OtojiCommand::Toggle]);
        // A late key-up after stop() is a no-op.
        m.on_key_up(KeyCode::V);
        std::thread::sleep(HOLD_THRESHOLD + Duration::from_millis(100));
        assert_eq!(otoji_calls(&mock), vec![OtojiCommand::Toggle]);
    }

    #[test]
    fn unsupported_platform_says_not_available() {
        let mock = Arc::new(MockPlatform::new());
        *mock.otoji_result.lock().unwrap() = OtojiSendResult::Unsupported;
        let p: Arc<dyn Platform> = mock.clone();
        deliver(&p, &Mutex::new(None), OtojiCommand::Toggle);
        assert!(mock
            .calls()
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains("isn't available"))));
    }

    #[test]
    fn not_installed_hints_then_opens_download_page() {
        let mock = Arc::new(MockPlatform::new());
        *mock.otoji_result.lock().unwrap() = OtojiSendResult::NotInstalled;
        let hint = Mutex::new(None);
        let p: Arc<dyn Platform> = mock.clone();
        deliver(&p, &hint, OtojiCommand::Toggle);
        assert!(mock
            .calls()
            .iter()
            .any(|c| matches!(c, Call::ShowBrainstormOverlay(s) if s.contains(OTOJI_INSTALL_URL))));
        assert!(!mock.calls().iter().any(|c| matches!(c, Call::OpenUrl(_))));
        deliver(&p, &hint, OtojiCommand::Toggle);
        assert!(mock
            .calls()
            .iter()
            .any(|c| matches!(c, Call::OpenUrl(u) if u == OTOJI_INSTALL_URL)));
    }

    #[test]
    fn failed_ptt_start_skips_ptt_end() {
        let mock = Arc::new(MockPlatform::new());
        *mock.otoji_result.lock().unwrap() = OtojiSendResult::Unsupported;
        let m = VoiceModule::new(mock.clone());
        m.on_key_down(KeyCode::V);
        std::thread::sleep(HOLD_THRESHOLD + Duration::from_millis(150));
        m.on_key_up(KeyCode::V);
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(otoji_calls(&mock), vec![OtojiCommand::PttStart]);
    }
}
