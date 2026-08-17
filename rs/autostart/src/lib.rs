//! "Launch at login" for CapsLockX on Windows, via a Task Scheduler logon task.
//!
//! # Why a scheduled task and not a Startup-folder shortcut
//!
//! clx wants to run elevated: an unelevated `WH_KEYBOARD_LL` hook can't see keys
//! while an elevated window has focus, so hotkeys die whenever you focus an
//! admin console. A Startup-folder entry can't launch elevated without throwing
//! a UAC prompt at *every* login; a logon task with `/rl highest` runs elevated
//! silently. (The AHK implementation learned this the hard way and ended up with
//! two competing autostart buttons — see `Modules/CLX-RunOnLogin.ahk`.)
//!
//! # Source of truth
//!
//! The task's existence in Task Scheduler *is* the enabled state. There is no
//! mirror flag in config.json, so the checkbox can never disagree with reality —
//! including when the user removes the task by hand via `taskschd.msc`.
//!
//! # Why an XML task definition instead of plain `schtasks /create`
//!
//! The task MUST set a working directory, and the `/create` command line has no
//! flag for one. clx really does depend on its cwd: `spawn_ahk()` in the Windows
//! adapter resolves `.\Core\capslockx-ahkv1.exe` relative to the current
//! directory, so a task launched with the scheduler's default cwd
//! (`C:\Windows\system32`) starts clx with every AHK module silently disabled.
//! `schtasks /create /xml` is the only way to express `<WorkingDirectory>`.

/// Task Scheduler name for the logon task. Distinct from the AHK build's
/// `CapsLockX_AutoStart`, so both implementations can coexist and be managed
/// independently rather than silently clobbering each other.
pub const TASK_NAME: &str = "CapsLockX_Rust";

#[cfg(windows)]
mod imp {
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;

    use super::TASK_NAME;

