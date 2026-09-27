//! User-defined gestures: a table from a key to something to run.
//!
//! The gap this fills: every action clx could perform was a `match` arm compiled
//! into it, and `ClxConfig` is a flat struct of typed settings with no extension
//! point. So a plugin had nowhere to be bound and could only be run from a shell,
//! which is not a feature anyone uses. See `lab/script-store`.
//!
//! The file is plain text, one binding per line:
//!
//! ```text
//! # trigger+key        what to run
//! p                    plugin clx-genpw dpw
//! space+;              plugin clx-genpw qpw
//! capslock+d           plugin clx-date --iso
//! ```
//!
//! A bare key matches under either trigger; `space+`/`capslock+` pins it to one.
//! Built-in gestures win — a binding for `h` will never fire, because Space+H is
//! cursor-left and silently shadowing that would be worse than refusing it.
//!
//! Parsing never fails. A typo disables its own line and reports why, because the
//! alternative is one bad character costing the user every other binding they
//! have.

use crate::key_code::KeyCode;

/// Which trigger a binding requires, if it cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Space,
    CapsLock,
    /// A bare key: fires under whichever trigger is held.
    Any,
}

/// What a gesture runs. Both forms describe effects rather than performing them;
/// the difference is only whether a process is involved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// A program whose stdout is effects.
    Plugin { command: String, args: Vec<String> },
    /// A `.js` file evaluated in-process. No build step, no binary per platform —
    /// see `lab/script-store` for why this is the shape the long tail wants.
    Script { path: String, args: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub trigger: Trigger,
    pub key: KeyCode,
    pub action: Action,
}

/// A parsed bindings file, plus whatever could not be understood.
#[derive(Debug, Default, Clone)]
pub struct Bindings {
    bindings: Vec<Binding>,
    /// Typed triggers — a gesture written as `#DPW#` rather than a key. Kept
    /// separate because they are matched against typed characters rather than key
    /// events, by `capslockx_core::hotstring`.
    hotstrings: Vec<(String, Action)>,
    /// `(line number, what was wrong)` — surfaced to the user rather than
    /// swallowed, because a binding that silently does nothing is the most
    /// annoying possible outcome.
    pub problems: Vec<(usize, String)>,
}

impl Bindings {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Typed triggers, for the adapter to hand to a hotstring watcher. Cloned
    /// rather than borrowed because the watcher outlives this object.
    pub fn hotstrings(&self) -> Vec<(String, Action)> {
        self.hotstrings.clone()
    }

    /// The action for `key` under `held`, if any.
    ///
    /// A trigger-specific binding beats a bare one, so `space+p` and `p` can
    /// coexist with the specific gesture winning.
    pub fn lookup(&self, key: KeyCode, held: Trigger) -> Option<&Action> {
        let exact = self
            .bindings
            .iter()
            .find(|b| b.key == key && b.trigger == held);
        let bare = self
            .bindings
            .iter()
            .find(|b| b.key == key && b.trigger == Trigger::Any);
        exact.or(bare).map(|b| &b.action)
    }

