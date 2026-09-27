//! Evaluate a snippet of JavaScript, in-process.
//!
//! `rquickjs` natively, `boa` on wasm32 — both already compiled in, so this costs
//! nothing new. It lived inside `agent.rs` as a private helper for the LLM's
//! `js_eval` tool; it is here because a keystroke wants it too (`CLX+=`), and a
//! keyboard module has no business depending on the agent.
//!
//! In-process matters. The AutoHotkey version of this feature launches a Node
//! server (`Modules/CLX-NodeEval.ahk:143`, `node --inspect`) and talks to it — a
//! second runtime to install, a process to supervise, and a listening port. None
//! of that is needed to evaluate `1+1`.

use std::time::Duration;

/// Evaluate `code` and return its result as a string.
///
/// `Err` carries a message fit to show a user; there is no separate error type
/// because every caller so far wants to display whatever went wrong rather than
/// branch on it.
///
/// The timeout is enforced by an interrupt handler rather than a thread kill, so
/// a runaway loop is stopped rather than leaked.
#[cfg(not(target_arch = "wasm32"))]
pub fn eval(code: &str, timeout: Duration) -> Result<String, String> {
    use rquickjs::{Context, Runtime};

    let rt = Runtime::new().map_err(|e| format!("JS runtime error: {e:?}"))?;

    let deadline = std::time::Instant::now() + timeout;
    rt.set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)));

    let ctx = Context::full(&rt).map_err(|e| format!("JS context error: {e:?}"))?;

    ctx.with(|ctx| {
        // `String(eval(<json>))` — the snippet travels as a JSON string literal so
        // quotes and newlines in it cannot terminate the expression early.
        let wrapped = format!("String(eval({}))", serde_json::json!(code));
        match ctx.eval::<String, _>(wrapped.as_bytes()) {
            Ok(s) => Ok(s),
            Err(e) => {
                // Classified by the clock, not by the message. The inherited
                // version matched on "interrupted"/"InternalError", and this
                // rquickjs reports a plain `Exception` for an interrupt — so a
                // timed-out snippet was being reported as a syntax error, in the
                // agent too. Whether we are past the deadline is not a guess.
                if std::time::Instant::now() > deadline {
                    // Sub-second timeouts exist (the keystroke path uses one), and
                    // "timed out after 0s" is not a message worth showing anyone.
                    let how_long = if timeout.as_secs() > 0 {
                        format!("{}s", timeout.as_secs())
                    } else {
                        format!("{}ms", timeout.as_millis())
                    };
                    Err(format!(
                        "Execution timed out after {how_long}. The code ran too long. \
                         Try a smaller computation, or break it into smaller steps."
                    ))
                } else {
                    // `{e:?}` on its own is the bare word "Exception", which tells
                    // the user nothing. The actual thrown value has to be pulled
                    // out of the context, and it is the whole point of the
                    // message: "asdf is not defined" versus "Exception".
                    Err(describe_exception(&ctx, &e))
                }
            }
        }
    })
}

/// Evaluate `code` and return its result as JSON.
///
/// The difference from [`eval`] is the wrapper: `String(...)` turns an array of
/// objects into `"[object Object]"`, which is useless for a script whose whole
/// job is to describe what should happen. `JSON.stringify` keeps the structure.
///
/// Scripts get a `clx` global — see [`install_host_api`]. Its contents are the
/// security boundary, so they are a deliberate, short list.
#[cfg(not(target_arch = "wasm32"))]
pub fn eval_json(code: &str, args: &[String], timeout: Duration) -> Result<String, String> {
    use rquickjs::{Context, Runtime};

    let rt = Runtime::new().map_err(|e| format!("JS runtime error: {e:?}"))?;
    let deadline = std::time::Instant::now() + timeout;
    rt.set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)));
    let ctx = Context::full(&rt).map_err(|e| format!("JS context error: {e:?}"))?;

    ctx.with(|ctx| {
        install_host_api(&ctx, args)?;
        let wrapped = format!("JSON.stringify(eval({}))", serde_json::json!(code));
        match ctx.eval::<String, _>(wrapped.as_bytes()) {
            Ok(json) => Ok(json),
            Err(e) => {
                if std::time::Instant::now() > deadline {
                    Err(format!("script timed out after {}ms", timeout.as_millis()))
                } else {
                    Err(describe_exception(&ctx, &e))
                }
            }
        }
    })
}

/// Everything a script is allowed to reach.
///
/// Short on purpose: each entry is a capability granted to every script that ever
/// runs, and the refusals matter as much as the grants. No network and no
/// filesystem, because clipboard-read plus network is an exfiltration path in two
/// capabilities; no reading keystrokes, because inside a process that already
/// holds a global keyboard hook that is a keylogger with extra steps. See
/// `lab/script-store`.
#[cfg(not(target_arch = "wasm32"))]
fn install_host_api(ctx: &rquickjs::Ctx<'_>, args: &[String]) -> Result<(), String> {
    use rquickjs::{Function, Object};

    let clx = Object::new(ctx.clone()).map_err(|e| format!("host API: {e:?}"))?;

    // The OS CSPRNG. Not a convenience: `Math.random()` is not cryptographically
    // secure, and a scripted password generator built on it would be weaker than
    // the AHK implementation this replaces — whose clock-seeded Mersenne Twister
    // was the specific defect worth fixing.
    let random = Function::new(ctx.clone(), |n: usize| -> rquickjs::Result<Vec<u8>> {
        // Bounded: a script asking for a gigabyte of randomness is a bug, and the
        // host should not oblige it.
        let n = n.min(4096);
        let mut bytes = vec![0u8; n];
        getrandom::getrandom(&mut bytes).map_err(|_| rquickjs::Error::Unknown)?;
        Ok(bytes)
    })
    .map_err(|e| format!("host API: {e:?}"))?;
    clx.set("random", random)
        .map_err(|e| format!("host API: {e:?}"))?;

    // Whatever followed the script path in the binding, so one script can serve
    // several gestures (`genpw.js dpw` and `genpw.js qpw`).
    clx.set("args", args.to_vec())
        .map_err(|e| format!("host API: {e:?}"))?;

    ctx.globals()
        .set("clx", clx)
        .map_err(|e| format!("host API: {e:?}"))
}

