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
    {
        let my_pid = std::process::id().to_string();
        let _ = std::process::Command::new("sh")
            .args(["-c", &format!(
                "pgrep -f 'CapsLockX/clx' | grep -v {} | xargs kill -9 2>/dev/null", my_pid
            )])
            .status();
    }

    eprintln!("[CLX] CapsLockX macOS adapter starting…");
    eprintln!("[CLX] running – hold CapsLock/Space to activate");
    eprintln!("[CLX] send SIGINT (Ctrl+C) or `pkill clx` to exit");

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
