#![windows_subsystem = "windows"]

//! CapsLockX – Windows adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-windows
//! Build (release): cargo build -p capslockx-windows --release

mod alt_tab;
mod commands;
mod config_store;
mod cursor_visibility;
mod hook;
mod output;
mod overlay;
mod prefs_window;
mod prompt_window;
mod raw_input;
mod release_state;
mod self_update;
mod shm;
mod vd_api;
mod vk;

use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use tauri::image::Image;
use tauri::tray::TrayIconId;
use tauri::AppHandle;

// ── Embedded tray icons ─────────────────────────────────────────────────────

static ICON_OFF: &[u8] = include_bytes!("../../../../Data/XIconWhite.ico");
static ICON_ON: &[u8] = include_bytes!("../../../../Data/XIconBlue.ico");

// ── Global AppHandle so hook.rs can update the tray icon ────────────────────

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Set when main() is about to return so background helper threads
/// (e.g. the quit-watch thread) can exit their poll loops cleanly instead
/// of blocking forever on kernel waits.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

const TRAY_ID: &str = "main";

/// Switch tray icon between on (blue) and off (white).
///
/// Marshalled onto the UI thread. `tray-icon`'s Windows `set_icon` ends with
/// a synchronous `SendMessageW` to the tray's hidden window, which lives on
/// the event-loop thread -- the same thread that owns the WH_KEYBOARD_LL
/// hook. Calling it from the tray worker therefore parked the worker in a
/// cross-thread wait *into the hook thread* on every mode edge. That is the
/// shape of the 3-way GUI-lock deadlock `hook.rs` warns about, and the prime
/// suspect for the unkillable 2-thread wedge (tmp/2026-09-11-clx-wedge-
/// incident.md). `run_on_main_thread` only posts to the event loop, so the
/// worker never blocks on the UI thread, and `set_icon` runs on the thread
/// that owns the icon, which is what the crate expects anyway.
pub fn update_tray_icon(active: bool) {
    let Some(app) = APP_HANDLE.get() else { return };
    let _ = app.run_on_main_thread(move || update_tray_icon_on_ui_thread(active));
}

/// The actual icon swap. Must run on the event-loop thread -- see above.
fn update_tray_icon_on_ui_thread(active: bool) {
    let Some(app) = APP_HANDLE.get() else { return };
    let id = TrayIconId::new(TRAY_ID);
    let Some(tray) = app.tray_by_id(&id) else {
        return;
    };
    let bytes = if active { ICON_ON } else { ICON_OFF };
    if let Ok(icon) = Image::from_bytes(bytes) {
        let _ = tray.set_icon(Some(icon));
    }
}

/// Open the preferences window — always as a SEPARATE process.
///
/// Prefers the native Slint prefs (`clx-prefs-slint.exe`, sitting next to
/// clx.exe): it starts in ~100ms instead of the WebView2 window's ~1-2s cold
/// start. Falls back to the WebView2/Tauri prefs (`clx prefs-window`) when the
/// native binary isn't present.
///
/// Either way it MUST be a separate process: an in-process WebView2 window makes
/// Windows stop delivering WH_KEYBOARD_LL while focused, killing all hotkeys.
///
/// Used by the tray "Preferences…" menu, which focuses an already-open window.
/// The Space+, hotkey uses [`toggle_prefs_window`] instead.
pub fn open_prefs_window() {
    let mut slot = PREFS_CHILD.lock().unwrap_or_else(|e| e.into_inner());
    if prefs_alive(&mut slot) || prefs_window_exists() {
        focus_prefs_window();
        return;
    }
    *slot = spawn_prefs_window();
}

/// Space+,: a toggle, not an "open another one" key. Closes the prefs window if
/// it's already up, otherwise opens it.
pub fn toggle_prefs_window() {
    let mut slot = PREFS_CHILD.lock().unwrap_or_else(|e| e.into_inner());
    // Close if we spawned it, OR if a prefs window is up that we did NOT spawn.
    // The second check matters: a window survives a clx restart or self-update
    // (and can be launched directly), leaving our tracked slot empty — without
    // it, the next press cheerfully opens a second window beside the first,
    // which is precisely the pile-up this toggle exists to prevent.
    if prefs_alive(&mut slot) || prefs_window_exists() {
        close_prefs_window(&mut slot);
        return;
    }
    *slot = spawn_prefs_window();
}

