//! CapsLockX – macOS adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-macos
//! Build (release): cargo build -p capslockx-macos --release
//!
//! Requires Accessibility permission:
//!   System Settings → Privacy & Security → Accessibility

#[cfg(target_os = "macos")]
mod hook;
#[cfg(target_os = "macos")]
mod key_map;
#[cfg(target_os = "macos")]
mod output;
#[cfg(target_os = "macos")]
mod tray;
#[cfg(target_os = "macos")]
mod config_store;
#[cfg(target_os = "macos")]
mod prefs;
#[cfg(target_os = "macos")]
mod voice_overlay;

#[cfg(target_os = "macos")]
fn main() {
    eprintln!("[CLX] CapsLockX macOS adapter starting…");
    eprintln!("[CLX] running – hold CapsLock/Space to activate");
    eprintln!("[CLX] send SIGINT (Ctrl+C) to exit");

    // Install the menu bar icon and voice overlay class before entering the run loop.
    tray::setup_tray();
    voice_overlay::init_overlay();

    // Install CGEventTap and block on CFRunLoop.
    hook::install_and_run();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("[CLX] Error: This binary only runs on macOS.");
    std::process::exit(1);
}