    /// Don't flash a console window when shelling out to schtasks from a GUI app.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    /// The binary the logon task should start — always a `clx.exe`.
    ///
    /// Resolves correctly from either prefs UI: the Slint window lives beside
    /// `clx.exe`, and for the Tauri prefs (hosted inside clx itself) the running
    /// exe already *is* clx.exe. That also keeps the task pointed at the stable
    /// install path `self_update` copies fresh builds into, rather than at a
    /// `target/release` path a `cargo clean` could delete.
    ///
    /// Deliberately errors instead of falling back to `current_exe()`: an
    /// earlier version did that and, when run from a directory with no sibling
    /// clx.exe, happily registered the *calling* binary as the logon program.
    /// Refusing is the safe failure — the caller surfaces it and the checkbox
    /// snaps back, rather than silently pointing autostart at the wrong file.
    pub fn target_exe() -> Result<PathBuf, String> {
        let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
        if exe
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case("clx.exe"))
        {
            return Ok(exe);
        }
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("clx.exe");
            if sibling.exists() {
                return Ok(sibling);
            }
        }
        Err(format!(
            "clx.exe not found next to {} — refusing to register a different binary",
            exe.display()
        ))
    }

    fn schtasks(args: &[&str]) -> bool {
        Command::new("schtasks.exe")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub fn is_enabled() -> bool {
        schtasks(&["/query", "/tn", TASK_NAME])
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Task Scheduler 1.2 definition: logon trigger for the current user, run
    /// elevated, no execution time limit, and — the reason we use XML at all —
    /// an explicit `<WorkingDirectory>`.
    fn task_xml(exe: &std::path::Path) -> String {
        let user = match (std::env::var("USERDOMAIN"), std::env::var("USERNAME")) {
            (Ok(d), Ok(u)) if !d.is_empty() => format!("{d}\\{u}"),
            (_, Ok(u)) => u,
            _ => String::new(),
        };
        let workdir = exe
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!(
            r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>CapsLockX (Rust) - start at logon, elevated</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{cmd}</Command>
      <WorkingDirectory>{wd}</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"#,
            user = xml_escape(&user),
            cmd = xml_escape(&exe.to_string_lossy()),
            wd = xml_escape(&workdir),
        )
    }

    /// schtasks /xml wants a real file; Task Scheduler exports are UTF-16LE with
    /// a BOM, so write it that way rather than UTF-8.
    fn write_task_xml(exe: &std::path::Path) -> Result<PathBuf, String> {
        let path = std::env::temp_dir().join(format!("clx-autostart-{}.xml", std::process::id()));
        let mut bytes = vec![0xFF, 0xFE];
        for unit in task_xml(exe).encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        std::fs::write(&path, bytes).map_err(|e| format!("writing task xml: {e}"))?;
        Ok(path)
    }

    pub fn enable() -> Result<(), String> {
        let exe = target_exe()?;
        let xml = write_task_xml(&exe)?;
        let xml_str = xml.to_string_lossy().into_owned();
        let args = ["/create", "/tn", TASK_NAME, "/xml", &xml_str, "/f"];

        // Fast path: clx is normally already elevated (it is started by this very
        // task at HighestAvailable), so no UAC prompt is needed at all.
        let ok = schtasks(&args) || {
            // Registering an elevated task needs admin. Re-run schtasks elevated;
            // this is the one moment the user sees a UAC prompt.
            run_elevated("schtasks.exe", &args)?;
            is_enabled()
        };
        let _ = std::fs::remove_file(&xml);
        if ok {
            Ok(())
        } else {
            Err("the logon task was not created (elevation declined?)".into())
        }
    }

    pub fn disable() -> Result<(), String> {
        let args = ["/delete", "/tn", TASK_NAME, "/f"];
        if schtasks(&args) {
            return Ok(());
        }
        run_elevated("schtasks.exe", &args)?;
        if is_enabled() {
            Err("the logon task could not be removed (elevation declined?)".into())
        } else {
            Ok(())
        }
    }

    /// Quote one argument for a command line we hand to ShellExecute, which takes
    /// a single pre-joined parameter string rather than an argv array.
    fn quote(arg: &str) -> String {
        if arg.is_empty() || arg.contains([' ', '\t', '"']) {
            format!("\"{}\"", arg.replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }

    /// Run `exe` elevated and wait for it to finish. Returns Err if the user
    /// declines the UAC prompt.
    fn run_elevated(exe: &str, args: &[&str]) -> Result<(), String> {
        use windows::core::{HSTRING, PCWSTR};
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};
        use windows::Win32::UI::Shell::{
            ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
        };
        use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

        let params = args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" ");
        let verb = HSTRING::from("runas");
        let file = HSTRING::from(exe);
        let par = HSTRING::from(params.as_str());

        unsafe {
            let mut sei = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: SEE_MASK_NOCLOSEPROCESS,
                lpVerb: PCWSTR(verb.as_ptr()),
                lpFile: PCWSTR(file.as_ptr()),
                lpParameters: PCWSTR(par.as_ptr()),
                nShow: SW_HIDE.0,
                ..Default::default()
            };
            // Err here is normally ERROR_CANCELLED — the user dismissed UAC.
            ShellExecuteExW(&mut sei).map_err(|e| format!("elevation failed: {e}"))?;
            if !sei.hProcess.is_invalid() {
                WaitForSingleObject(sei.hProcess, INFINITE);
                let _ = CloseHandle(sei.hProcess);
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::PathBuf;

    // macOS has its own richer implementation (a per-user LaunchAgent, see
    // adapters/macos/src/launch_at_login.rs); these stubs exist only so the
    // cross-platform Slint prefs window still compiles off Windows.
    pub fn target_exe() -> Result<PathBuf, String> {
        Err("launch at login is Windows-only in this crate".into())
    }
    pub fn is_enabled() -> bool {
        false
    }
    pub fn enable() -> Result<(), String> {
        Err("launch at login is Windows-only in this crate".into())
    }
    pub fn disable() -> Result<(), String> {
        Err("launch at login is Windows-only in this crate".into())
    }
}

pub use imp::{disable, enable, is_enabled, target_exe};

/// Apply `on`, then report what the system *actually* ends up as.
///
/// Callers should drive their checkbox from this return value rather than from
/// the requested value: if the user declines the UAC prompt the setting must
/// visibly snap back instead of lying about being enabled.
pub fn set_enabled(on: bool) -> bool {
    let _ = if on { enable() } else { disable() };
    is_enabled()
}