/// The prefs subprocess we spawned, while it's still running.
///
/// Tracking the child — instead of only looking for its window — is what makes
/// the toggle reliable during the ~100ms (Slint) to ~2s (WebView2) before the
/// window actually exists: a quick second Space+, would otherwise find no window
/// yet and spawn a duplicate, which is exactly how prefs windows piled up.
static PREFS_CHILD: std::sync::Mutex<Option<Child>> = std::sync::Mutex::new(None);

/// True if our prefs child is still running. Clears the slot when it has exited
/// (e.g. the user closed the window), so the next press opens a fresh one.
fn prefs_alive(slot: &mut Option<Child>) -> bool {
    match slot {
        Some(child) => match child.try_wait() {
            Ok(None) => true,
            _ => {
                *slot = None;
                false
            }
        },
        None => false,
    }
}

fn spawn_prefs_window() -> Option<Child> {
    let exe = std::env::current_exe().ok()?;
    // Pass our PID so the prefs subprocess can self-terminate if we (the parent)
    // exit/crash/self-update while it's open — otherwise it orphans and lingers.
    let parent_pid = std::process::id().to_string();
    // Re-execute ourselves. The window has to be a separate *process* — a UI
    // toolkit inside the hook process kills the keyboard hook while it has
    // focus — but it does not have to be a separate *file*, and a portable clx
    // is one executable on a USB stick.
    Command::new(exe)
        .arg("prefs-window")
        .env("CLX_PARENT_PID", &parent_pid)
        .spawn()
        .ok()
}

/// Both prefs UIs — native Slint and the Tauri/WebView2 fallback — use this
/// exact window title, so one lookup covers either.
fn find_prefs_window() -> Option<windows::Win32::Foundation::HWND> {
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
    unsafe {
        match FindWindowW(PCWSTR::null(), w!("CapsLockX Preferences")) {
            Ok(hwnd) if !hwnd.0.is_null() => Some(hwnd),
            _ => None,
        }
    }
}

fn prefs_window_exists() -> bool {
    find_prefs_window().is_some()
}

/// Ask the prefs window to close. Returns false if no window was found — which
/// means the spawn is still in flight and the caller should kill the process.
fn post_close_to_prefs_window() -> bool {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
    let Some(hwnd) = find_prefs_window() else {
        return false;
    };
    unsafe { PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)).is_ok() }
}

fn close_prefs_window(slot: &mut Option<Child>) {
    let closed = post_close_to_prefs_window();
    if let Some(mut child) = slot.take() {
        if !closed {
            // Window isn't up yet — kill the starting process instead.
            let _ = child.kill();
        }
        // Deliberately no wait(): this can run on the WH_KEYBOARD_LL hook
        // thread, and blocking there risks Windows silently dropping our hook.
        // Dropping a Child on Windows just closes the handle — no zombie.
    }
}

fn focus_prefs_window() {
    use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_RESTORE};
    let Some(hwnd) = find_prefs_window() else {
        return;
    };
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Footer text shown in the native prefs window: "CapsLockX v<ver> · built <ts>".
fn prefs_version_line() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    let built = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::metadata(&p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            // seconds -> "YYYY-MM-DD" (UTC), civil-from-days (Howard Hinnant).
            let days = (d.as_secs() / 86400) as i64;
            let z = days + 719468;
            let era = if z >= 0 { z } else { z - 146096 } / 146097;
            let doe = z - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let dd = doy - (153 * mp + 2) / 5 + 1;
            let mm = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if mm <= 2 { y + 1 } else { y };
            format!("{:04}-{:02}-{:02}", y, mm, dd)
        })
        .unwrap_or_default();
    if built.is_empty() {
        format!("CapsLockX v{ver}")
    } else {
        format!("CapsLockX v{ver} · built {built}")
    }
}

