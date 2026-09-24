//! The CLX effect language — parse and perform.
//!
//! One line, one effect. This is the action half of a `.clx` binding
//! (`lab/clx-lang`), the thing `clx agent --exec` runs, and the contract a
//! plugin speaks on stdout (`lab/plugins`). All three want the same executor,
//! which is why it lives here rather than in an adapter.
//!
//! ```text
//! k enter              tap a key
//! k c-c                Ctrl+C          (c=ctrl s=shift a=alt w=win/cmd)
//! k "hello\n"          type a string
//! retype "hello!"      revise what was just typed, in place
//! commit               end the revisable run
//! m +40 -10            move the pointer, relative
//! click                click where the pointer is
//! click r x2           right button, twice
//! scroll down 3        wheel
//! w 200ms              wait
//! # anything           comment
//! ```
//!
//! **Effects, not domains.** Every verb here says *what to do* and none says
//! *why*. That is what lets CLX host a plugin without learning what the plugin
//! is for: otoji emits `k "the words"` and CLX types them, knowing nothing
//! about speech.
//!
//! [`Effect::Retype`] is the one verb that needs the executor to remember
//! something, and it is here rather than in the plugin for a reason: only CLX
//! knows what it actually injected. A plugin doing its own backspace
//! arithmetic would have to guess, and would be wrong the moment the user
//! typed anything in between.
//!
//! Deliberately smaller than the `.clx` draft. The draft's pointer grammar has
//! absolute coordinates, percentages, window-relative positions and
//! `@element` targets; `Platform` today offers relative movement and nothing
//! else, so those parse to [`Effect::Unsupported`] with the reason attached
//! rather than silently doing something wrong. Widening this means widening
//! `Platform` first.

use std::time::Duration;

use crate::key_code::KeyCode;
use crate::platform::{MouseButton, Platform};

/// One parsed line. `Comment` and the two error cases are values rather than
/// parse failures so that a stream can be executed without stopping at the
/// first bad line — a plugin that emits one malformed effect should lose that
/// effect, not the rest of its output.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// `k enter`, `k c-c` — tap `key` while `mods` are held.
    Key { key: KeyCode, mods: Vec<KeyCode> },
    /// `k "text"` — type a literal string, and begin a revisable run.
    Type(String),
    /// `retype "text"` — revise the current run in place. The executor diffs
    /// against what it last typed, erases only the differing suffix and types
    /// the replacement, so a draft becoming a correction costs a couple of
    /// keystrokes rather than a full rewrite.
    Retype(String),
    /// `commit` — end the run. The next `retype` then has nothing to revise
    /// and simply types.
    Commit,
    /// `m +40 -10` — move the pointer by a delta.
    MouseMove { dx: i32, dy: i32 },
    /// `click`, `click r x2` — press and release at the current position.
    Click { button: MouseButton, times: u32 },
    /// `scroll down 3` — positive `dy` scrolls down, `dx` right.
    Scroll { dx: i32, dy: i32 },
    /// `w 200ms`
    Wait(Duration),
    /// `# …`, or a blank line.
    Comment,
    /// Understood, but this build cannot perform it. Carries why.
    Unsupported { line: String, reason: String },
    /// Not understood at all.
    Unknown(String),
}

/// Scroll units. `.clx` counts in lines; `Platform::scroll_v` takes pixels.
const PIXELS_PER_SCROLL_LINE: i32 = 16;

/// Map a key name to a [`KeyCode`]. Single characters are themselves.
fn key_by_name(name: &str) -> Option<KeyCode> {
    let lower = name.to_ascii_lowercase();
    Some(match lower.as_str() {
        "enter" | "return" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Space,
        "esc" | "escape" => KeyCode::Escape,
        "backspace" | "bs" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        "capslock" => KeyCode::CapsLock,
        "insert" | "ins" => KeyCode::Insert,
        _ => {
            if let Some(n) = lower.strip_prefix('f') {
                if let Ok(n) = n.parse::<u8>() {
                    if (1..=24).contains(&n) {
                        return KeyCode::from_fn_number(n);
                    }
                }
            }
            if lower.chars().count() == 1 {
                return KeyCode::from_char(lower.chars().next().unwrap());
            }
            return None;
        }
    })
}

/// `c` `s` `a` `w` — ctrl, shift, alt, win/cmd. Left-hand keys, matching what
/// the agent language has always injected.
fn modifier_by_letter(c: char) -> Option<KeyCode> {
    Some(match c {
        'c' => KeyCode::LCtrl,
        's' => KeyCode::LShift,
        'a' => KeyCode::LAlt,
        'w' => KeyCode::LWin,
        _ => return None,
    })
}

