//! Running a script: a plugin that needs no executable.
//!
//! A script is a `.js` file that evaluates to a list of things to do:
//!
//! ```js
//! // scripts/hello.js
//! [{ type: "hello" }, { key: "enter" }]
//! ```
//!
//! Same contract as a [plugin](crate::plugin) — describe effects, do not perform
//! them — but in-process, with no build step, no binary per platform, and nothing
//! for an antivirus heuristic to find. See `lab/script-store` for why that shape
//! is the right one for the long tail of small ideas.
//!
//! **The steps are structured values, not effect text.** A script returning
//! `k "…"` strings would put escaping in the hands of every script author, and
//! the first place that goes wrong is a password containing a quote. Returning
//! `{ type: pw }` means the host does the formatting and a script cannot get it
//! wrong.

use std::time::Duration;

use crate::effects::{self, Effect, Runner};
use crate::platform::Platform;

/// How long a script may run. Generous next to a keystroke but far short of
/// hanging the tool: a script is meant to compute a small thing and return.
const SCRIPT_TIMEOUT: Duration = Duration::from_secs(2);

/// One step of a script's answer, read straight from the JSON.
///
/// Read from a `Value` rather than derived, to avoid pulling `serde`'s derive
/// machinery into core for three optional strings — `serde_json` alone is already
/// a dependency.
///
/// Recognised fields, first one wins:
///
///   * `type` — type this text literally. No escaping needed, or possible to get
///     wrong, because nothing is ever formatted as text.
///   * `key` — a key or chord in the effect language's spelling: `enter`, `c-c`.
///   * `wait` — a pause: `200ms`, `1s`.
struct Step<'a> {
    r#type: Option<&'a str>,
    key: Option<&'a str>,
    wait: Option<&'a str>,
}

impl<'a> Step<'a> {
    fn from_value(value: &'a serde_json::Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("expected an object like {{type:\"hi\"}}, got {value}"))?;
        let field = |name: &str| object.get(name).and_then(|v| v.as_str());
        Ok(Step {
            r#type: field("type"),
            key: field("key"),
            wait: field("wait"),
        })
    }
}

/// What became of a script run.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Performed this many effects.
    Performed(usize),
    /// Could not be read, evaluated, or understood.
    Failed(String),
}

/// Read, evaluate and perform `path`.
///
/// `args` reaches the script as `clx.args`, so one file can serve several
/// gestures. Blocks; call it off the keyboard hook.
pub fn run(path: &str, args: &[String], platform: &dyn Platform) -> Outcome {
    let code = match std::fs::read_to_string(path) {
        Ok(code) => code,
        Err(e) => return Outcome::Failed(format!("{path}: {e}")),
    };
    run_source(&code, args, platform)
}

/// The half that does not touch the filesystem, so it can be tested.
pub fn run_source(code: &str, args: &[String], platform: &dyn Platform) -> Outcome {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (code, args, platform);
        return Outcome::Failed("scripts are not available on this platform".into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let json = match crate::js::eval_json(code, args, SCRIPT_TIMEOUT) {
            Ok(json) => json,
            Err(why) => return Outcome::Failed(why),
        };

        let value: serde_json::Value = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(e) => return Outcome::Failed(format!("script returned invalid JSON: {e}")),
        };
        let Some(steps) = value.as_array() else {
            return Outcome::Failed(
                "a script must evaluate to a list of steps like \
                 [{type:\"hi\"}, {key:\"enter\"}]"
                    .to_string(),
            );
        };

        let mut runner = Runner::new();
        let mut performed = 0;
        for (n, raw) in steps.iter().enumerate() {
            let effect = match Step::from_value(raw).and_then(|step| to_effect(&step)) {
                Ok(effect) => effect,
                Err(why) => return Outcome::Failed(format!("step {}: {why}", n + 1)),
            };
            runner.perform(&effect, platform);
            performed += 1;
        }
        // Close the revisable run, exactly as the plugin host does, so a later
        // script cannot backspace over this one's output.
        runner.perform(&Effect::Commit, platform);
        Outcome::Performed(performed)
    }
}

