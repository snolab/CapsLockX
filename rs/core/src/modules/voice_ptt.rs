//! Push-to-talk voice input — event-driven via otoji subprocess.
//!
//! All STT runs in the otoji process. CLX sends SIGUSR1/SIGUSR2 to otoji
//! to mark PTT segment boundaries. Otoji emits `ptt_partial` and `ptt_final`
//! JSON events which the otoji reader thread dispatches to PttSession.
//!
//! Visual feedback at cursor (tail glyph):
//!   `.`       mic stream starting (otoji not ready yet)
//!   `-`       mic ready, VAD silent (no voice detected)
//!   `~`       VAD detected voice (mic hearing you)
//!   `text~`   partial transcription streaming — body + VAD tail
//!   `text…`   polish in flight (after release, before upgrade)
//!   `text`    final, polished, committed (no tail)
//!
//! Modes:
//!   Hold      — press V to start, release to commit
//!   Locked    — double-tap V to enter, tap V again to commit + exit

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::key_code::KeyCode;
use crate::platform::{Platform, PttTrayState};

/// Delay before showing any placeholder — quick taps (<150ms) stay invisible.
const PLACEHOLDER_DELAY_MS: u64 = 150;

/// Path to the context file shared with the otoji subprocess.
/// Written on press (AX tree snapshot), read by otoji on PTT end.
pub fn ptt_context_file_path() -> String {
    format!("/tmp/capslockx-otoji-ctx-{}.txt", std::process::id())
}

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

/// Tail glyph typed right after the body text. Represents the "live"
/// micro-state: mic starting, listening silent, listening with voice, or
/// polish pending.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Tail {
    None,
    MicStarting,   // "."  — otoji subprocess not yet ready
    ListenSilent,  // "-"  — mic_ready, VAD silent
    ListenVoice,   // "~"  — VAD detected voice
    Polishing,     // "…"  — ptt_final received, awaiting upgrade
}

impl Tail {
    fn glyph(self) -> Option<&'static str> {
        match self {
            Tail::None => None,
            Tail::MicStarting => Some("."),
            Tail::ListenSilent => Some("-"),
            Tail::ListenVoice => Some("~"),
            Tail::Polishing => Some("…"),
        }
    }
}

pub struct PttSession {
    platform: Arc<dyn Platform>,
    /// Body text currently typed at the cursor (partial or raw committed
    /// segment), **without** the tail glyph. The tail is tracked separately
    /// so VAD can swap `./−/~` without disturbing the body.
    displayed: Arc<Mutex<String>>,
    /// Active tail glyph. Reflects micro-state: mic starting, listening
    /// silent, listening with voice, polish pending, or none.
    tail: Arc<Mutex<Tail>>,
    /// Monotonic token — cancels stale deferred tasks.
    token_counter: AtomicU64,
    /// True when locked PTT mode is active (double-tap to enter).
    locked: AtomicBool,
    /// Timestamp of last tap release — for double-click detection.
    last_tap_time: Mutex<Option<Instant>>,
    /// Text that was last typed by `on_ptt_final` — used by `on_ptt_upgrade`
    /// to compute a minimal diff against the polished version.
    last_committed: Mutex<String>,
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
            tail: Arc::new(Mutex::new(Tail::None)),
            token_counter: AtomicU64::new(0),
            locked: AtomicBool::new(false),
            last_tap_time: Mutex::new(None),
            last_committed: Mutex::new(String::new()),
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
        self.platform.set_ptt_tray_state(PttTrayState::Recording);

