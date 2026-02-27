//! CapsLockX – Linux adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-linux
//! Build (release): cargo build -p capslockx-linux --release
//!
//! Requires membership in the `input` group:
//!   sudo usermod -aG input $USER
//!   # then log out and back in

#[cfg(target_os = "linux")]
mod hook;
#[cfg(target_os = "linux")]
mod key_map;
#[cfg(target_os = "linux")]
mod output;

#[cfg(target_os = "linux")]
fn main() {
    eprintln!("[CLX] CapsLockX Linux adapter starting\u{2026}");

    // Create the uinput virtual device before starting any hook threads.
    output::init();

    // Enumerate physical keyboards, grab each one, start event threads.
    hook::install_hooks();

    eprintln!("[CLX] running \u{2013} hold CapsLock/Space to activate");
    eprintln!("[CLX] send SIGINT (Ctrl+C) to exit");

    // Block the main thread forever; hook threads keep the process alive.
    loop {
        std::thread::park();
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("[CLX] Error: This binary only runs on Linux.");
    std::process::exit(1);
}
