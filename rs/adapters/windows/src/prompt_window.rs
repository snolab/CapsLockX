//! Out-of-process brainstorm prompt window (`clx prompt-window`).
//!
//! Mirrors the macOS `clx-prompt` subprocess: shows a focused text input so the
//! user can type their CLX+B question, then hands the result back to the hook
//! process. Runs as its OWN process because a focused WebView2 window in the hook
//! process kills WH_KEYBOARD_LL (see `prefs_window.rs`).
//!
//! Result channel is a temp FILE, not stdout: WebView2 spawns Chromium helper
//! processes that can inherit a stdout pipe handle and make the parent's
//! `.output()` hang. The parent passes a path; we write the result there on
//! submit. A "Keep history" checkbox prepends the `[KEEP]\n` sentinel the core
//! brainstorm module parses.

use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use tauri::webview::WebviewWindowBuilder;
use tauri::{Manager as _, WebviewUrl};

#[derive(Default, Clone, serde::Serialize)]
struct PromptArgs {
    title: String,
    message: String,
    prefill: String,
}

static PROMPT_ARGS: Lazy<Mutex<PromptArgs>> = Lazy::new(|| Mutex::new(PromptArgs::default()));
static OUT_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::new()));

/// Frontend fetches the title/context/prefill to render.
#[tauri::command]
fn get_prompt_args() -> PromptArgs {
    PROMPT_ARGS.lock().unwrap().clone()
}

/// User submitted: write the (optionally `[KEEP]`-prefixed) text to the result
/// file and exit. The file is the only signal the parent reads.
#[tauri::command]
fn submit(app: tauri::AppHandle, text: String, keep: bool) {
    let out = if keep {
        format!("[KEEP]\n{text}")
    } else {
        text
    };
    let path = OUT_PATH.lock().unwrap().clone();
    let _ = std::fs::write(&path, out.as_bytes());
    app.exit(0);
}

/// User cancelled (Esc / window close): write nothing → parent treats as None.
#[tauri::command]
fn cancel(app: tauri::AppHandle) {
    app.exit(0);
}

/// Entry point for the `prompt-window` subcommand. Blocks until submit/cancel.
pub fn run(title: String, message: String, prefill: String, out_path: String) {
    let title_for_window = if title.is_empty() {
        "CapsLockX Brainstorm".to_string()
    } else {
        title.clone()
    };
    *PROMPT_ARGS.lock().unwrap() = PromptArgs {
        title,
        message,
        prefill,
    };
    *OUT_PATH.lock().unwrap() = PathBuf::from(out_path);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_prompt_args, submit, cancel])
        .setup(move |app| {
            WebviewWindowBuilder::new(
                app.handle(),
                "prompt",
                WebviewUrl::App("prompt.html".into()),
            )
            .title(title_for_window)
            .inner_size(620.0, 340.0)
            .resizable(true)
            .always_on_top(true)
            .center()
            .focused(true)
            .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("tauri build error (prompt window)")
        // Closing the window (X) ends the subprocess with no file written, which
        // the parent reads as a cancel.
        .run(|_app, _event| {});
}
