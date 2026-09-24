//! Rendered either standalone or inside `clx.exe` as `clx prompt-window`.
//! Keeping it a library is what lets the portable build ship one executable.
//! clx-prompt (Slint) — CapsLockX brainstorm prompt dialog.
//!
//! Argv contract (matches the legacy AppKit binary so CLX needs no change):
//!   clx-prompt <title> <context> <prefill> [out_path]
//!
//! Output:
//!   stdout: "<text>" or "[KEEP]\n<text>" then exit 0
//!   exit code 1 = cancelled
//!
//! With a 4th `out_path` argument the result is written to that FILE instead of
//! stdout (cancelled = file left absent). That's the channel the Windows hook
//! uses — same contract as `clx prompt-window`, so the two are interchangeable.

// No console window on Windows: this is spawned from a GUI process on every
// CLX+B press and a flashing cmd box would be worse than the WebView2 it replaces.

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

/// Exit this prompt process if the parent CLX (PID in `CLX_PARENT_PID`) exits,
/// so a clx restart / self-update doesn't leave an orphaned dialog behind.
#[cfg(windows)]
fn exit_with_parent() {
    let Some(pid) = std::env::var("CLX_PARENT_PID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
    else {
        return;
    };
    std::thread::spawn(move || unsafe {
        use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::{
            OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
        };
        if let Ok(h) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            if WaitForSingleObject(h, u32::MAX) == WAIT_OBJECT_0 {
                std::process::exit(1);
            }
            let _ = CloseHandle(h);
        }
    });
}
#[cfg(not(windows))]
fn exit_with_parent() {}

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    exit_with_parent();

    let title = args
        .first()
        .cloned()
        .unwrap_or_else(|| "CapsLockX Brainstorm".into());
    let context = args.get(1).cloned().unwrap_or_default();
    let prefill = args.get(2).cloned().unwrap_or_default();
    let out_path = args
        .get(3)
        .filter(|p| !p.is_empty())
        .map(std::path::PathBuf::from);

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
            if t.is_empty() {
                return;
            }
            *result.borrow_mut() = Some((t, keep));
            if let Some(w) = win_weak.upgrade() {
                let _ = w.hide();
            }
        });
    }

    {
        let win_weak = win.as_weak();
        win.on_cancelled(move || {
            if let Some(w) = win_weak.upgrade() {
                let _ = w.hide();
            }
        });
    }

    win.run()?;

    let final_result = result.borrow().clone();
    match final_result {
        Some((text, keep)) => {
            let out = if keep {
                format!("[KEEP]\n{text}")
            } else {
                text
            };
            match out_path {
                Some(p) => std::fs::write(&p, out.as_bytes())?,
                None => println!("{out}"),
            }
            Ok(())
        }
        None => std::process::exit(1),
    }
}