fn main() {
    // Log panics to file since we're a windows subsystem app with no console.
    // Panics use the synchronous logger so they're recorded even if the async
    // writer isn't running (CLX_DEBUG disabled, or panic before init). The
    // record also goes to the durable `%LOCALAPPDATA%\CapsLockX\crash.log` so a
    // post-mortem days later still has the message + location even after TEMP is
    // wiped. Thread name is included because panics that unwind across an
    // `extern "system"` callback (WndProc / the LL keyboard hook / a COM vtable
    // up-call) abort the whole process, and the thread tells us which boundary.
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let tname = thread.name().unwrap_or("<unnamed>");
        let ver = self_update::version_string();
        hook::crash_log_sync(&format!("[PANIC] {ver} thread '{tname}' {info}"));
    }));
    hook::init_debug_log();
    hook::debug_log("[main] started");
    // ── CLI subcommands (delegate to external tools, no GUI) ───────────
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            // Print version + build commit and exit. stdout is inherited from
            // the npm launcher (node) / parent shell, so this reaches the
            // terminal when run as `clx --version`.
            "--version" | "-V" | "version" => {
                self_update::print_version();
                return;
            }
            // Out-of-process preferences window. MUST be handled here, before
            // kill_previous / elevation / hook install — this subprocess shares
            // the clx.exe name but must NOT touch the running main instance or
            // install a keyboard hook. It only shows the prefs UI.
            "prefs-window" => {
                // Native Slint, linked in. Falls back to the WebView2 window
                // only if it cannot start, so a broken UI never costs you
                // access to your settings.
                let args = vec![prefs_version_line()];
                if let Err(error) = clx_prefs_slint::run(&args) {
                    eprintln!("[CLX] native prefs window failed ({error}); using WebView2");
                    prefs_window::run();
                }
                return;
            }
            // Out-of-process brainstorm prompt window. Like prefs-window, this
            // must NOT install a hook or touch the running instance — it only
            // shows the input UI and writes the result to the temp file given as
            // the final argument.  Usage: clx prompt-window <title> <msg> <prefill> <out_path>
            "prompt-window" => {
                let title = std::env::args().nth(2).unwrap_or_default();
                let message = std::env::args().nth(3).unwrap_or_default();
                let prefill = std::env::args().nth(4).unwrap_or_default();
                let out_path = std::env::args().nth(5).unwrap_or_default();
                let args = vec![
                    title.clone(),
                    message.clone(),
                    prefill.clone(),
                    out_path.clone(),
                ];
                if let Err(error) = clx_prompt_slint::run(&args) {
                    eprintln!("[CLX] native prompt window failed ({error}); using WebView2");
                    prompt_window::run(title, message, prefill, out_path);
                }
                return;
            }
            // Out-of-process brainstorm streaming overlay. Reads the shared text
            // region and renders it; never installs a hook.
            "overlay-window" => {
                overlay::run();
                return;
            }
            // Read-only diagnostic: acquire the virtual-desktop COM interface on
            // this (main, hook-less) thread and dump version / count / current
            // index / GUIDs (exercises GetCurrentDesktop + GetDesktops without
            // switching). Result goes to %TEMP%\capslockx_vd.log.
            // Stop a running instance cleanly, so it uninstalls its keyboard
            // hook on the way out. Scripts and build steps must use this rather
            // than taskkill — see `SharedState::request_quit` for what killing it
            // from outside costs.
            "quit" => {
                if shm::SharedState::request_quit() {
                    println!("clx: asked the running instance to quit");
                } else {
                    println!("clx: no running instance");
                }
                return;
            }
            // Recovery for "my space bar stopped working": free the keyboard from
            // a clx that was killed from outside and will not finish dying. See
            // `SharedState::unstick_others`. No reboot required.
            "unstick" => {
                let (suspended, unreachable) = shm::SharedState::unstick_others();
                if suspended == 0 && unreachable == 0 {
                    println!("clx unstick: nothing stuck");
                } else {
                    println!("clx unstick: {suspended} freed, {unreachable} unreachable");
                    if unreachable > 0 {
                        println!("clx unstick: re-run from an administrator shell for the rest");
                    }
                }
                return;
            }
            "vd-test" => {
                vd_api::log_line("[vd_api] vd-test");
                vd_api::dump();
                return;
            }
            // Run a plugin: spawn it and perform whatever effects it writes
            // to stdout. This is the entire plugin contract — any program that
            // can print a line can extend CLX, and CLX learns nothing about
            // what the program is for. See lab/plugins.
            //   clx plugin <command> [args...]
            "plugin" => {
                let argv: Vec<String> = std::env::args().skip(2).collect();
                let Some((command, rest)) = argv.split_first() else {
                    eprintln!("usage: clx plugin <command> [args...]");
                    hard_exit(2);
                };
                let platform = output::WinPlatform::new();
                match capslockx_core::plugin::run(command, rest, &platform) {
                    capslockx_core::plugin::Outcome::Exited(code) => hard_exit(code.max(0) as u32),
                    capslockx_core::plugin::Outcome::NotStarted(why) => {
                        eprintln!("clx plugin: {why}");
                        hard_exit(127);
                    }
                }
            }
            // What CLX+Z would actually cycle through, and what it hides.
            //
            // The cycling-flood bugs all came down to landing on a window that
            // then swallowed the keyboard, and each round of diagnosis had to
            // guess at the window list from the outside. This prints it, without
            // touching focus.
            "windows" => {
                output::dump_window_list();
                return;
            }
            // Dev smoke test for the overlay pipeline (no LLM needed).
            "overlay-selftest" => {
                overlay::selftest();
                return;
            }
            "read-screen-text" => {
                let tool = Path::new(r".\rs\target\release\clx-screen-reader.exe");
                let tool_alt = Path::new(r".\clx-screen-reader.exe");
                let exe = if tool.exists() {
                    tool
                } else if tool_alt.exists() {
                    tool_alt
                } else {
                    eprintln!("error: clx-screen-reader.exe not found");
                    std::process::exit(1);
                };
                let status = Command::new(exe).status().unwrap_or_else(|e| {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                });
                std::process::exit(status.code().unwrap_or(1));
            }
            _ => {}
        }
    }

    // Clear any `clx.exe.old-<pid>` sidecar left by a prior self-update swap.
    self_update::cleanup_old_binaries();

    // ── Ensure only one instance runs at a time. ───────────────────────
    // If we hit a previous instance we can't terminate (it's elevated and
    // we're not), re-launch self elevated and let the elevated child retry.
    // This mirrors the AHK version's behavior of UAC-prompting on demand.
    let needs_elevation_for_kill = shm::SharedState::kill_previous();
    hook::debug_log("[main] stage: previous instance handled");

    // ── Elevate to admin if configured, or if a stuck old instance demands it ─
    let cfg_pre = config_store::load();
    if (cfg_pre.request_admin || needs_elevation_for_kill) && !is_elevated() {
        if needs_elevation_for_kill {
            eprintln!(
                "[CLX] previous elevated instance detected — requesting elevation to kill it"
            );
        } else {
            eprintln!("[CLX] requesting elevation …");
        }
        relaunch_elevated();
        return;
    }

    let cfg = config_store::load();

    // Background: log version, auto-upgrade (git pull --ff-only) and rebuild if
    // the local checkout has moved ahead of this binary. No-op for released
    // (non-git) binaries. Runs off-thread so it never delays hotkey startup, and
    // relaunches fully detached so it can't couple clx to our launching session.
    self_update::spawn_check(cfg.auto_rebuild);
    // Stage markers. Everything between `[main] started` and `[main] hook
    // installed` is time during which clx is running but deaf, so each stage
    // that can block says when it finished. Debug-gated, so free in production.
    hook::debug_log("[main] stage: config loaded, self-update spawned");

    // DISABLED pending diagnosis — 2026-09-25.
    //
    // Routing injection through the dedicated thread silently dropped every
    // injected event: with it on, a Space *tap* was suppressed on both halves and
    // the replacement Space never arrived, so the space bar simply stopped
    // working. The hook log shows the suppression and no following `ours=true`
    // pair, where before the change there was one.
    //
    // Without this call `send` falls back to injecting inline, which is the
    // long-standing behaviour — including its hazard of injecting from the hook
    // callback. A freeze after nineteen hours is a far better failure than a
    // keyboard with no space bar, so the hazard stays until the queue path is
    // understood rather than guessed at.
    //
    // output::start_injector();

    hook::init_engine(cfg.clone().into_clx_config());
    hook::debug_log("[main] stage: engine ready");

    // Create shared memory for IPC with AHK before installing the hook.
    if let Some(shm) = shm::SharedState::create() {
        eprintln!("[CLX] shared memory IPC created");
        hook::init_shared_state(shm);
    } else {
        eprintln!("[CLX] shared memory creation failed (standalone mode)");
    }

    hook::debug_log("[main] stage: shared memory done");

    vd_api::init();
    hook::debug_log("[main] stage: vd_api ready");

    // Spawn AHK modules only when --with-ahk is passed.
    // WH_KEYBOARD_LL hooks are called most-recent-first (LIFO), so installing
    // our hook AFTER AHK ensures Rust gets first crack at every key.
    // We use a named Win32 event so AHK signals us when it's truly ready
    // instead of sleeping for an arbitrary duration.
    let with_ahk = std::env::args().any(|a| a == "--with-ahk");
    let ahk_ready_event = if with_ahk {
        unsafe {
            use windows::core::w;
            use windows::Win32::System::Threading::CreateEventW;
            CreateEventW(None, true, false, w!("CapsLockX_AhkReady")).ok()
        }
    } else {
        None
    };
    let mut ahk_child = if with_ahk { spawn_ahk() } else { None };
    // Store AHK PID in shared memory so the next instance can kill it.
    if let Some(ref child) = ahk_child {
        if let Some(shm) = hook::get_shared_state() {
            shm.write_ahk_pid(child.id());
        }
    }
    if ahk_child.is_some() {
        if let Some(ev) = &ahk_ready_event {
            unsafe {
                use windows::Win32::System::Threading::WaitForSingleObject;
                WaitForSingleObject(*ev, 10_000); // wait up to 10s
            }
        }
    }
    if let Some(ev) = ahk_ready_event {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(ev);
        }
    }
    // Spawn the tray/cursor worker BEFORE install_hook so the first hook
    // callback already has a channel to send edge events into. The worker
    // absorbs all Tauri and SystemParametersInfoW work that used to run
    // inside the WH_KEYBOARD_LL callback — keeping the hook callback fast
    // and avoiding deadlocks that could leave the process unkillable.
    hook::init_tray_worker();
    hook::debug_log("[main] stage: tray worker spawned");

    // Install hook on the main thread BEFORE Tauri init.
    // Tauri's setup takes ~15s, during which the hook would be starved of
    // message pumping.  We install here and run a brief PeekMessage pump
    // in install_hook's SetTimer callback to keep it alive.
    hook::install_hook();
    hook::debug_log("[main] hook installed");
    let engine = hook::engine();

    tauri::Builder::default()
        .manage(engine)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            // Store the handle so hook.rs can update the tray icon.
            let _ = APP_HANDLE.set(app.handle().clone());

            // Watch the quit event so a new instance can ask us to exit cleanly.
            // Uses a 500 ms polling wait instead of INFINITE so the thread
            // observes the SHUTDOWN flag during normal shutdown and exits
            // cleanly, rather than sitting in a kernel wait forever.
            //
            // Leaving via `hard_exit` rather than `app_handle.exit(0)` is the
            // whole point. Tauri's exit runs a full teardown, which is exactly
            // what deadlocks (see hard_exit), so the old instance could not
            // finish within the 1500 ms `kill_previous` waits — and was then
            // TerminateProcess'd **with its keyboard hook still installed**.
            // Nothing can remove another process's hook, and the corpse's threads
            // sit in win32k so it never finishes dying, so that orphaned hook
            // lives on: still suppressing Space as a CLX trigger, no longer
            // injecting the replacement. The user loses their space bar and only
            // a reboot brings it back.
            //
            // That happened repeatedly, and it is self-inflicted: every
            // self-update relaunch replaces the instance, so each one was one
            // more orphaned hook. `hard_exit` uninstalls the hook first and then
            // leaves immediately, so there is nothing left to orphan.
            if let Some(evt) = shm::SharedState::create_quit_event() {
                let raw = evt.0 as usize; // extract raw ptr for Send
                let _ = std::thread::Builder::new()
                    .name("clx-quit-watch".into())
                    .spawn(move || {
                        use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
                        use windows::Win32::System::Threading::WaitForSingleObject;
                        unsafe {
                            let h = HANDLE(raw as *mut _);
                            loop {
                                let r = WaitForSingleObject(h, 500);
                                if r == WAIT_OBJECT_0 {
                                    let _ = CloseHandle(h);
                                    hook::debug_log("[main] quit requested — releasing the hook");
                                    hard_exit(0);
                                }
                                if SHUTDOWN.load(Ordering::Relaxed) {
                                    let _ = CloseHandle(h);
                                    return;
                                }
                                // WAIT_TIMEOUT or WAIT_FAILED — retry.
                            }
                        }
                    });
            }

            // Watch the config-changed event so the out-of-process prefs window
            // can push new settings to our live engine. The prefs subprocess
            // writes config.json then signals this event; we reload and apply.
            if let Some(evt) = shm::SharedState::create_config_changed_event() {
                let raw = evt.0 as usize;
                let _ = std::thread::Builder::new()
                    .name("clx-config-watch".into())
                    .spawn(move || {
                        use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
                        use windows::Win32::System::Threading::WaitForSingleObject;
                        unsafe {
                            let h = HANDLE(raw as *mut _);
                            loop {
                                let r = WaitForSingleObject(h, 500);
                                if r == WAIT_OBJECT_0 {
                                    let cfg = config_store::load();
                                    hook::engine().update_config(cfg.into_clx_config());
                                    hook::debug_log("[main] config reloaded from prefs subprocess");
                                }
                                if SHUTDOWN.load(Ordering::Relaxed) {
                                    let _ = CloseHandle(h);
                                    return;
                                }
                                // WAIT_TIMEOUT or WAIT_OBJECT_0 — loop again.
                            }
                        }
                    });
            }

            let prefs_item = MenuItemBuilder::with_id("prefs", "Preferences…").build(app)?;
            let config_item =
                MenuItemBuilder::with_id("config_dir", "Open Config Folder…").build(app)?;
            let restart_item =
                MenuItemBuilder::with_id("restart", "Restart CapsLockX").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&prefs_item, &config_item, &restart_item, &quit_item])
                .build()?;

            let icon = Image::from_bytes(ICON_OFF).expect("embedded ICO must be valid");
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .tooltip(self_update::version_string())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    // This closure fires from inside the framework's `extern
                    // "system"` WndProc; a panic here would unwind across that
                    // boundary and abort the process. Firewall it.
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        match event.id().as_ref() {
                            // Single code path: builds (with hide-on-close
                            // handler) or re-shows the prefs window.
                            "prefs" => open_prefs_window(),
                            "config_dir" => {
                                if let Some(dir) = config_store::config_path().parent() {
                                    let _ = std::fs::create_dir_all(dir);
                                    let _ = Command::new("explorer").arg(dir).spawn();
                                }
                            }
                            // Relaunch, then exit. Reuses the self-updater's
                            // spawn: the replacement must break away from any
                            // Job Object and inherit no console, or restarting
                            // from a terminal-launched clx takes the terminal
                            // down with it (see self_update's module docs).
                            "restart" => {
                                let exe = std::env::current_exe();
                                match exe {
                                    Ok(exe) => {
                                        let cwd =
                                            exe.parent().unwrap_or(Path::new(".")).to_path_buf();
                                        match self_update::spawn_detached(&exe, &cwd) {
                                            Ok(()) => app.exit(0),
                                            Err(e) => hook::crash_log_sync(&format!(
                                                "[tray] restart: spawn failed: {e}"
                                            )),
                                        }
                                    }
                                    Err(e) => hook::crash_log_sync(&format!(
                                        "[tray] restart: current_exe failed: {e}"
                                    )),
                                }
                            }
                            "quit" => app.exit(0),
                            _ => {}
                        }
                    }));
                    if r.is_err() {
                        hook::crash_log_sync("[PANIC] recovered in tray on_menu_event");
                    }
                })
                .build(app)?;
            // Tao registers keyboard raw input during runtime creation. Our
            // independent release receiver takes ownership only after that;
            // mouse registration and ordinary window messages stay with Tao.
            raw_input::start();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
        ])
        .build(tauri::generate_context!())
        .expect("tauri build error")
        .run(|_app, event| {
            // Keep running when the prefs window is closed; only exit via app.exit().
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });

    // The AHK child is the one thing the kernel will not clean up for us: it is
    // a separate process and would be orphaned. Kill it before we go.
    if let Some(ref mut child) = ahk_child {
        eprintln!("[CLX] terminating AHK child");
        let _ = child.kill();
        let _ = child.wait();
    }

    // Deliberately not a return from `main` — see `hard_exit`.
    hard_exit(0);
}