    /// Parse a bindings file. Never fails; unparseable lines land in `problems`.
    pub fn parse(text: &str) -> Self {
        let mut out = Bindings::default();
        for (n, raw) in text.lines().enumerate() {
            let line_no = n + 1;
            // BOM tolerance: a file written by a Windows editor starts with one,
            // and losing the first binding to an invisible character is a poor way
            // to spend an afternoon.
            let line = raw.trim_start_matches('\u{feff}').trim();
            // `#` is both the comment character and the hotstring leader, so the
            // two have to be told apart before either is handled. A hotstring's
            // first token is *delimited* by the leader — `#DPW#` — while a comment
            // opens with a bare `#`. Checked first, or every trigger is silently
            // read as a comment, which is exactly what happened.
            if is_hotstring(line) {
                match parse_hotstring(line) {
                    Ok((trigger, action)) => out.hotstrings.push((trigger, action)),
                    Err(why) => out.problems.push((line_no, why)),
                }
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match parse_line(line) {
                Ok(binding) => {
                    if let Some(prior) = out
                        .bindings
                        .iter()
                        .position(|b| b.key == binding.key && b.trigger == binding.trigger)
                    {
                        // Last one wins, but say so: a duplicate is usually a
                        // half-finished edit rather than an intention.
                        out.problems.push((
                            line_no,
                            format!("overrides the binding on line {}", prior + 1),
                        ));
                        out.bindings[prior] = binding;
                    } else {
                        out.bindings.push(binding);
                    }
                }
                Err(why) => out.problems.push((line_no, why)),
            }
        }
        out
    }
}

/// `#DPW#   script scripts/genpw.js dpw`
fn parse_hotstring(line: &str) -> Result<(String, Action), String> {
    let mut parts = line.split_whitespace();
    let trigger = parts.next().ok_or("empty line")?.to_string();
    let verb = parts
        .next()
        .ok_or_else(|| format!("{trigger:?} has no action — expected `script` or `plugin`"))?;
    Ok((trigger, parse_action(verb, parts)?))
}

fn parse_line(line: &str) -> Result<Binding, String> {
    let mut parts = line.split_whitespace();
    let gesture = parts.next().ok_or("empty line")?;
    let verb = parts
        .next()
        .ok_or_else(|| format!("{gesture:?} has no action — expected `plugin <command>`"))?;

    let (trigger, key_text) = match gesture.split_once('+') {
        Some(("space", k)) => (Trigger::Space, k),
        Some(("capslock", k)) => (Trigger::CapsLock, k),
        Some((other, _)) => {
            return Err(format!(
                "unknown trigger {other:?} — expected `space+` or `capslock+`"
            ))
        }
        None => (Trigger::Any, gesture),
    };

    // Says what *is* allowed, because "not bindable" on its own leaves the user
    // guessing — and the guess they will make is `;`, which has no `KeyCode`
    // variant yet. See TODO.
    let key = KeyCode::from_char(key_text.chars().next().ok_or("missing key")?)
        .filter(|_| key_text.chars().count() == 1)
        .ok_or_else(|| {
            format!(
                "{key_text:?} is not a bindable key — expected one of \
                 a-z 0-9 - = , . / [ ] \\"
            )
        })?;

    Ok(Binding {
        trigger,
        key,
        action: parse_action(verb, parts)?,
    })
}

/// The `plugin …` / `script …` half of a line, shared by key bindings and
/// hotstrings so both halves of the file speak the same language.
fn parse_action<'a>(
    verb: &str,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<Action, String> {
    Ok(match verb {
        "plugin" => Action::Plugin {
            command: parts
                .next()
                .ok_or("`plugin` needs a command to run")?
                .to_string(),
            args: parts.map(str::to_string).collect(),
        },
        "script" => Action::Script {
            path: parts
                .next()
                .ok_or("`script` needs a .js file to run")?
                .to_string(),
            args: parts.map(str::to_string).collect(),
        },
        other => {
            return Err(format!(
                "unknown action {other:?} — expected `plugin` or `script`"
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_key_binds_under_either_trigger() {
        let b = Bindings::parse("p plugin clx-genpw dpw");
        assert_eq!(b.len(), 1);
        assert!(b.problems.is_empty(), "{:?}", b.problems);
        for held in [Trigger::Space, Trigger::CapsLock] {
            match b.lookup(KeyCode::P, held) {
                Some(Action::Plugin { command, args }) => {
                    assert_eq!(command, "clx-genpw");
                    assert_eq!(args, &["dpw"]);
                }
                other => panic!("{held:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn a_trigger_can_be_pinned() {
        let b = Bindings::parse("space+q plugin one\ncapslock+q plugin two");
        assert!(b.problems.is_empty(), "{:?}", b.problems);
        assert!(matches!(
            b.lookup(KeyCode::Q, Trigger::Space),
            Some(Action::Plugin { command, .. }) if command == "one"
        ));
        assert!(matches!(
            b.lookup(KeyCode::Q, Trigger::CapsLock),
            Some(Action::Plugin { command, .. }) if command == "two"
        ));
    }

    /// The specific gesture is the one the user went out of their way to write.
    #[test]
    fn a_pinned_trigger_beats_a_bare_key() {
        let b = Bindings::parse("r plugin bare\nspace+r plugin pinned");
        assert!(matches!(
            b.lookup(KeyCode::R, Trigger::Space),
            Some(Action::Plugin { command, .. }) if command == "pinned"
        ));
        assert!(matches!(
            b.lookup(KeyCode::R, Trigger::CapsLock),
            Some(Action::Plugin { command, .. }) if command == "bare"
        ));
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let b = Bindings::parse("# a comment\n\n   \np plugin x\n");
        assert_eq!(b.len(), 1);
        assert!(b.problems.is_empty(), "{:?}", b.problems);
    }

    /// One bad line must not cost the user the rest of the file.
    #[test]
    fn a_broken_line_disables_only_itself() {
        let b = Bindings::parse("p plugin good\nnonsense\nq plugin also-good");
        assert_eq!(b.len(), 2, "{:?}", b.problems);
        assert_eq!(b.problems.len(), 1);
        assert_eq!(b.problems[0].0, 2, "should name the offending line");
    }

    #[test]
    fn unparseable_lines_say_why() {
        let b = Bindings::parse(
            "ctrl+p plugin x\n\
             pp plugin x\n\
             p\n\
             p wiggle x\n\
             p script\n",
        );
        assert!(b.is_empty(), "none of these should bind");
        let why: Vec<&str> = b.problems.iter().map(|(_, w)| w.as_str()).collect();
        assert!(why[0].contains("unknown trigger"), "{:?}", why[0]);
        assert!(why[1].contains("not a bindable key"), "{:?}", why[1]);
        assert!(why[2].contains("no action"), "{:?}", why[2]);
        assert!(why[3].contains("unknown action"), "{:?}", why[3]);
        assert!(why[4].contains("needs a .js file"), "{:?}", why[4]);
    }

    /// `#` opens a comment *and* a hotstring. Getting this wrong made every
    /// trigger silently vanish into the comment branch.
    #[test]
    fn a_comment_and_a_hotstring_are_told_apart() {
        let b = Bindings::parse(
            "# key gestures\n\
             #DPW#   script scripts/genpw.js dpw\n\
             # another comment, with #hashes# in prose\n\
             p       script scripts/genpw.js dpw\n",
        );
        assert_eq!(b.len(), 1, "one key binding: {:?}", b.problems);
        let hot = b.hotstrings();
        assert_eq!(hot.len(), 1, "one hotstring: {hot:?}");
        assert_eq!(hot[0].0, "#DPW#");
        assert!(b.problems.is_empty(), "{:?}", b.problems);
    }

    #[test]
    fn a_hotstring_carries_its_arguments() {
        let b = Bindings::parse("#QPW#  script scripts/genpw.js qpw");
        let hot = b.hotstrings();
        assert_eq!(hot.len(), 1);
        match &hot[0].1 {
            Action::Script { path, args } => {
                assert_eq!(path, "scripts/genpw.js");
                assert_eq!(args, &["qpw"]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_script_binds_with_its_path_and_arguments() {
        let b = Bindings::parse("p script scripts/genpw.js dpw");
        assert!(b.problems.is_empty(), "{:?}", b.problems);
        match b.lookup(KeyCode::P, Trigger::Any) {
            Some(Action::Script { path, args }) => {
                assert_eq!(path, "scripts/genpw.js");
                assert_eq!(args, &["dpw"]);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_duplicate_takes_effect_and_is_reported() {
        let b = Bindings::parse("p plugin first\np plugin second");
        assert_eq!(b.len(), 1);
        assert!(matches!(
            b.lookup(KeyCode::P, Trigger::Any),
            Some(Action::Plugin { command, .. }) if command == "second"
        ));
        assert_eq!(b.problems.len(), 1, "the shadowed line should be reported");
    }

    /// A Windows editor writes one and it is invisible.
    #[test]
    fn a_byte_order_mark_does_not_eat_the_first_binding() {
        let b = Bindings::parse(&format!("{}p plugin x", '\u{feff}'));
        assert_eq!(b.len(), 1, "{:?}", b.problems);
    }

    #[test]
    fn arguments_are_preserved_in_order() {
        let b = Bindings::parse("p plugin cmd --flag value  trailing");
        match b.lookup(KeyCode::P, Trigger::Any) {
            Some(Action::Plugin { command, args }) => {
                assert_eq!(command, "cmd");
                assert_eq!(args, &["--flag", "value", "trailing"]);
            }
            other => panic!("{other:?}"),
        }
    }
}

/// Is this line a typed trigger rather than a comment?
///
/// Both start with `#`. A trigger's first token is *delimited* by the leader
/// (`#DPW#`); a comment opens with a bare `#` and continues in prose. Without this
/// distinction every hotstring in the file reads as a comment and silently does
/// nothing — which is how the first version shipped.
fn is_hotstring(line: &str) -> bool {
    let leader = crate::hotstring::DEFAULT_LEADER;
    let Some(first) = line.split_whitespace().next() else {
        return false;
    };
    first.chars().count() > 1 && first.starts_with(leader) && first.ends_with(leader)
}
