//! Typed triggers: `#DPW#` becomes a generated password.
//!
//! The AHK behaviour people have muscle memory for
//! (`Modules/QuickInput.ahk`), with one deliberate difference: **nothing is
//! buffered until the leader character is typed.**
//!
//! That difference is the whole design. A hotstring engine needs to know what you
//! recently typed, and a rolling log of every keystroke inside the process that
//! owns the global keyboard hook is the one capability this project refuses to
//! give scripts — "a keylogger with extra steps". Starting the buffer at `#` and
//! discarding it the moment the text cannot become a trigger bounds what is held
//! to a few characters after a rare leader, instead of everything you type.
//!
//! What is still true of AHK's version and cannot be fixed without holding back
//! ordinary typing: the trigger characters reach the focused app before being
//! erased, so `#DPW#` briefly appears and is then backspaced away. In a plain text
//! field nobody notices; in an incremental search box the intermediate text is
//! visible, and something watching for "user is typing" sees it. Suppressing that
//! would mean withholding every `#` until it is clear no trigger is coming, which
//! costs more than it buys.

use crate::bindings::Action;

/// The character that opens a trigger. Everything before it is discarded, so this
/// is the knob that decides how much typing clx can ever see.
pub const DEFAULT_LEADER: char = '#';

/// A match: what to run, and how many characters to erase first.
#[derive(Debug, PartialEq, Eq)]
pub struct Match<'a> {
    pub action: &'a Action,
    /// Includes the leader, so the caller knows exactly how many backspaces the
    /// focused app needs to undo what it already received.
    pub erase: usize,
}

/// Registered triggers, and the buffer of what has been typed since the leader.
#[derive(Debug, Clone)]
pub struct Hotstrings {
    leader: char,
    /// `(trigger, action)`, trigger including its leader, matched case-insensitively
    /// because `#dpw#` and `#DPW#` are the same intent.
    entries: Vec<(String, Action)>,
    /// Since the leader. Empty means "not currently in a trigger".
    buffer: String,
    longest: usize,
}

impl Hotstrings {
    pub fn new(leader: char) -> Self {
        Self {
            leader,
            entries: Vec::new(),
            buffer: String::new(),
            longest: 0,
        }
    }

