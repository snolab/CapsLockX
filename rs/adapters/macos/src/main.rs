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
mod system_audio;
#[cfg(target_os = "macos")]
mod voice_capture;
#[cfg(target_os = "macos")]
mod brainstorm_overlay;
#[cfg(target_os = "macos")]
mod agent_cmd;
#[cfg(target_os = "macos")]
mod mic_mode;
#[cfg(target_os = "macos")]
mod audio_tap;
#[cfg(target_os = "macos")]
mod observe_cmd;

#[cfg(target_os = "macos")]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("agent") => {
            agent_cmd::main(&args[2..]);
            return;
        }
        Some("dino") => {
            agent_cmd::main(&["dino".to_string()]);
            return;
        }
        Some("observe") => {
            observe_cmd::main(&args[2..].to_vec());
            return;
        }
        Some("--help") | Some("-h") | Some("help") => {
            println!("CapsLockX — keyboard productivity tool + LLM agent");
            println!();
            println!("USAGE:");
            println!("  clx                         Start CapsLockX (forks to background)");
            println!("  clx -f                      Start in foreground (blocks shell)");
            println!("  clx agent --tree            Dump accessibility tree of frontmost app");
            println!("  clx agent --exec            Execute CLX commands from stdin");
            println!("  clx agent --prompt \"task\"    Run LLM agent to perform a task");
            println!("  clx agent \"task\"             Shorthand for --prompt");
            println!("  clx observe                  Capture screen + Gemini vision description");
            println!("  clx observe --help           Show observe options");
            println!("  clx --help                  Show this help");
            println!();
            println!("CLX AGENT COMMANDS:");
            println!("  k a          tap key 'a'       m 400 300    move mouse");
            println!("  k c-c        Ctrl+C            m 400 300 c  move + click");
            println!("  k \"text\"     type string       w 200ms      wait 200ms");
            println!();
            println!("HOTKEYS (while CapsLock or Space held):");
            println!("  HJKL         cursor movement    WASD        mouse movement");
            println!("  B            brainstorm (AI)    M           agent (AI control)");
            println!("  V            voice input        1-9         virtual desktops");
            return;
        }
        _ => {}
    }

    // --foreground / -f: run in foreground (block the shell).
    let foreground = args.iter().any(|a| a == "--foreground" || a == "-f");

    if !foreground {
        // Spawn a new process (not fork — fork + ObjC/AppKit = crash).
        // The new process runs with -f (foreground) so it doesn't re-spawn.
        let exe = std::env::current_exe().unwrap_or_else(|_| "clx".into());
        match std::process::Command::new(&exe)
            .arg("-f")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
        {
            Ok(child) => {
                eprintln!("[CLX] started (pid {})", child.id());
                return;
            }
            Err(e) => {
                eprintln!("[CLX] spawn failed ({}), running in foreground", e);
            }
        }
    }

    // Kill any existing clx daemon instance (deduplicate).
    // Match by executable path — works regardless of how clx was invoked.
    {
        let my_pid = std::process::id();
        let exe = std::env::current_exe().unwrap_or_default();
        let exe_str = exe.to_string_lossy();
        eprintln!("[CLX] dedup: my_pid={} exe={}", my_pid, exe_str);

        // Use pgrep to find processes with the same executable name.
        if let Ok(output) = std::process::Command::new("pgrep").arg("-x").arg("clx").output() {
            let pids = String::from_utf8_lossy(&output.stdout);
            for line in pids.lines() {
                if let Ok(pid) = line.trim().parse::<u32>() {
                    if pid != my_pid {
                        eprintln!("[CLX] dedup: killing old instance pid={}", pid);
                        let _ = std::process::Command::new("kill").arg("-9").arg(pid.to_string()).status();
                    }
                }
            }
        }
        // Also try matching by path pattern for when launched as ./clx or /path/to/clx
        let _ = std::process::Command::new("sh")
            .args(["-c", &format!(
                "pgrep -f '/clx\\b|/capslockx\\b' | grep -v {} | xargs kill -9 2>/dev/null", my_pid
            )])
            .status();
    }

    // Tee stderr to /tmp/clx-debug.log (truncated on each start) so the log
    // file is always populated regardless of how CLX was launched.
    {
        use std::os::unix::io::{AsRawFd, FromRawFd};
        extern "C" {
            fn dup(fd: i32) -> i32;
            fn dup2(old: i32, new: i32) -> i32;
            fn pipe(fds: *mut i32) -> i32;
            fn close(fd: i32) -> i32;
        }
        let log_path = "/tmp/clx-debug.log";
        if let Ok(log_file) = std::fs::File::create(log_path) {
            let stderr_fd = std::io::stderr().as_raw_fd();
            let orig_copy = unsafe { dup(stderr_fd) };
            let log_fd = log_file.as_raw_fd();
            let mut pipe_fds = [0i32; 2];
            if unsafe { pipe(pipe_fds.as_mut_ptr()) } == 0 {
                unsafe { dup2(pipe_fds[1], stderr_fd); }
                unsafe { close(pipe_fds[1]); }
                let reader_fd = pipe_fds[0];
                let log_fd_copy = unsafe { dup(log_fd) };
                std::thread::Builder::new().name("clx-log-tee".into()).spawn(move || {
                    let mut reader = unsafe { std::fs::File::from_raw_fd(reader_fd) };
                    let mut orig_w = unsafe { std::fs::File::from_raw_fd(orig_copy) };
                    let mut log_w = unsafe { std::fs::File::from_raw_fd(log_fd_copy) };
                    let mut buf = [0u8; 4096];
                    loop {
                        use std::io::Read;
                        match reader.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let _ = std::io::Write::write_all(&mut orig_w, &buf[..n]);
                                let _ = std::io::Write::write_all(&mut log_w, &buf[..n]);
                            }
                        }
                    }
                }).ok();
            }
        }
    }

    // Install panic hook to log panic messages before abort.
    // IMPORTANT: Do NOT use eprintln! here — if stderr is a broken pipe,
    // eprintln! panics, causing a double-panic → abort with no diagnostics.
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() { s.to_string() }
                  else if let Some(s) = info.payload().downcast_ref::<String>() { s.clone() }
                  else { "unknown".to_string() };
        let loc = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_default();
        // Try stderr but ignore errors (broken pipe safe).
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            format!("[CLX] PANIC: {} at {}\n", msg, loc).as_bytes(),
        );
        // Also write to a crash file for post-mortem.
        let crash_path = std::env::current_exe().ok()
            .and_then(|e| e.parent().map(|p| p.join("tmp/last-panic.txt")));
        if let Some(p) = crash_path {
            let _ = std::fs::write(&p, format!("{} at {}\n", msg, loc));
        }
    }));

    eprintln!("[CLX] CapsLockX macOS adapter starting…");
    eprintln!("[CLX] running – hold CapsLock/Space to activate");
    eprintln!("[CLX] send SIGINT (Ctrl+C) or `pkill clx` to exit");

    // Install the menu bar icon and voice overlay class before entering the run loop.
    tray::setup_tray();
    voice_overlay::init_overlay();

    // Prompt user to enable Voice Isolation if not active.
    mic_mode::ensure_voice_isolation();

    // Install CGEventTap and block on CFRunLoop.
    hook::install_and_run();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("[CLX] Error: This binary only runs on macOS.");
    std::process::exit(1);
}
