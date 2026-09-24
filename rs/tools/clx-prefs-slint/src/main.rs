//! Standalone launcher for `clx-prefs-slint`.
//!
//! The window itself lives in this crate's library so `clx.exe` can render it
//! in-process (via `clx prefs-window`)
//! and the portable build can ship a single executable. This binary stays so
//! the tool can still be run and iterated on by itself.

#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    clx_prefs_slint::run(&args)
}