    /// Register a trigger. Returns the reason if it cannot be one.
    pub fn add(&mut self, trigger: &str, action: Action) -> Result<(), String> {
        if !trigger.starts_with(self.leader) {
            return Err(format!(
                "a hotstring must start with {:?} — got {trigger:?}",
                self.leader
            ));
        }
        if trigger.chars().count() < 2 {
            return Err(format!("{trigger:?} is too short to be a trigger"));
        }
        let lowered = trigger.to_lowercase();
        self.longest = self.longest.max(lowered.chars().count());
        self.entries.push((lowered, action));
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Forget any partial trigger. The caller should do this whenever the typing
    /// context changes under it — focus moved, a click landed, an arrow key was
    /// pressed — because the characters in the buffer may no longer be adjacent to
    /// the cursor, and backspacing then eats the wrong text.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Feed one typed character.
    ///
    /// `Some` means a trigger completed: erase that many characters and run the
    /// action. The buffer is cleared either way when a match is returned.
    pub fn push(&mut self, c: char) -> Option<Match<'_>> {
        if self.entries.is_empty() {
            return None;
        }

        // Nothing is retained until the leader arrives. A second leader restarts
        // rather than extends, so `##DPW#` still fires.
        if c == self.leader && self.buffer.is_empty() {
            self.buffer.push(c);
            return None;
        }
        if self.buffer.is_empty() {
            return None;
        }

        self.buffer.push(c);
        let lowered = self.buffer.to_lowercase();

        if let Some(index) = self.entries.iter().position(|(t, _)| *t == lowered) {
            let erase = self.buffer.chars().count();
            self.buffer.clear();
            return Some(Match {
                action: &self.entries[index].1,
                erase,
            });
        }

        // Still a prefix of something? Keep waiting. Otherwise drop it — but a
        // fresh leader starts over, so `#no#DPW#` works.
        let still_possible = self.entries.iter().any(|(t, _)| t.starts_with(&lowered));
        if !still_possible || lowered.chars().count() >= self.longest {
            self.buffer.clear();
            if c == self.leader {
                self.buffer.push(c);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(name: &str) -> Action {
        Action::Plugin {
            command: name.to_string(),
            args: vec![],
        }
    }

    fn with(triggers: &[&str]) -> Hotstrings {
        let mut h = Hotstrings::new(DEFAULT_LEADER);
        for t in triggers {
            h.add(t, action(t)).expect("valid trigger");
        }
        h
    }

    /// Type the string; return the (erase, command) of every match it produced.
    fn feed(h: &mut Hotstrings, text: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for c in text.chars() {
            if let Some(m) = h.push(c) {
                let Action::Plugin { command, .. } = m.action.clone() else {
                    panic!("unexpected action")
                };
                out.push((m.erase, command));
            }
        }
        out
    }

    #[test]
    fn a_trigger_fires_and_says_how_much_to_erase() {
        let mut h = with(&["#DPW#"]);
        assert_eq!(feed(&mut h, "#DPW#"), vec![(5, "#DPW#".to_string())]);
    }

    #[test]
    fn case_does_not_matter() {
        let mut h = with(&["#DPW#"]);
        assert_eq!(feed(&mut h, "#dpw#").len(), 1);
        assert_eq!(feed(&mut h, "#DpW#").len(), 1);
    }

    /// The point of the leader: ordinary typing is never retained.
    #[test]
    fn nothing_is_buffered_before_the_leader() {
        let mut h = with(&["#DPW#"]);
        feed(&mut h, "the quick brown fox jumps over the lazy dog");
        assert!(h.buffer.is_empty(), "held {:?}", h.buffer);
    }

    #[test]
    fn a_trigger_fires_mid_sentence() {
        let mut h = with(&["#DPW#"]);
        let hits = feed(&mut h, "password: #DPW# done");
        assert_eq!(hits, vec![(5, "#DPW#".to_string())]);
    }

    /// A near miss must not leave anything behind, or the next `#` would resume a
    /// stale buffer and fire something the user never typed.
    #[test]
    fn a_wrong_trigger_is_discarded() {
        let mut h = with(&["#DPW#"]);
        assert!(feed(&mut h, "#XYZ#").is_empty());
        // The trailing `#` legitimately opens a fresh attempt, so the buffer
        // holding just the leader is correct. What must be gone is the failed
        // attempt itself — otherwise a later `DPW#` could complete against junk.
        assert!(
            !h.buffer.to_lowercase().contains(['x', 'y', 'z']),
            "kept part of the failed attempt: {:?}",
            h.buffer
        );
        assert!(h.buffer.chars().count() <= 1, "held {:?}", h.buffer);
    }

    /// `#no#DPW#` — the second leader has to start a fresh attempt.
    #[test]
    fn a_leader_inside_a_failed_attempt_starts_over() {
        let mut h = with(&["#DPW#"]);
        assert_eq!(feed(&mut h, "#no#DPW#"), vec![(5, "#DPW#".to_string())]);
    }

    #[test]
    fn two_leaders_in_a_row_still_work() {
        let mut h = with(&["#DPW#"]);
        assert_eq!(feed(&mut h, "##DPW#").len(), 1);
    }

    /// Several triggers sharing a prefix must each resolve to themselves.
    #[test]
    fn a_shared_prefix_resolves_to_the_right_trigger() {
        let mut h = with(&["#PW#", "#DPW#", "#QPW#"]);
        assert_eq!(feed(&mut h, "#PW#"), vec![(4, "#PW#".to_string())]);
        assert_eq!(feed(&mut h, "#DPW#"), vec![(5, "#DPW#".to_string())]);
        assert_eq!(feed(&mut h, "#QPW#"), vec![(5, "#QPW#".to_string())]);
    }

    /// Unbounded growth would mean holding arbitrary typing after one `#`.
    #[test]
    fn the_buffer_cannot_grow_past_the_longest_trigger() {
        let mut h = with(&["#DPW#"]);
        feed(&mut h, "#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert!(
            h.buffer.chars().count() <= 5,
            "held {} chars: {:?}",
            h.buffer.chars().count(),
            h.buffer
        );
    }

    /// The caller resets on anything that moves the cursor, because backspacing
    /// then would eat text the user did not type as part of the trigger.
    #[test]
    fn reset_abandons_a_partial_trigger() {
        let mut h = with(&["#DPW#"]);
        feed(&mut h, "#DP");
        h.reset();
        assert!(
            feed(&mut h, "W#").is_empty(),
            "should not complete after a reset"
        );
    }

    #[test]
    fn a_trigger_without_the_leader_is_refused() {
        let mut h = Hotstrings::new('#');
        assert!(h.add("DPW#", action("x")).is_err());
        assert!(h.add("#", action("x")).is_err());
        assert!(h.add("#D", action("x")).is_ok());
    }

    #[test]
    fn with_nothing_registered_nothing_is_retained() {
        let mut h = Hotstrings::new('#');
        assert!(h.push('#').is_none());
        assert!(h.buffer.is_empty(), "must not buffer with no triggers");
    }
}
