//! Wake-word listener — spawns `otoji kws` as an always-on child process
//! and reacts to `{"type":"wake"}` JSON lines by enabling agent mode and
//! starting a PTT segment.
//!
//! Mic ownership stays in otoji: CLX never opens cpal itself for this
//! path. The KWS otoji and the sensevoice otoji run side-by-side —
//! CoreAudio allows multiple shared readers on the same input device.
//!
//! Config flows in from `ClxConfig.wake_word_*` (set via the Preferences
//! panel). Env-var fallback is supported for headless / dev use:
//!   OTOJI_KWS_DIR            — path to sherpa KWS model dir
//!   OTOJI_KWS_KEYWORDS       — path to BPE keywords.txt
//!   OTOJI_KWS_THRESHOLD      — detection threshold (default 0.25)
//!   OTOJI_KWS_HOLD_MS        — fixed hold duration after wake (default 5000)
//!
//! When `wake_word_enabled=false` or either path is empty the listener is
//! a no-op — wake-word is strictly opt-in.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

use super::voice_ptt::PttSession;

/// Maximum time we keep PTT held after a wake event, even if the user
/// keeps talking. Acts as a safety cap so a stuck VAD or background noise
/// can't pin the mic open forever. The watchdog usually releases earlier
/// once VAD silence is observed.
const DEFAULT_HOLD_MS: u64 = 8000;

/// How long VAD must report `active=false` continuously before we
/// auto-release the PTT segment. 1.2s matches Siri/Alexa endpointing.
const SILENCE_RELEASE_MS: u64 = 1200;

/// Grace window after wake fires during which we ignore VAD silence —
/// covers the gap between wake detection and the user starting their
/// command. Without this the watchdog would release immediately.
const POST_WAKE_GRACE_MS: u64 = 600;

// ── Shared VAD state ────────────────────────────────────────────────────
//
// `voice_otoji` writes here on every `vad` event from otoji. The wake-word
// release watchdog reads from here. Shared globals are the simplest way
// to bridge two independent threads (otoji reader + wake watchdog) without
// threading a channel through the module surface.

static VAD_EPOCH: Lazy<Instant> = Lazy::new(Instant::now);
static VAD_ACTIVE: AtomicBool = AtomicBool::new(false);
static VAD_LAST_TRANSITION_MS: AtomicU64 = AtomicU64::new(0);

/// Called by `voice_otoji` whenever an otoji `vad` JSON event arrives.
/// Writes the latest active flag + timestamp into shared atomics so the
/// wake-word release watchdog can poll them.
pub fn note_vad(active: bool) {
    let prev = VAD_ACTIVE.swap(active, Ordering::Relaxed);
    if prev != active {
        let ms = VAD_EPOCH.elapsed().as_millis() as u64;
        VAD_LAST_TRANSITION_MS.store(ms, Ordering::Relaxed);
    }
}

/// Path used to locate otoji on disk. Must live next to `clx`.
fn otoji_bin() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("otoji");
            if p.is_file() { return p; }
            // `./clx` is sibling of `./lib/otoji/target/release/otoji` during dev.
            let dev = dir.join("lib/otoji/target/release/otoji");
            if dev.is_file() { return dev; }
        }
    }
    std::path::PathBuf::from("otoji")
}

/// Config slice consumed by the listener. Mirrors the `wake_word_*` fields
/// in `ClxConfig`. Kept here (instead of importing `ClxConfig`) so the
/// wake_word module can be unit-tested without a full engine state.
#[derive(Clone, Debug)]
pub struct WakeWordConfig {
    pub enabled: bool,
    pub model_dir: String,
    pub keywords_file: String,
    pub threshold: f32,
    pub hold_ms: u64,
}

impl WakeWordConfig {
    /// Build a config from env vars. Used as a fallback when prefs haven't
    /// been populated (e.g. CI / first run).
    pub fn from_env() -> Self {
        Self {
            enabled: true,
            model_dir: std::env::var("OTOJI_KWS_DIR").unwrap_or_default(),
            keywords_file: std::env::var("OTOJI_KWS_KEYWORDS").unwrap_or_default(),
            threshold: std::env::var("OTOJI_KWS_THRESHOLD")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(0.25),
            hold_ms: std::env::var("OTOJI_KWS_HOLD_MS")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_HOLD_MS),
        }
    }
}

pub struct WakeWordListener {
    child: Mutex<Option<Child>>,
    stop: Arc<AtomicBool>,
}

