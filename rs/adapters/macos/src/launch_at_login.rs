//! "Launch at Login" via a per-user LaunchAgent (~/Library/LaunchAgents).
//!
//! CapsLockX ships as a raw binary (not an .app bundle registered through
//! SMAppService), so a LaunchAgent plist with RunAtLoad is the standard way
//! to auto-start it at login. The plist file's presence on disk is the
//! source of truth for "enabled" — no separate config flag to desync.

use std::path::PathBuf;

pub const LABEL: &str = "com.snomiao.capslockx";

pub fn plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"))
}

pub fn is_enabled() -> bool {
    plist_path().exists()
}

/// Resolve the repo root from the running binary's canonical path.
/// current_exe() can return a path with literal ".." components (e.g. when
/// exec'd from bin/clx as "$DIR/clx" where $DIR itself ends in "/.."), so
/// canonicalize first. The binary may live at the repo root (`./clx`) or
/// inside the dev bundle (`dist/CapsLockX-dev.app/Contents/MacOS/clx`), so
/// walk up the ancestors until the directory looks like the checkout.
fn repo_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    exe.ancestors()
        .skip(1)
        .find(|d| d.join("build.sh").exists() && d.join("rs").is_dir())
        .map(|p| p.to_path_buf())
}

/// Prefer the lightweight `dist/CapsLockX-dev.app` wrapper (built by
/// build.sh) so the process runs from inside a real .app bundle — that's
/// what makes it show up branded ("CapsLockX" + icon) in System Settings →
/// Accessibility, instead of an unbranded raw executable. Falls back to the
/// bare binary if the bundle hasn't been built yet.
fn program_arguments() -> Vec<String> {
    let exe = std::env::current_exe().unwrap_or_else(|_| "clx".into());
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    if let Some(root) = repo_root() {
        let bundled = root.join("dist/CapsLockX-dev.app/Contents/MacOS/clx");
        if bundled.exists() {
            return vec![bundled.to_string_lossy().into_owned(), "-f".to_string()];
        }
    }
    vec![exe.to_string_lossy().into_owned(), "-f".to_string()]
}

/// Locate the versioned onnxruntime dylib that `ort`/ten-vad dlopens at
/// runtime (its filename embeds a version, e.g. libonnxruntime.1.23.2.dylib).
/// The binary's own rpath (baked in by build.sh) already covers regular
/// dylib linking; this env var is only needed for the dlopen path.
fn find_ort_dylib() -> Option<PathBuf> {
    let dir = repo_root()?.join("rs/target/release");
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            (name.starts_with("libonnxruntime.") && name.ends_with(".dylib")).then(|| e.path())
        })
}

fn gui_domain() -> String {
    let uid = unsafe { libc::getuid() };
    format!("gui/{uid}")
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn plist_xml() -> String {
    let args_xml: String = program_arguments()
        .iter()
        .map(|a| format!("        <string>{}</string>\n", xml_escape(a)))
        .collect();

    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/Logs/CapsLockX");
    let _ = std::fs::create_dir_all(&log_dir);
    let out_log = xml_escape(&log_dir.join("launchd.out.log").to_string_lossy());
    let err_log = xml_escape(&log_dir.join("launchd.err.log").to_string_lossy());

    let env_xml = match find_ort_dylib() {
        Some(p) => format!(
            "    <key>EnvironmentVariables</key>\n    <dict>\n        <key>ORT_DYLIB_PATH</key><string>{}</string>\n    </dict>\n",
            xml_escape(&p.to_string_lossy())
        ),
        None => String::new(),
    };

    // NOTE: deliberately NO `KeepAlive`. clx has its own startup dedup
    // (main.rs kills any other `clx -f` instance), so a launchd auto-restart
    // fights it: a manually-launched dev instance kills the launchd one, then
    // KeepAlive resurrects the launchd one, which dedups the manual one right
    // back — an endless swap that always leaves the (possibly permission-
    // starved) launchd instance as the survivor. `RunAtLoad` alone gives
    // launch-at-login without that war; crash-restart, if ever wanted, must be
    // reconciled with dedup as a separate deliberate change.
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
{args_xml}    </array>
    <key>RunAtLoad</key><true/>
{env_xml}    <key>ProcessType</key><string>Interactive</string>
    <key>StandardOutPath</key><string>{out_log}</string>
    <key>StandardErrorPath</key><string>{err_log}</string>
</dict>
</plist>
"#
    )
}

/// Write the LaunchAgent plist so clx auto-starts at the next login.
///
/// Deliberately does NOT `launchctl bootstrap`/start the job now. When the
/// user flips this toggle, clx is — by definition — already running (the
/// toggle lives in clx's own tray menu). Starting the launchd-managed
/// instance immediately would spawn a second clx whose startup dedup
/// (main.rs) kills the one the user is actively using, and — if the launchd
/// target hasn't been granted Accessibility yet — leaves them with a dead,
/// permission-blocked instance. So "enable" just registers for next login;
/// the running instance is left untouched. The plist-on-disk is the
/// source-of-truth for the checkbox state (`is_enabled`).
pub fn enable() {
    let path = plist_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&path, plist_xml()) {
        eprintln!("[CLX] launch-at-login: failed to write plist: {e}");
        return;
    }
    eprintln!(
        "[CLX] launch-at-login: enabled (starts at next login) — wrote {}",
        path.display()
    );
}

/// Stop the launchd-managed instance (if any) and remove the plist.
pub fn disable() {
    let domain = gui_domain();
    let _ = std::process::Command::new("launchctl")
        .args(["bootout", &format!("{domain}/{LABEL}")])
        .output();

    let path = plist_path();
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            eprintln!("[CLX] launch-at-login: failed to remove plist: {e}");
            return;
        }
    }
    eprintln!("[CLX] launch-at-login: disabled");
}

pub fn set_enabled(enabled: bool) {
    if enabled {
        enable();
    } else {
        disable();
    }
}

/// Flip the current state and return the new state.
pub fn toggle() -> bool {
    let now = !is_enabled();
    set_enabled(now);
    now
}