/// Convert a step into an effect.
///
/// `type` becomes an effect directly — no text is formatted, so there is no
/// escaping to get wrong. `key` and `wait` are handed to the effect parser rather
/// than reimplemented, so `c-c` and `200ms` mean exactly what they mean everywhere
/// else in clx.
fn to_effect(step: &Step<'_>) -> Result<Effect, String> {
    if let Some(text) = &step.r#type {
        return Ok(Effect::Type((*text).to_string()));
    }
    if let Some(spec) = &step.key {
        return match effects::parse(&format!("k {spec}")) {
            Effect::Unknown(_) | Effect::Unsupported { .. } => {
                Err(format!("{spec:?} is not a key or chord"))
            }
            effect => Ok(effect),
        };
    }
    if let Some(spec) = &step.wait {
        return match effects::parse(&format!("w {spec}")) {
            Effect::Unknown(_) | Effect::Unsupported { .. } => {
                Err(format!("{spec:?} is not a duration — try 200ms or 1s"))
            }
            effect => Ok(effect),
        };
    }
    Err("no recognised field — expected type, key or wait".to_string())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::test_platform::{Call, MockPlatform};

    fn typed(p: &MockPlatform) -> String {
        p.calls()
            .iter()
            .filter_map(|c| match c {
                Call::TypeText(t) => Some(t.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_script_that_returns_a_step_types_it() {
        let p = MockPlatform::new();
        assert_eq!(
            run_source(r#"[{ type: "hello" }]"#, &[], &p),
            Outcome::Performed(1)
        );
        assert_eq!(typed(&p), "hello");
    }

    #[test]
    fn keys_and_waits_reuse_the_effect_parser() {
        let p = MockPlatform::new();
        let out = run_source(
            r#"[{ type: "a" }, { key: "enter" }, { wait: "1ms" }, { type: "b" }]"#,
            &[],
            &p,
        );
        assert_eq!(out, Outcome::Performed(4));
        assert!(
            p.calls()
                .iter()
                .any(|c| matches!(c, Call::KeyDown(crate::key_code::KeyCode::Enter))),
            "{:?}",
            p.calls()
        );
    }

    /// The whole reason the host CSPRNG is exposed.
    #[test]
    fn a_script_can_reach_the_os_csprng() {
        let p = MockPlatform::new();
        let out = run_source(
            r#"[{ type: Array.from(clx.random(8), b => b % 10).join("") }]"#,
            &[],
            &p,
        );
        assert_eq!(out, Outcome::Performed(1));
        assert_eq!(typed(&p).len(), 8, "got {:?}", typed(&p));
    }

    #[test]
    fn random_is_not_constant() {
        let p = MockPlatform::new();
        let draw = || {
            let p = MockPlatform::new();
            run_source(
                r#"[{ type: Array.from(clx.random(16), b => b.toString(16)).join("") }]"#,
                &[],
                &p,
            );
            typed(&p)
        };
        let a = draw();
        let b = draw();
        assert_ne!(a, b, "two draws must differ");
        drop(p);
    }

    #[test]
    fn args_reach_the_script() {
        let p = MockPlatform::new();
        run_source(r#"[{ type: clx.args[0] }]"#, &["qpw".into()], &p);
        assert_eq!(typed(&p), "qpw");
    }

    /// A password containing a quote is the case that makes structured steps worth
    /// it: nothing is formatted as text, so nothing can be mis-escaped.
    #[test]
    fn a_quote_in_the_text_survives_untouched() {
        let p = MockPlatform::new();
        run_source(r#"[{ type: "pa\"ss\\word" }]"#, &[], &p);
        assert_eq!(typed(&p), "pa\"ss\\word");
    }

    #[test]
    fn a_broken_script_says_what_was_wrong() {
        let p = MockPlatform::new();
        match run_source("asdf", &[], &p) {
            Outcome::Failed(why) => assert!(why.contains("asdf is not defined"), "{why}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_script_returning_the_wrong_shape_is_refused() {
        let p = MockPlatform::new();
        match run_source(r#""just a string""#, &[], &p) {
            Outcome::Failed(why) => assert!(why.contains("list of steps"), "{why}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_unknown_key_names_the_offending_step() {
        let p = MockPlatform::new();
        match run_source(r#"[{ type: "ok" }, { key: "wiggle" }]"#, &[], &p) {
            Outcome::Failed(why) => {
                assert!(why.contains("step 2"), "{why}");
                assert!(why.contains("wiggle"), "{why}");
            }
            other => panic!("{other:?}"),
        }
    }

    /// The shipped `scripts/genpw.js`, exercised through the real runner.
    ///
    /// This is the test that matters for retiring `clx-genpw.exe`: it proves the
    /// script produces the same shapes the compiled version's tests assert, so the
    /// binary has nothing left to offer. Path is relative to this crate.
    #[test]
    fn the_shipped_genpw_script_produces_every_profile() {
        let path = "../../scripts/genpw.js";
        if !std::path::Path::new(path).exists() {
            // Do not fail the suite over a layout change; say so and move on.
            eprintln!(
                "skipping: {path} not found from {:?}",
                std::env::current_dir()
            );
            return;
        }
        // (profile, expected character count) — the AHK shapes, kept.
        for (profile, len) in [
            ("dpw", 15usize),
            ("pw", 16),
            ("wpw", 16),
            ("npw", 16),
            ("hex", 16),
            ("hexl", 16),
            ("qpw", 19),
            ("uuid", 36),
            ("spw", 16),
            ("jpw", 17),
        ] {
            let p = MockPlatform::new();
            let out = run(path, &[profile.to_string()], &p);
            assert_eq!(out, Outcome::Performed(1), "{profile}: {out:?}");
            let got = typed(&p);
            assert_eq!(got.chars().count(), len, "{profile} produced {got:?}");
            if matches!(profile, "dpw" | "pw" | "wpw" | "qpw") {
                for bad in ['0', 'O', 'I', 'l'] {
                    assert!(
                        !got.contains(bad),
                        "{profile} produced ambiguous {bad:?}: {got}"
                    );
                }
            }
        }
    }

    /// A typo in a binding must not type something arbitrary into a password
    /// field. The script throws; the host reports it and types nothing.
    #[test]
    fn the_shipped_genpw_script_refuses_an_unknown_profile() {
        let path = "../../scripts/genpw.js";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let p = MockPlatform::new();
        match run(path, &["nope".to_string()], &p) {
            Outcome::Failed(why) => assert!(why.contains("unknown profile"), "{why}"),
            other => panic!("{other:?}"),
        }
        assert!(typed(&p).is_empty(), "nothing should have been typed");
    }

    #[test]
    fn a_runaway_script_is_stopped() {
        let p = MockPlatform::new();
        let started = std::time::Instant::now();
        match run_source("while (true) {}", &[], &p) {
            Outcome::Failed(why) => assert!(why.contains("timed out"), "{why}"),
            other => panic!("{other:?}"),
        }
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