        // Fetch AX tree + Vision OCR in parallel — both complete during
        // the user's speech and become available to otoji (via ctx file)
        // by the time the user releases V. Zero added latency on the
        // critical path. OCR is the one that rescues STT drops of short
        // technical tokens (CLI, tmux, PR numbers) that AX doesn't expose
        // but the on-screen pixels do.
        std::thread::Builder::new()
            .name("ptt-ctx-fetch".into())
            .spawn(|| {
                let ax_handle = std::thread::Builder::new()
                    .name("ptt-ax-fetch".into())
                    .spawn(fetch_frontmost_ax_tree)
                    .ok();
                let ocr_handle = std::thread::Builder::new()
                    .name("ptt-ocr-fetch".into())
                    .spawn(fetch_frontmost_ocr)
                    .ok();
                let tree = ax_handle.and_then(|h| h.join().ok()).unwrap_or_default();
                let ocr  = ocr_handle.and_then(|h| h.join().ok()).unwrap_or_default();
                let mut ctx = tree;
                if !ocr.trim().is_empty() {
                    if !ctx.is_empty() && !ctx.ends_with('\n') { ctx.push('\n'); }
                    ctx.push_str("[OCR]\n");
                    ctx.push_str(&ocr);
                }
                let path = ptt_context_file_path();
                let tmp = format!("{path}.tmp");
                if std::fs::write(&tmp, &ctx).is_ok() {
                    let _ = std::fs::rename(&tmp, &path);
                }
            })
            .ok();

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
                    // "." — mic still starting.
                    this.set_tail(Tail::MicStarting);
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
                // "-" — mic ready, awaiting VAD. Flips to "~" on on_vad(true).
                this.set_tail(Tail::ListenSilent);
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
                        this.set_tail(Tail::ListenSilent);
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
        // Body = partial text; tail = VoiceDetected (partial implies voice).
        self.set_body(text.to_string(), Tail::ListenVoice);
    }

    /// Called when otoji emits a `vad` event. Only affects the tail glyph
    /// when the body is still empty (i.e. before any partial arrives) —
    /// once partials are streaming they already imply active voice.
    pub fn on_vad(&self, active: bool) {
        if !self.ptt_active.load(Ordering::Relaxed) { return; }
        // Mirror VAD on/off to the otoji-tray icon so the menu-bar reflects
        // voice activity during PTT (matches note-mode behavior).
        super::voice_otoji::notify_tray(if active {
            super::voice_otoji::TrayState::ListenVoice
        } else {
            super::voice_otoji::TrayState::ListenSilent
        });
        let body_empty = self.displayed.lock().unwrap().is_empty();
        if !body_empty { return; }
        // Only flip between the two listening states; don't override
        // MicStarting (not ready yet) or Polishing (after release).
        let current = *self.tail.lock().unwrap();
        let can_flip = matches!(current, Tail::ListenSilent | Tail::ListenVoice);
        if !can_flip { return; }
        self.set_tail(if active { Tail::ListenVoice } else { Tail::ListenSilent });
    }

    /// Called when otoji emits a `ptt_final` event — the RAW transcription,
    /// typed immediately while polish runs in the background.
    pub fn on_ptt_final(self: &Arc<Self>, text: &str) {
        let text = text.trim();
        // Erase live span (body + tail) — what's left to the left is text0/text1.
        self.set_body(String::new(), Tail::None);
        if !text.is_empty() {
            eprintln!("[CLX] PTT: final (raw) -> {:?}", text);
            // Type raw text + "…" tail directly. We DON'T put raw into
            // `displayed` — it becomes a "pending-polish committed prefix"
            // that on_ptt_upgrade will diff-replace using last_committed.
            // The "…" tail IS tracked via `tail` so on_vad/set_body know
            // to leave it alone (Polishing is not in the flippable set).
            self.platform.type_text(text);
            *self.tail.lock().unwrap() = Tail::Polishing;
            self.platform.type_text("…");
            *self.last_committed.lock().unwrap() = text.to_string();

            self.platform.set_ptt_tray_state(PttTrayState::Processing);

            let this = Arc::clone(self);
            std::thread::Builder::new()
                .name("ptt-tray-reset".into())
                .spawn(move || {
                    std::thread::sleep(Duration::from_secs(3));
                    // Drop the "…" tail if upgrade never arrived. Body is
                    // already empty so set_body just clears the tail glyph.
                    if *this.tail.lock().unwrap() == Tail::Polishing {
                        this.set_body(String::new(), Tail::None);
                    }
                    this.platform.set_ptt_tray_state(PttTrayState::Idle);
                })
                .ok();
        }

        // (on_ptt_upgrade may arrive later with polished text — we diff-update there.)

        // If locked mode, start a new PTT segment for the next utterance.
        if self.locked.load(Ordering::Relaxed) {
            self.send_ptt_start();
        } else {
            self.ptt_active.store(false, Ordering::Relaxed);
        }
    }

    /// Called when otoji emits a `ptt_translated` event. Behavior depends on
    /// the translation config (env-driven in Phase 1):
    ///
    /// - `CLX_TRANSLATE_TYPE=original` (default): ignore — typed text stays
    ///   in the source language.
    /// - `CLX_TRANSLATE_TYPE=translated`: replace the already-typed original
    ///   with the translation.
    /// - `CLX_TRANSLATE_TYPE=both`: append the translation after the original,
    ///   joined by `CLX_TRANSLATE_BOTH_TEMPLATE` (default `\n`) — or rendered
    ///   via the template if it contains `__ORIGINAL__` / `__TRANSLATION__`.
    pub fn on_ptt_translated(&self, translated: &str, _lang: &str) {
        let translated = translated.trim();
        if translated.is_empty() { return; }
        // Drop polish tail if still showing.
        if *self.tail.lock().unwrap() == Tail::Polishing {
            self.set_body(String::new(), Tail::None);
        }

        let mode = std::env::var("CLX_TRANSLATE_TYPE").unwrap_or_else(|_| "original".into());
        match mode.as_str() {
            "translated" => {
                // Replace original with translation (like ptt_upgrade).
                let mut prev = self.last_committed.lock().unwrap();
                if translated == *prev { return; }
                let prev_chars = prev.chars().count();
                let new_chars = translated.chars().count();
                if (prev_chars as isize - new_chars as isize).abs() > 200 {
                    eprintln!("[CLX] PTT: translation skipped — size delta too large");
                    return;
                }
                let common: usize = prev.chars().zip(translated.chars())
                    .take_while(|(a, b)| a == b).count();
                let old_tail = prev_chars - common;
                let new_tail: String = translated.chars().skip(common).collect();
                for _ in 0..old_tail {
                    self.platform.key_tap(KeyCode::Backspace);
                }
                if !new_tail.is_empty() {
                    self.platform.type_text(&new_tail);
                }
                *prev = translated.to_string();
                eprintln!("[CLX] PTT: translated → {:?}", translated);
            }
            "both" => {
                // Append translation after original, using template.
                let tmpl = std::env::var("CLX_TRANSLATE_BOTH_TEMPLATE")
                    .unwrap_or_else(|_| "__ORIGINAL__\n__TRANSLATION__".into());
                let orig = self.last_committed.lock().unwrap().clone();
                let rendered = tmpl
                    .replace("__ORIGINAL__", &orig)
                    .replace("__TRANSLATION__", translated)
                    .replace("\\n", "\n");
                // Diff against what's already displayed (= orig).
                let common: usize = orig.chars().zip(rendered.chars())
                    .take_while(|(a, b)| a == b).count();
                let old_tail = orig.chars().count() - common;
                let new_tail: String = rendered.chars().skip(common).collect();
                for _ in 0..old_tail {
                    self.platform.key_tap(KeyCode::Backspace);
                }
                if !new_tail.is_empty() {
                    self.platform.type_text(&new_tail);
                }
                *self.last_committed.lock().unwrap() = rendered;
                eprintln!("[CLX] PTT: both (orig + translation) appended");
            }
            _ => {
                // "original" or unknown — do nothing.
            }
        }
    }

    /// Called when otoji emits a `ptt_upgrade` event — the polished version
    /// of the most recent `ptt_final`. Diff-updates the cursor: backspace the
    /// differing suffix, type the new suffix. Usually just a character or two.
    pub fn on_ptt_upgrade(&self, polished: &str) {
        let polished = polished.trim();
        // Drop the "…" polish tail before diff-replacing the raw text.
        // `displayed` body is empty (raw was typed direct, not tracked) so
        // set_body just deletes the tail glyph.
        if *self.tail.lock().unwrap() == Tail::Polishing {
            self.set_body(String::new(), Tail::None);
        }
        let mut prev = self.last_committed.lock().unwrap();
        if polished == *prev || polished.is_empty() {
            return;
        }
        // Safety net: if polish changed the text drastically, skip the upgrade
        // to avoid a jarring rewrite. Conservative threshold.
        let prev_chars = prev.chars().count();
        let new_chars = polished.chars().count();
        let size_delta = (prev_chars as isize - new_chars as isize).abs();
        if size_delta > 20 {
            eprintln!("[CLX] PTT: upgrade skipped — size delta too large ({prev_chars} → {new_chars})");
            return;
        }

        // Find longest common prefix.
        let common: usize = prev.chars().zip(polished.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let old_tail_chars = prev_chars - common;
        let new_tail: String = polished.chars().skip(common).collect();

        eprintln!("[CLX] PTT: upgrade {:?} → {:?} (bs={}, type={:?})",
            &**prev, polished, old_tail_chars, new_tail);

        for _ in 0..old_tail_chars {
            self.platform.key_tap(KeyCode::Backspace);
        }
        if !new_tail.is_empty() {
            self.platform.type_text(&new_tail);
        }
        *prev = polished.to_string();
        // Upgrade finished — return tray to Idle (or NoteMode if active).
        self.platform.set_ptt_tray_state(PttTrayState::Idle);
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

    #[cfg(unix)]
    fn send_signal(pid: u32, sig: i32) {
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let ret = unsafe { kill(pid as i32, sig) };
        if ret != 0 {
            // errno accessor is libc-specific (__error on macOS/BSD,
            // __errno_location on glibc). std::io::Error does the right thing
            // portably, so use that for the diagnostic.
            let err = std::io::Error::last_os_error();
            eprintln!("[CLX] PTT: kill({pid}, {sig}) FAILED: ret={ret}, err={err}");
        }
    }

    #[cfg(not(unix))]
    fn send_signal(_pid: u32, _sig: i32) {
        // Windows otoji handshake would use a named pipe or TCP socket rather
        // than POSIX signals. PTT is a no-op on non-unix until that lands.
        eprintln!("[CLX] PTT: send_signal skipped (non-unix platform)");
    }

    /// Atomically replace the live (body + tail) span at the cursor using a
    /// common-prefix diff. Anything to the left of the live span (committed
    /// segments, text0) is never touched.
    fn set_body(&self, new_body: String, new_tail: Tail) {
        let mut displayed = self.displayed.lock().unwrap();
        let mut tail = self.tail.lock().unwrap();
        let old: String = format!("{}{}", &**displayed, tail.glyph().unwrap_or(""));
        let new: String = format!("{}{}", new_body, new_tail.glyph().unwrap_or(""));

        let common: usize = old.chars().zip(new.chars())
            .take_while(|(a, b)| a == b).count();
        let old_tail_chars = old.chars().count() - common;
        let new_tail_str: String = new.chars().skip(common).collect();

        for _ in 0..old_tail_chars {
            self.platform.key_tap(KeyCode::Backspace);
        }
        if !new_tail_str.is_empty() {
            self.platform.type_text(&new_tail_str);
        }
        *displayed = new_body;
        *tail = new_tail;
    }

    /// Update only the tail glyph, leaving the body alone. Convenience over
    /// `set_body(current_body, t)`.
    fn set_tail(&self, new_tail: Tail) {
        let body = self.displayed.lock().unwrap().clone();
        self.set_body(body, new_tail);
    }
}

/// Fetch the frontmost app's accessibility tree via osascript (System Events).
/// Never calls AX APIs directly — that hangs in kernel without Accessibility
/// permission. Safe on macOS, empty string on other platforms.
#[cfg(target_os = "macos")]
fn fetch_frontmost_ax_tree() -> String {
    let script = r#"
tell application "System Events"
    set fp to first application process whose frontmost is true
    set appName to name of fp
    set res to "[AX] app=" & appName & linefeed
    try
        repeat with w in (windows of fp)
            try
                set wName to name of w
                set res to res & "  window \"" & wName & "\"" & linefeed
            end try
            try
                repeat with e in (UI elements of w)
                    try
                        set eRole to role of e
                        set eTitle to ""
                        try
                            set eTitle to title of e
                        end try
                        if eTitle is "" then try
                            set eTitle to description of e
                        end try
                        if eTitle is "" then try
                            set eTitle to value of e as string
                        end try
                        set res to res & "    " & eRole & " \"" & eTitle & "\"" & linefeed
                    end try
                end repeat
            end try
        end repeat
    end try
    return res
end tell
"#;
    match std::process::Command::new("osascript").arg("-e").arg(script).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => String::new(),
    }
}

#[cfg(not(target_os = "macos"))]
fn fetch_frontmost_ax_tree() -> String { String::new() }

/// Capture the frontmost window and run Vision OCR on it. Invokes the
/// sibling `clx-ocr` binary (Swift, compiled by build.sh). Returns the
/// empty string if the binary is missing, permission is denied, or the
/// 3s budget expires.
#[cfg(target_os = "macos")]
fn fetch_frontmost_ocr() -> String {
    // Look for clx-ocr next to the current executable.
    let Ok(exe) = std::env::current_exe() else { return String::new(); };
    let Some(dir) = exe.parent() else { return String::new(); };
    let ocr_bin = dir.join("clx-ocr");
    if !ocr_bin.exists() { return String::new(); }

    match std::process::Command::new(&ocr_bin).output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        _ => String::new(),
    }
}

#[cfg(not(target_os = "macos"))]
fn fetch_frontmost_ocr() -> String { String::new() }