/// Turn a failed `eval` into something worth showing a person.
///
/// `rquickjs` returns `Error::Exception` as a marker — the thrown value stays in
/// the context and has to be taken out with `catch`. Without this the user sees
/// the literal word "Exception" instead of "asdf is not defined", which is the
/// difference between feedback and noise.
#[cfg(not(target_arch = "wasm32"))]
fn describe_exception(ctx: &rquickjs::Ctx<'_>, fallback: &rquickjs::Error) -> String {
    let thrown = ctx.catch();

    // The common case: a real Error object, which knows its own name and message.
    //
    // Only the first line. QuickJS appends a stack trace of its own internals —
    // `at <eval> (eval_script:1:7)` — and this message gets pasted into the
    // user's document, where clx's implementation details have no business.
    if let Some(exception) = thrown.as_exception() {
        let text = exception.to_string();
        if let Some(first) = first_line(&text) {
            return first;
        }
        if let Some(message) = exception.message().as_deref().and_then(first_line) {
            return message;
        }
    }

    // `throw "a string"` and friends: not an Error, still has a value.
    if let Ok(text) = thrown.get::<String>() {
        if let Some(first) = first_line(&text) {
            return first;
        }
    }

    format!("JS error: {fallback:?}")
}

/// The first non-empty line, trimmed. `None` if there isn't one.
#[cfg(not(target_arch = "wasm32"))]
fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(target_arch = "wasm32")]
pub fn eval(code: &str, _timeout: Duration) -> Result<String, String> {
    use boa_engine::{Context, Source};

    let mut context = Context::default();
    match context.eval(Source::from_bytes(code)) {
        Ok(v) => v
            .to_string(&mut context)
            .map(|s| s.to_std_string_escaped())
            .map_err(|e| format!("JS error: {e:?}")),
        Err(e) => Err(format!("JS error: {e:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(code: &str) -> String {
        eval(code, Duration::from_secs(2)).expect("should evaluate")
    }

    #[test]
    fn arithmetic_and_strings() {
        assert_eq!(ok("1+1"), "2");
        assert_eq!(ok("[1,2,3].map(x => x * 2).join(',')"), "2,4,6");
        assert_eq!(ok("'a'.repeat(3)"), "aaa");
    }

    /// The snippet is embedded in a wrapper expression, so anything that could
    /// close that expression early has to survive intact.
    #[test]
    fn quotes_and_newlines_in_the_snippet_survive() {
        assert_eq!(ok("\"he said \\\"hi\\\"\""), "he said \"hi\"");
        assert_eq!(ok("1 +\n  2"), "3");
        assert_eq!(ok("')' + '\"'"), ")\"");
    }

    #[test]
    fn a_syntax_error_is_reported_not_panicked() {
        let err = eval("1 +", Duration::from_secs(2)).expect_err("should fail");
        assert!(err.contains("unexpected token"), "{err}");
    }

    /// The message a user sees has to name the actual problem. This used to read
    /// "JS error: Exception", which is the kind of feedback that is worse than
    /// none: it tells you something failed and nothing about what.
    #[test]
    fn an_undefined_name_says_so() {
        let err = eval("asdf", Duration::from_secs(2)).expect_err("should fail");
        assert!(err.contains("asdf is not defined"), "{err}");
    }

    /// It gets pasted into the user's document, so clx's internals must not ride
    /// along — QuickJS appends `at <eval> (eval_script:1:7)` and similar.
    #[test]
    fn the_message_carries_no_stack_trace() {
        for code in ["asdf", "null.x", "1 +"] {
            let err = eval(code, Duration::from_secs(2)).expect_err("should fail");
            assert!(!err.contains('\n'), "{code}: multi-line message {err:?}");
            assert!(
                !err.contains("eval_script"),
                "{code}: leaked internals {err:?}"
            );
            assert!(!err.contains("<input>"), "{code}: leaked internals {err:?}");
        }
    }

    /// `throw "a string"` is not an Error object but still has something to say.
    #[test]
    fn a_thrown_string_is_reported_verbatim() {
        let err = eval("throw 'plain string'", Duration::from_secs(2)).expect_err("should fail");
        assert_eq!(err, "plain string");
    }

    /// A runaway loop must be interrupted, not leaked — this is the difference
    /// between a timeout and a wedged clx.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn an_infinite_loop_is_interrupted() {
        let started = std::time::Instant::now();
        let err = eval("while (true) {}", Duration::from_millis(300)).expect_err("should time out");
        assert!(err.contains("timed out"), "{err}");
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "took {:?} — the interrupt handler is not firing",
            started.elapsed()
        );
    }
}
