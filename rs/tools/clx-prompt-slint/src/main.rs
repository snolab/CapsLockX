//! clx-prompt (Slint) — CapsLockX brainstorm prompt dialog.
//!
//! Argv contract (matches the legacy AppKit binary so CLX needs no change):
//!   clx-prompt <title> <context> <prefill>
//!
//! Output:
//!   stdout: "<text>" or "[KEEP]\n<text>" then exit 0
//!   exit code 1 = cancelled

use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

const HIST_PATH_ENV: &str = "CLX_PROMPT_HIST";

fn hist_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var(HIST_PATH_ENV) {
        return std::path::PathBuf::from(p);
    }
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("CapsLockX").join("brainstorm_history.txt")
}

fn hist_count() -> i32 {
    std::fs::read_to_string(hist_path())
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count() as i32)
        .unwrap_or(0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let title   = args.get(1).cloned().unwrap_or_else(|| "CapsLockX Brainstorm".into());
    let context = args.get(2).cloned().unwrap_or_default();
    let prefill = args.get(3).cloned().unwrap_or_default();

    let win = PromptWindow::new()?;
    win.set_window_title(title.into());
    win.set_context_text(context.into());
    win.set_prompt_text(prefill.into());
    win.set_hist_count(hist_count());

    let result: Rc<RefCell<Option<(String, bool)>>> = Rc::new(RefCell::new(None));

    {
        let result = result.clone();
        let win_weak = win.as_weak();
        win.on_submitted(move |text, keep| {
            let t = text.trim().to_string();
            if t.is_empty() { return; }
            *result.borrow_mut() = Some((t, keep));
            if let Some(w) = win_weak.upgrade() { let _ = w.hide(); }
        });
    }

    {
        let win_weak = win.as_weak();
        win.on_cancelled(move || {
            if let Some(w) = win_weak.upgrade() { let _ = w.hide(); }
        });
    }

    win.run()?;

    let final_result = result.borrow().clone();
    match final_result {
        Some((text, keep)) => {
            if keep { println!("[KEEP]\n{text}"); } else { println!("{text}"); }
            Ok(())
        }
        None => std::process::exit(1),
    }
}
