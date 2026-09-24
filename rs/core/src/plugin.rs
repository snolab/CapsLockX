//! Running a plugin.
//!
//! A plugin is a program that writes [effects](crate::effects) to stdout. That
//! is the whole contract — no SDK, no linking, no ABI, no manifest. Anything
//! that can print a line can extend CLX, and CLX learns nothing about what the
//! plugin is for. See `lab/plugins`.
//!
//! ```text
//! $ clx plugin -- my-plugin --some-flag
//! ```
//!
//! What this module is responsible for is the part a plugin should not have to
//! think about: reading its output as it arrives rather than at exit, keeping
//! its diagnostics separate from its effects, and leaving nothing latched when
//! it dies.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::effects::{self, Effect, Runner};
use crate::platform::Platform;

/// How a plugin run ended. A plugin that fails is a normal event, not an error
/// for the caller to handle — the binding simply did less than it hoped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Ran and exited with this code. Zero is the ordinary case.
    Exited(i32),
    /// Could not be started at all — not installed, not executable, bad path.
    NotStarted(String),
}

/// Spawn `command` and perform every effect it writes to stdout.
///
/// Blocks until the plugin exits, so call it off the keyboard hook.
///
/// Three deliberate choices:
///
/// * **stdout is effects, stderr is diagnostics.** A plugin can log freely
///   without its debug output being interpreted as commands. stderr is
///   inherited so it lands wherever CLX's own does.
/// * **Lines are performed as they arrive**, not collected first. A speech
///   plugin that only emitted at exit would be useless.
/// * **The run is committed when the plugin ends**, however it ends. Otherwise
///   a plugin that died mid-sentence would leave a revisable run open and the
///   next one could backspace over text that is no longer its own.
pub fn run(command: &str, args: &[String], platform: &dyn Platform) -> Outcome {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return Outcome::NotStarted(format!("{command}: {e}")),
    };

    let mut runner = Runner::new();
    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            runner.perform(&effects::parse(&line), platform);
        }
    }

    // Whatever happened, close the run. See the note above.
    runner.perform(&Effect::Commit, platform);

    match child.wait() {
        Ok(status) => Outcome::Exited(status.code().unwrap_or(-1)),
        Err(e) => Outcome::NotStarted(format!("{command}: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    /// Drive a "plugin" without needing one on disk. The effects travel via a
    /// file rather than a shell argument on purpose: quoting rules differ per
    /// shell, and what is being tested is the contract, not cmd.exe.
    fn echo(lines: &[&str]) -> (Outcome, MockPlatform) {
        use std::io::Write as _;
        let p = MockPlatform::new();
        // Unique per call: tests run in parallel and two fixtures of the same
        // length would otherwise write to the same file and corrupt each other.
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "clx-plugin-test-{}-{}.txt",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        {
            let mut f = std::fs::File::create(&path).expect("temp script");
            for line in lines {
                writeln!(f, "{line}").expect("write");
            }
        }
        let path_str = path.to_string_lossy().to_string();
        // `findstr` rather than `cmd /C type`: invoking cmd re-parses the
        // arguments with its own quoting rules and mangled the effects.
        // Spawning a real executable directly does not.
        let (cmd, args) = if cfg!(windows) {
            (
                "findstr".to_string(),
                vec!["/v".into(), "^~~nomatch~~".into(), path_str],
            )
        } else {
            ("cat".to_string(), vec![path_str])
        };
        let outcome = run(&cmd, &args, &p);
        let _ = std::fs::remove_file(&path);
        (outcome, p)
    }

    fn typed(p: &MockPlatform) -> String {
        p.calls
            .lock()
            .unwrap()
            .iter()
            .filter_map(|c| match c {
                Call::TypeText(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_program_that_prints_an_effect_is_a_plugin() {
        let (outcome, p) = echo(&[r#"k "hello from a plugin""#]);
        assert_eq!(outcome, Outcome::Exited(0));
        assert!(
            typed(&p).contains("hello from a plugin"),
            "got {:?}",
            typed(&p)
        );
    }

    #[test]
    fn a_missing_plugin_is_an_outcome_not_a_panic() {
        let p = MockPlatform::new();
        match run("definitely-not-a-real-program-xyz", &[], &p) {
            Outcome::NotStarted(_) => {}
            other => panic!("expected NotStarted, got {other:?}"),
        }
    }

    #[test]
    fn nothing_is_left_latched_when_a_plugin_stops() {
        // The plugin leaves a revisable run open by never committing.
        let (_, p) = echo(&[r#"k "draft""#]);
        // A second run must type fresh rather than revise the first one's text.
        let mut r = Runner::new();
        r.perform(&Effect::Retype("second".into()), &p);
        let calls = p.calls.lock().unwrap();
        assert!(
            !calls.iter().any(
                |c| matches!(c, Call::KeyDown(k) if *k == crate::key_code::KeyCode::Backspace)
            ),
            "a new run must not backspace over the previous plugin's output"
        );
    }
}