impl WakeWordListener {
    /// Spawn the KWS child + reader thread. Returns `None` when wake-word
    /// is disabled or paths are missing — caller treats this as a no-op.
    pub fn try_start(ptt: Arc<PttSession>, cfg: WakeWordConfig) -> Option<Self> {
        // Env-var override — lets a user flip wake-word on without touching
        // prefs, useful for dev builds and CI.
        let cfg = if !cfg.enabled && std::env::var("OTOJI_KWS_DIR").is_ok() {
            WakeWordConfig::from_env()
        } else {
            cfg
        };
        if !cfg.enabled { return None; }
        let model = if cfg.model_dir.is_empty() {
            std::env::var("OTOJI_KWS_DIR").ok()?
        } else { cfg.model_dir };
        let keywords = if cfg.keywords_file.is_empty() {
            std::env::var("OTOJI_KWS_KEYWORDS").ok()?
        } else { cfg.keywords_file };
        if !std::path::Path::new(&model).is_dir() {
            eprintln!("[CLX] wake-word: model dir not found, skipping ({model})");
            return None;
        }
        if !std::path::Path::new(&keywords).is_file() {
            eprintln!("[CLX] wake-word: keywords file not found, skipping ({keywords})");
            return None;
        }
        let threshold = cfg.threshold;
        let hold_ms = cfg.hold_ms;

        let bin = otoji_bin();
        let mut cmd = Command::new(&bin);
        cmd.arg("kws")
            .arg("--model").arg(&model)
            .arg("--keywords").arg(&keywords)
            .arg("--threshold").arg(threshold.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[CLX] wake-word: spawn {:?}: {}", bin, e);
                return None;
            }
        };
        let stdout = child.stdout.take()?;
        eprintln!(
            "[CLX] wake-word: listener up (model={}, keywords={}, threshold={}, hold={}ms)",
            model, keywords, threshold, hold_ms
        );

        let stop = Arc::new(AtomicBool::new(false));
        let stop_r = Arc::clone(&stop);
        let ptt_r = Arc::clone(&ptt);
        std::thread::Builder::new()
            .name("clx-wake-word".into())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if stop_r.load(Ordering::Relaxed) { break; }
                    let line = match line {
                        Ok(l) => l,
                        Err(_) => break,
                    };
                    if !line.contains("\"type\":\"wake\"") { continue; }
                    let keyword = extract_str(&line, "keyword").unwrap_or_default();
                    eprintln!("[CLX] wake-word: detected \"{}\"", keyword);
                    trigger_wake(Arc::clone(&ptt_r), hold_ms);
                }
                eprintln!("[CLX] wake-word: reader thread exited");
            })
            .ok();

        Some(Self {
            child: Mutex::new(Some(child)),
            stop,
        })
    }

    /// Stop the KWS child. Safe to call multiple times.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(mut c) = self.child.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for WakeWordListener {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Fire the wake reaction: enable agent mode, press PTT, then watch VAD
/// for end-of-utterance and release. `max_hold_ms` caps total hold time
/// in case VAD never goes silent (background noise, speaker noise).
fn trigger_wake(ptt: Arc<PttSession>, max_hold_ms: u64) {
    super::agent::enable_agent_mode();
    let _ = ptt.on_press();
    let ptt_rel = Arc::clone(&ptt);
    std::thread::Builder::new()
        .name("clx-wake-release".into())
        .spawn(move || {
            release_when_silent(ptt_rel, max_hold_ms);
        })
        .ok();
}

/// Poll the shared VAD atomics until either:
///   - VAD has reported silence for `SILENCE_RELEASE_MS` continuously, or
///   - the safety cap `max_hold_ms` is hit.
/// During the first `POST_WAKE_GRACE_MS` we don't release even if silent —
/// gives the user time to start speaking after the wake word.
fn release_when_silent(ptt: Arc<PttSession>, max_hold_ms: u64) {
    let _ = *VAD_EPOCH; // touch lazy to ensure epoch is set
    let start = Instant::now();
    loop {
        std::thread::sleep(Duration::from_millis(80));
        let elapsed = start.elapsed().as_millis() as u64;
        if elapsed >= max_hold_ms {
            eprintln!("[CLX] wake-word: max_hold_ms cap reached ({max_hold_ms}ms), releasing");
            break;
        }
        if elapsed < POST_WAKE_GRACE_MS {
            continue;
        }
        let active = VAD_ACTIVE.load(Ordering::Relaxed);
        if active {
            continue;
        }
        let now_ms = VAD_EPOCH.elapsed().as_millis() as u64;
        let last_transition = VAD_LAST_TRANSITION_MS.load(Ordering::Relaxed);
        let silent_for = now_ms.saturating_sub(last_transition);
        if silent_for >= SILENCE_RELEASE_MS {
            eprintln!("[CLX] wake-word: VAD silent {silent_for}ms, releasing");
            break;
        }
    }
    let _ = ptt.on_release();
}

/// Minimal JSON string extractor — avoids pulling serde into core's hot path.
fn extract_str(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", key);
    let start = line.find(&needle)? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}