/// `c-s-x` → (X, [LCtrl, LShift]). A bare name is a key with no modifiers.
fn parse_key_spec(spec: &str) -> Option<(KeyCode, Vec<KeyCode>)> {
    let parts: Vec<&str> = spec.split('-').collect();
    let (last, mods) = parts.split_last()?;

    // `k -` is the minus key, not an empty spec with a trailing separator.
    if last.is_empty() {
        return key_by_name("-").map(|k| (k, Vec::new()));
    }

    let mut keys = Vec::with_capacity(mods.len());
    for m in mods {
        let mut chars = m.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => keys.push(modifier_by_letter(c)?),
            _ => return None, // a multi-character segment is not a modifier
        }
    }
    Some((key_by_name(last)?, keys))
}

/// `"a\nb"` → `a`, newline, `b`. Only the four escapes the language defines.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn parse_duration(s: &str) -> Option<Duration> {
    if let Some(ms) = s.strip_suffix("ms") {
        return ms.trim().parse::<u64>().ok().map(Duration::from_millis);
    }
    if let Some(sec) = s.strip_suffix('s') {
        return sec
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(Duration::from_secs_f64);
    }
    None
}

fn parse_button(s: &str) -> Option<MouseButton> {
    Some(match s {
        "l" | "left" => MouseButton::Left,
        "r" | "right" => MouseButton::Right,
        "m" | "middle" => MouseButton::Middle,
        _ => return None,
    })
}

/// `x3` → 3.
fn parse_times(s: &str) -> Option<u32> {
    s.strip_prefix('x')?.parse().ok()
}

/// A relative coordinate: `+40`, `-10`, `40`.
fn parse_delta(s: &str) -> Option<i32> {
    s.strip_prefix('+').unwrap_or(s).parse().ok()
}

/// Parse one line. Never fails — see [`Effect`].
pub fn parse(line: &str) -> Effect {
    // A UTF-8 BOM on the first line is common enough to be worth absorbing:
    // several languages and shells emit one by default when writing UTF-8, and
    // without this the plugin's very first effect fails to parse for a reason
    // that is invisible in any log.
    let line = line.trim_start_matches('\u{feff}').trim();
    if line.is_empty() || line.starts_with('#') {
        return Effect::Comment;
    }
    let unsupported = |reason: &str| Effect::Unsupported {
        line: line.to_string(),
        reason: reason.to_string(),
    };
    let (verb, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();

    match verb {
        "k" => {
            if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
                return Effect::Type(unescape(&rest[1..rest.len() - 1]));
            }
            match parse_key_spec(rest) {
                Some((key, mods)) => Effect::Key { key, mods },
                None => Effect::Unknown(line.to_string()),
            }
        }

        "retype" => {
            if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
                Effect::Retype(unescape(&rest[1..rest.len() - 1]))
            } else {
                Effect::Unknown(line.to_string())
            }
        }

        "commit" => Effect::Commit,

        "m" | "hover" => {
            let args: Vec<&str> = rest.split_whitespace().collect();
            // Only `cur`-relative movement exists today. Absolute pixels,
            // percentages, `win`/`screen` origins and `@element` targets all
            // need pointer-position and geometry APIs that `Platform` does not
            // have — see the "Where the language is ahead of the engine" note
            // in lab/clx-lang.
            let deltas = match args.as_slice() {
                ["cur", dx, dy] => Some((dx, dy)),
                [dx, dy] if dx.starts_with(['+', '-']) && dy.starts_with(['+', '-']) => {
                    Some((dx, dy))
                }
                _ => None,
            };
            match deltas.and_then(|(dx, dy)| Some((parse_delta(dx)?, parse_delta(dy)?))) {
                Some((dx, dy)) => Effect::MouseMove { dx, dy },
                None => unsupported(
                    "only relative movement is available; Platform has no absolute pointer API",
                ),
            }
        }

        "click" | "dbl" => {
            let mut button = MouseButton::Left;
            let mut times = if verb == "dbl" { 2 } else { 1 };
            for arg in rest.split_whitespace() {
                if let Some(b) = parse_button(arg) {
                    button = b;
                } else if let Some(n) = parse_times(arg) {
                    times = n;
                } else {
                    return unsupported("click targets need element or coordinate support");
                }
            }
            Effect::Click { button, times }
        }

        "scroll" => {
            let args: Vec<&str> = rest.split_whitespace().collect();
            let (dir, amount) = match args.as_slice() {
                [dir] => (*dir, 3),
                [dir, n] => match n.trim_end_matches("px").parse::<i32>() {
                    Ok(v) => (
                        *dir,
                        if n.ends_with("px") {
                            v
                        } else {
                            v * PIXELS_PER_SCROLL_LINE
                        },
                    ),
                    Err(_) => return Effect::Unknown(line.to_string()),
                },
                _ => return unsupported("scroll targets need element support"),
            };
            let amount = if args.len() == 1 {
                3 * PIXELS_PER_SCROLL_LINE
            } else {
                amount
            };
            match dir {
                "down" => Effect::Scroll { dx: 0, dy: amount },
                "up" => Effect::Scroll { dx: 0, dy: -amount },
                "right" => Effect::Scroll { dx: amount, dy: 0 },
                "left" => Effect::Scroll { dx: -amount, dy: 0 },
                _ => Effect::Unknown(line.to_string()),
            }
        }

        "w" => match parse_duration(rest) {
            Some(d) => Effect::Wait(d),
            None => Effect::Unknown(line.to_string()),
        },

        // Verbs the language defines but this executor cannot serve yet. Named
        // explicitly so they read as "not yet" rather than "typo".
        "wf" => unsupported("wait-for needs an accessibility backend"),
        "drag" | "md" | "mu" | "mark" => unsupported("pointer state tracking is not implemented"),
        "scan" | "scan_stop" | "S" | "?" => unsupported("agent-only verb"),

        _ => Effect::Unknown(line.to_string()),
    }
}