/// Spawn AHK module loader (lightweight, Rust-first path).
///
/// 1. Run `Core\capslockx-ahkv1.exe Core\GenerateModuleRunner.ahk` and wait for it to finish
///    (generates `Core\CapsLockX-ModulesRunner.ahk` and `Core\CapsLockX-ModulesFunctions.ahk`).
/// 2. Spawn `Core\capslockx-ahkv1.exe Core\ModuleLoader.ahk` (stays running for modules).
fn spawn_ahk() -> Option<Child> {
    let exe = Path::new(r".\Core\capslockx-ahkv1.exe");
    let generator = Path::new(r".\Core\GenerateModuleRunner.ahk");
    let loader = Path::new(r".\Core\ModuleLoader.ahk");

    if !exe.exists() {
        eprintln!("[CLX] Core\\ahk.exe not found, AHK modules disabled");
        return None;
    }
    if !generator.exists() || !loader.exists() {
        eprintln!(
            "[CLX] GenerateModuleRunner.ahk or ModuleLoader.ahk not found, AHK modules disabled"
        );
        return None;
    }

    // Step 1: generate module runner/functions files (blocking).
    eprintln!("[CLX] running GenerateModuleRunner.ahk …");
    match Command::new(exe)
        .arg(r"Core\GenerateModuleRunner.ahk")
        .status()
    {
        Ok(status) => {
            if !status.success() {
                eprintln!("[CLX] GenerateModuleRunner exited with {status}");
            }
        }
        Err(e) => {
            eprintln!("[CLX] failed to run GenerateModuleRunner: {e}");
            return None;
        }
    }

    // Step 2: spawn the lightweight module loader (non-blocking).
    match Command::new(exe).arg(r"Core\ModuleLoader.ahk").spawn() {
        Ok(child) => {
            eprintln!("[CLX] spawned ModuleLoader (pid={})", child.id());
            Some(child)
        }
        Err(e) => {
            eprintln!("[CLX] failed to spawn ModuleLoader: {e}");
            None
        }
    }
}

