//! CLX+= and CLX+- — evaluate the selection as JavaScript.
//!
//! Select `1+1`, hold a trigger, press `=`, and the selection becomes `2`. The
//! port of `Modules/CLX-NodeEval.ahk`, which bound the same two keys:
//!
//! | gesture | AHK | here |
//! |---|---|---|
//! | `CLX+-` | evaluate, replace the selection | same |
//! | `CLX+=` | evaluate, append if the selection ended in `=` | same |
//!
//! `CLX+=` appending is the useful half: select `2+2=`, press it, and you get
//! `2+2=4` — the working left where you can see it. Select `2+2` without the
//! trailing `=` and it is replaced instead.
//!
//! Three things differ from the AHK original, all deliberate:
//!
//!   * **No Node.** AHK launched `node --inspect` and talked to it over a port
//!     (`CLX-NodeEval.ahk:143`). This evaluates in-process with the JS engine that
//!     is already compiled into clx — nothing to install, nothing to supervise.
//!   * **The clipboard is put back.** AHK cleared it (`Clipboard =`) and left the
//!     result sitting there, so evaluating an expression silently destroyed
//!     whatever you had copied.
//!   * **A timeout.** `while(true){}` in a selection stops after two seconds
//!     instead of hanging the tool that owns your keyboard.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::key_code::{KeyCode, Modifiers};
use crate::platform::Platform;

/// How long a selection gets to evaluate. Short on purpose: this sits behind a
/// keystroke, and a keystroke that appears to do nothing for four seconds reads
/// as a bug. The agent's own budget is longer because a model is waiting, not a
/// person.
const EVAL_TIMEOUT: Duration = Duration::from_secs(2);

/// Worst case wait for the OS to service our synthetic copy before we read the
/// clipboard. Polled, so the usual cost is 10–30 ms.
const COPY_TIMEOUT_MS: u64 = 1000;

/// Let the paste land before restoring the old clipboard. Unavoidably a guess: we
/// can see the clipboard change when *we* write it, but not when the target app
/// has finished reading it. Too short and the paste gets the restored value; this
/// is generous enough that it has not been observed to matter.
const PASTE_SETTLE_MS: u64 = 250;

pub struct JsCalcModule {
    platform: Arc<dyn Platform>,
    /// One evaluation at a time. Two overlapping clipboard round-trips would
    /// restore each other's saved value and lose the user's clipboard.
    busy: Arc<AtomicBool>,
}

/// What to do with the result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// `CLX+-` — always replace the selection.
    Replace,
    /// `CLX+=` — append when the selection ended in `=`, else replace.
    AppendIfEquals,
}