/// Never erase more than this in one revision. A revision that large is not a
/// correction, it is a different sentence, and blindly backspacing through it
/// would eat whatever the user had already written.
const MAX_REVISION_ERASE: usize = 256;

/// Executes effects, remembering just enough to revise its own output.
///
/// The only state is the text of the current run — what [`Effect::Retype`]
/// diffs against. Everything else is stateless.
#[derive(Default)]
pub struct Runner {
    typed: String,
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform one effect.
    ///
    /// `Wait` blocks the calling thread, so drive this somewhere that can
    /// afford to block: never the keyboard hook.
    pub fn perform(&mut self, effect: &Effect, platform: &dyn Platform) {
        match effect {
            Effect::Type(text) => {
                platform.type_text(text);
                self.typed = text.clone();
            }
            Effect::Retype(text) => self.revise(text, platform),
            Effect::Commit => self.typed.clear(),
            other => perform(other, platform),
        }
    }

    /// Backspace the differing suffix, type the new one.
    fn revise(&mut self, next: &str, platform: &dyn Platform) {
        if next == self.typed {
            return;
        }
        let common = self
            .typed
            .chars()
            .zip(next.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let erase = self.typed.chars().count() - common;

        if erase > MAX_REVISION_ERASE {
            eprintln!(
                "[CLX] revision would erase {erase} characters — refusing, and                  starting a fresh run instead"
            );
            platform.type_text(next);
            self.typed = next.to_string();
            return;
        }

        if erase > 0 {
            platform.key_tap_n(KeyCode::Backspace, erase as i32);
        }
        let tail: String = next.chars().skip(common).collect();
        if !tail.is_empty() {
            platform.type_text(&tail);
        }
        self.typed = next.to_string();
    }
}

/// Perform one stateless effect.
///
/// [`Effect::Type`], [`Effect::Retype`] and [`Effect::Commit`] need a
/// [`Runner`] to remember the current run; passed here, `Retype` can only type
/// its text outright, since there is nothing to diff against.
///
/// `Wait` blocks the calling thread, so run a stream somewhere that can afford
/// to: never the keyboard hook.
pub fn perform(effect: &Effect, platform: &dyn Platform) {
    match effect {
        Effect::Key { key, mods } => platform.key_tap_with_mods(*key, mods, 1),
        Effect::Type(text) | Effect::Retype(text) => platform.type_text(text),
        Effect::Commit => {}
        Effect::MouseMove { dx, dy } => platform.mouse_move(*dx, *dy),
        Effect::Click { button, times } => {
            for _ in 0..*times {
                platform.mouse_button(*button, true);
                platform.mouse_button(*button, false);
            }
        }
        Effect::Scroll { dx, dy } => {
            if *dy != 0 {
                platform.scroll_v(*dy);
            }
            if *dx != 0 {
                platform.scroll_h(*dx);
            }
        }
        Effect::Wait(d) => std::thread::sleep(*d),
        Effect::Comment => {}
        Effect::Unsupported { line, reason } => {
            eprintln!("[CLX] effect not available: {line}  ({reason})");
        }
        Effect::Unknown(line) => {
            eprintln!("[CLX] unrecognised effect: {line}");
        }
    }
}

/// Parse and perform every line of a stream.
///
/// This is the plugin host in one function: give it a plugin's stdout and it
/// turns what the plugin wanted into what actually happens, without either side
/// knowing anything about the other's purpose.
pub fn perform_stream(reader: impl std::io::BufRead, platform: &dyn Platform) {
    let mut runner = Runner::new();
    for line in reader.lines() {
        let Ok(line) = line else { break };
        runner.perform(&parse(&line), platform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    fn run(script: &str) -> MockPlatform {
        let p = MockPlatform::new();
        perform_stream(std::io::Cursor::new(script), &p);
        p
    }

    #[test]
    fn keys_and_modifiers() {
        assert_eq!(
            parse("k enter"),
            Effect::Key {
                key: KeyCode::Enter,
                mods: vec![]
            }
        );
        assert_eq!(
            parse("k c-c"),
            Effect::Key {
                key: KeyCode::from_char('c').unwrap(),
                mods: vec![KeyCode::LCtrl]
            }
        );
        assert_eq!(
            parse("k c-s-a-w-x").unwrap_mods_len(),
            4,
            "every modifier letter should be accepted together"
        );
    }

    #[test]
    fn function_keys_parse() {
        assert!(matches!(parse("k f12"), Effect::Key { .. }));
        assert!(matches!(parse("k f99"), Effect::Unknown(_)));
    }

    #[test]
    fn strings_are_typed_not_tapped() {
        assert_eq!(parse(r#"k "hi""#), Effect::Type("hi".into()));
        assert_eq!(parse(r#"k "a\nb""#), Effect::Type("a\nb".into()));
        assert_eq!(parse(r#"k "q\"q""#), Effect::Type("q\"q".into()));
    }

    #[test]
    fn a_byte_order_mark_does_not_eat_the_first_effect() {
        // PowerShell, among others, writes a BOM when asked for UTF-8. Without
        // tolerance here, a plugin's opening line silently does nothing and
        // nothing in any log says why.
        let with_bom = format!("{}k \"hi\"", '\u{feff}');
        assert_eq!(parse(&with_bom), Effect::Type("hi".into()));
    }

    #[test]
    fn comments_and_blanks_do_nothing() {
        assert_eq!(parse("# note"), Effect::Comment);
        assert_eq!(parse("   "), Effect::Comment);
        assert!(run("# just a comment\n\n").calls.lock().unwrap().is_empty());
    }

    #[test]
    fn waits_accept_both_units() {
        assert_eq!(parse("w 200ms"), Effect::Wait(Duration::from_millis(200)));
        assert_eq!(parse("w 2s"), Effect::Wait(Duration::from_secs(2)));
        assert!(matches!(parse("w 5"), Effect::Unknown(_)));
    }

    #[test]
    fn relative_movement_works_absolute_is_refused() {
        assert_eq!(parse("m +40 -10"), Effect::MouseMove { dx: 40, dy: -10 });
        assert_eq!(parse("m cur +5 +5"), Effect::MouseMove { dx: 5, dy: 5 });
        // Absolute coordinates are in the language but not in Platform, and
        // must say so rather than move somewhere arbitrary.
        assert!(matches!(parse("m 400 300"), Effect::Unsupported { .. }));
    }

    #[test]
    fn clicks_count_and_choose_a_button() {
        assert_eq!(
            parse("click"),
            Effect::Click {
                button: MouseButton::Left,
                times: 1
            }
        );
        assert_eq!(
            parse("click r"),
            Effect::Click {
                button: MouseButton::Right,
                times: 1
            }
        );
        assert_eq!(
            parse("dbl"),
            Effect::Click {
                button: MouseButton::Left,
                times: 2
            }
        );
        assert_eq!(
            parse("click r x3"),
            Effect::Click {
                button: MouseButton::Right,
                times: 3
            }
        );
    }

    #[test]
    fn scrolling_has_a_direction() {
        assert_eq!(
            parse("scroll down 1"),
            Effect::Scroll {
                dx: 0,
                dy: PIXELS_PER_SCROLL_LINE
            }
        );
        assert_eq!(
            parse("scroll up 1"),
            Effect::Scroll {
                dx: 0,
                dy: -PIXELS_PER_SCROLL_LINE
            }
        );
        assert_eq!(parse("scroll down 48px"), Effect::Scroll { dx: 0, dy: 48 });
        assert!(matches!(parse("scroll sideways"), Effect::Unknown(_)));
    }

    #[test]
    fn unimplemented_verbs_say_why_rather_than_looking_like_typos() {
        for line in ["wf \"Save\" 3s", "drag a -> b", "mark here", "? screen"] {
            assert!(
                matches!(parse(line), Effect::Unsupported { .. }),
                "{line} should report a reason"
            );
        }
        assert!(matches!(parse("frobnicate"), Effect::Unknown(_)));
    }

    #[test]
    fn a_stream_performs_in_order_and_survives_a_bad_line() {
        let p = run("k enter\nnonsense verb\nk \"hi\"\n");
        let calls = p.calls.lock().unwrap();
        // The bad line is skipped; the good ones on either side still happen.
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::TypeText(t) if t == "hi")));
        assert!(
            calls.len() >= 2,
            "expected the enter tap and the typed text"
        );
    }

    #[test]
    fn a_plugin_needs_no_more_than_this() {
        // What otoji would emit: CLX types the words and knows nothing else.
        let p = run("# partial\nk \"hello world\"\n");
        let calls = p.calls.lock().unwrap();
        assert!(calls
            .iter()
            .any(|c| matches!(c, Call::TypeText(t) if t == "hello world")));
    }

    fn typed_and_erased(p: &MockPlatform) -> (String, usize) {
        let calls = p.calls.lock().unwrap();
        let mut text = String::new();
        let mut erased = 0;
        for c in calls.iter() {
            match c {
                Call::TypeText(t) => text.push_str(t),
                Call::KeyTapExtended(KeyCode::Backspace) | Call::KeyDown(KeyCode::Backspace) => {
                    erased += 1
                }
                _ => {}
            }
        }
        (text, erased)
    }

    #[test]
    fn retype_revises_only_the_differing_suffix() {
        // The shape every streaming recogniser produces: a draft, then a fix.
        let p = run("k \"hello worl\"\nretype \"hello world\"\n");
        let (text, _) = typed_and_erased(&p);
        assert!(text.ends_with('d'), "the correction should be typed");
        // "hello worl" -> "hello world" shares a 10-character prefix, so the
        // revision is one keystroke, not eleven.
        assert!(
            text.len() < "hello worlhello world".len(),
            "a shared prefix must not be retyped: {text:?}"
        );
    }

    #[test]
    fn retype_after_commit_starts_fresh() {
        let p = MockPlatform::new();
        let mut r = Runner::new();
        r.perform(&parse("k \"abc\""), &p);
        r.perform(&parse("commit"), &p);
        r.perform(&parse("retype \"xyz\""), &p);
        let (_, erased) = typed_and_erased(&p);
        assert_eq!(erased, 0, "a committed run must not be backspaced over");
    }

    #[test]
    fn an_identical_revision_does_nothing() {
        let p = MockPlatform::new();
        let mut r = Runner::new();
        r.perform(&parse("k \"same\""), &p);
        let before = p.calls.lock().unwrap().len();
        r.perform(&parse("retype \"same\""), &p);
        assert_eq!(p.calls.lock().unwrap().len(), before);
    }

    #[test]
    fn a_wild_revision_refuses_to_backspace_through_the_document() {
        let p = MockPlatform::new();
        let mut r = Runner::new();
        r.perform(&Effect::Type("x".repeat(MAX_REVISION_ERASE + 50)), &p);
        r.perform(&Effect::Retype("completely different".into()), &p);
        let (_, erased) = typed_and_erased(&p);
        assert_eq!(erased, 0, "should type fresh rather than erase that much");
    }

    #[test]
    fn a_streaming_plugin_reads_naturally() {
        // What otoji would emit across one utterance.
        let p = run(concat!(
            "k \"the quick brown\"\n",
            "retype \"the quick brown fox\"\n",
            "retype \"The quick brown fox.\"\n",
            "commit\n"
        ));
        let (text, _) = typed_and_erased(&p);
        assert!(text.contains("The quick brown fox."), "got {text:?}");
    }

    // small helper so the modifier test reads cleanly
    impl Effect {
        fn unwrap_mods_len(&self) -> usize {
            match self {
                Effect::Key { mods, .. } => mods.len(),
                other => panic!("expected a key effect, got {other:?}"),
            }
        }
    }
}