/// Leave the process without going through `ExitProcess`.
///
/// Returning from `main`, `std::process::exit` and `app.exit` all funnel into
/// the CRT's exit path, which calls `ExitProcess`. That is documented to
/// terminate every other thread *"without regard to whether they are still
/// using resources"*, and then to run `DLL_PROCESS_DETACH` while holding the
/// loader lock. clx has around sixteen detached threads at that moment — the
/// Tauri UI thread, the hook ticker, seven `AccModel` tickers, the raw-input
/// receiver in `GetMessageW`, the virtual-desktop COM worker, the tray worker,
/// the debug logger — and most of them are inside user32 or win32k calls
/// (`SendInput`, `EnumWindows`, `GetMessageW`). Kill one of those while it
/// holds a user-mode window-manager lock and the exiting thread deadlocks with
/// the loader lock in its hand.
///
/// That is not a theoretical concern here: it is the "wedge" this project has
/// been chasing for weeks. The result is a process stuck *inside* termination —
/// one or two threads left in `Wait`/`UserRequest`, `ExitCode` already set, and
/// `TerminateProcess` returning `ERROR_ACCESS_DENIED` because termination has
/// already begun and cannot be restarted. Nothing in user mode can reap it, and
/// crucially **Windows never releases its `WH_KEYBOARD_LL` registration**, so
/// after a few restarts a freshly launched clx comes up with a hook that
/// receives nothing and a tray menu that does not open. Every symptom traces
/// back to this one exit.
///
/// `TerminateProcess` on our own process takes neither path: it does not notify
/// DLLs and does not touch the loader lock. Nothing here needs an orderly
/// teardown — there is no buffered state to flush, and the kernel reclaims the
/// hook, the windows, the raw-input registration and the shared memory for us.
/// So do the two things that actually matter to the *outside* world first, then
/// leave immediately.
pub fn hard_exit(code: u32) -> ! {
    SHUTDOWN.store(true, Ordering::Relaxed);
    hook::uninstall_hook();
    cursor_visibility::disable();
    unsafe {
        use windows::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
        let _ = TerminateProcess(GetCurrentProcess(), code);
    }
    // TerminateProcess on self does not return, but the compiler wants a `!`.
    unreachable!("TerminateProcess(self) returned");
}

// ── Elevation helpers ────────────────────────────────────────────────────────

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        );
        let _ = windows::Win32::Foundation::CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

fn relaunch_elevated() {
    use windows::core::w;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().unwrap_or_default();
    let exe_w: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let args: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let args_w: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();
    let dir = std::env::current_dir().unwrap_or_default();
    let dir_w: Vec<u16> = dir
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use windows::core::PCWSTR;
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(exe_w.as_ptr()),
            PCWSTR(args_w.as_ptr()),
            PCWSTR(dir_w.as_ptr()),
            SW_SHOWNORMAL,
        );
    }
}