impl JsCalcModule {
    pub fn new(platform: Arc<dyn Platform>) -> Self {
        Self {
            platform,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_mapped_key(&self, key: KeyCode) -> bool {
        matches!(key, KeyCode::Equal | KeyCode::Minus)
    }

    pub fn on_key_down(&self, key: KeyCode, mods: &Modifiers) -> bool {
        // Leave decorated presses alone: Ctrl+= is browser zoom, and a user
        // holding a modifier is not asking for arithmetic.
        if mods.ctrl || mods.alt || mods.win {
            return false;
        }
        let mode = match key {
            KeyCode::Equal => Mode::AppendIfEquals,
            KeyCode::Minus => Mode::Replace,
            _ => return false,
        };

        // Claim the slot, or decline. `swap` rather than load-then-store so two
        // fast presses cannot both pass.
        if self.busy.swap(true, Ordering::SeqCst) {
            return true; // still ours to swallow; just don't start a second one
        }

        // Off the hook thread: this copies, evaluates and pastes, and none of
        // that belongs inside a keyboard callback.
        let platform = Arc::clone(&self.platform);
        let busy = Arc::clone(&self.busy);
        let spawned = std::thread::Builder::new()
            .name("clx-js-calc".into())
            .spawn(move || {
                evaluate_selection(&*platform, mode);
                busy.store(false, Ordering::SeqCst);
            })
            .is_ok();
        if !spawned {
            self.busy.store(false, Ordering::SeqCst);
        }
        true
    }

    pub fn on_key_up(&self, _key: KeyCode) -> bool {
        false
    }
}

/// Strip the trailing `=` (and any trailing newline or `*/`) that marks "put the
/// answer after this", and say whether one was there.
///
/// The `*/` case is what makes it work inside a comment: `/* 2+2= */` in source
/// code evaluates and appends without breaking the comment.
fn split_trailing_equals(code: &str) -> (&str, bool) {
    let trimmed = code.trim_end();
    let without_comment = trimmed.strip_suffix("*/").unwrap_or(trimmed).trim_end();
    match without_comment.strip_suffix('=') {
        Some(expr) => (expr.trim_end(), true),
        None => (without_comment, false),
    }
}

/// Should the text be appended after the selection rather than replacing it?
///
/// Appending means stepping right off the end of the selection before pasting.
/// Two reasons to do it, and they are different in kind:
///
///   * the user asked, by ending the selection with `=` (`CLX+=` only);
///   * **it failed**, in which case appending is what keeps their input. A typo
///     must not cost you the expression you were writing — AHK pasted error text
///     over the selection and `asdf=` became a ReferenceError with the original
///     gone.
fn appends(mode: Mode, had_equals: bool, failed: bool) -> bool {
    failed || (mode == Mode::AppendIfEquals && had_equals)
}

fn evaluate_selection(platform: &dyn Platform, mode: Mode) {
    // 1. Copy the selection, keeping the old clipboard to put back.
    let saved = platform.get_clipboard_text();
    let seq_before = platform.clipboard_sequence();
    platform.key_tap_cmd_or_ctrl(KeyCode::C);
    wait_for_clipboard_change(platform, seq_before);
    let selection = platform.get_clipboard_text();

    // Nothing selected: the copy did nothing, so the clipboard still holds what it
    // did. Leave it exactly as found rather than writing the same value back.
    if selection.trim().is_empty() || selection == saved {
        eprintln!("[CLX] js-calc: nothing selected");
        return;
    }

    // 2. Evaluate.
    let (expr, had_equals) = split_trailing_equals(&selection);
    if expr.is_empty() {
        restore(platform, &saved);
        return;
    }
    // On failure the user gets the reason, written where they are looking.
    //
    // The first version failed silently — it logged to stderr and put the
    // clipboard back, so `asdf=` did nothing at all and there was no way to tell
    // a broken expression from a broken feature. Silence is the worst outcome.
    //
    // But the error is *appended*, never substituted, even in replace mode. AHK
    // pasted error text over the selection, so a typo turned `asdf=` into a
    // ReferenceError and lost what you had written. Destroying input in order to
    // report a failure is a bad trade.
    let (result, failed) = match crate::js::eval(expr, EVAL_TIMEOUT) {
        Ok(value) => (value, false),
        Err(message) => {
            eprintln!("[CLX] js-calc: {message}");
            (message, true)
        }
    };

    // 3. Put the result where it goes.
    platform.set_clipboard_text(&result);
    if appends(mode, had_equals, failed) {
        platform.key_tap(KeyCode::Right);
    }
    platform.key_tap_cmd_or_ctrl(KeyCode::V);

    // 4. Give the paste time to read the clipboard, then put the old one back.
    std::thread::sleep(Duration::from_millis(PASTE_SETTLE_MS));
    restore(platform, &saved);
}

fn restore(platform: &dyn Platform, saved: &str) {
    // An empty save means there was nothing there; writing "" would be a change,
    // not a restoration.
    if !saved.is_empty() {
        platform.set_clipboard_text(saved);
    }
}

/// Block until the clipboard sequence counter moves, or the timeout expires.
///
/// Same approach as `brainstorm`: poll the counter rather than sleep a fixed
/// worst case, and fall back to sleeping when the platform cannot report one.
fn wait_for_clipboard_change(platform: &dyn Platform, before: Option<u64>) {
    let timeout = Duration::from_millis(COPY_TIMEOUT_MS);
    let Some(before) = before else {
        std::thread::sleep(timeout);
        return;
    };
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if platform
            .clipboard_sequence()
            .is_some_and(|now| now != before)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    #[test]
    fn only_equal_and_minus_are_claimed() {
        let m = JsCalcModule::new(Arc::new(MockPlatform::new()));
        assert!(m.is_mapped_key(KeyCode::Equal));
        assert!(m.is_mapped_key(KeyCode::Minus));
        assert!(!m.is_mapped_key(KeyCode::A));
        assert!(!m.is_mapped_key(KeyCode::D1));
    }

    /// Ctrl+= is browser zoom. A decorated press is not a request for arithmetic.
    #[test]
    fn a_modified_press_is_left_alone() {
        let m = JsCalcModule::new(Arc::new(MockPlatform::new()));
        let ctrl = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        assert!(!m.on_key_down(KeyCode::Equal, &ctrl));
        assert!(!m.on_key_down(KeyCode::Minus, &ctrl));
    }

    #[test]
    fn a_trailing_equals_asks_for_the_answer_to_be_appended() {
        assert_eq!(split_trailing_equals("2+2="), ("2+2", true));
        assert_eq!(split_trailing_equals("2+2 = "), ("2+2", true));
        assert_eq!(split_trailing_equals("2+2=\n"), ("2+2", true));
        assert_eq!(split_trailing_equals("2+2"), ("2+2", false));
    }

    /// So it works inside source code: `/* 1+1= */` keeps the comment intact.
    #[test]
    fn a_comment_terminator_does_not_hide_the_equals() {
        assert_eq!(split_trailing_equals("1+1= */"), ("1+1", true));
        assert_eq!(split_trailing_equals("1+1 =*/"), ("1+1", true));
    }

    /// `a == b` must not be mistaken for `a =` plus an append marker.
    #[test]
    fn a_comparison_is_not_an_append_marker() {
        let (expr, had) = split_trailing_equals("1 == 1");
        assert_eq!(expr, "1 == 1");
        assert!(!had);
    }

    /// The exact case the user hit: `asdf=` did nothing at all, with no way to
    /// tell a broken expression from a broken feature.
    #[test]
    fn a_broken_expression_has_a_reason_worth_showing() {
        let (expr, had_equals) = split_trailing_equals("asdf= ");
        assert_eq!((expr, had_equals), ("asdf", true));

        let err = crate::js::eval(expr, EVAL_TIMEOUT).expect_err("asdf is undefined");
        assert!(
            err.contains("asdf is not defined"),
            "the user needs the actual reason, got {err:?}"
        );
    }

    /// A failure must never cost the user what they typed, in either mode.
    #[test]
    fn a_failure_always_appends_so_the_input_survives() {
        for mode in [Mode::Replace, Mode::AppendIfEquals] {
            for had_equals in [true, false] {
                assert!(
                    appends(mode, had_equals, true),
                    "{mode:?} had_equals={had_equals}: an error must be appended, not substituted"
                );
            }
        }
    }

    /// Success does what was asked: replace, unless the trailing `=` asked for the
    /// answer to go after the working.
    #[test]
    fn success_replaces_unless_the_equals_asked_otherwise() {
        assert!(appends(Mode::AppendIfEquals, true, false));
        assert!(!appends(Mode::AppendIfEquals, false, false));
        assert!(!appends(Mode::Replace, true, false));
        assert!(!appends(Mode::Replace, false, false));
    }

    #[test]
    fn nothing_selected_leaves_the_clipboard_untouched() {
        let p = MockPlatform::new();
        // MockPlatform's clipboard starts empty, so the synthetic copy changes
        // nothing and the selection reads as empty.
        evaluate_selection(&p, Mode::Replace);
        let calls = p.calls();
        assert!(
            !calls.iter().any(|c| matches!(c, Call::SetClipboardText(_))),
            "must not write the clipboard when there was no selection: {calls:?}"
        );
    }
}
