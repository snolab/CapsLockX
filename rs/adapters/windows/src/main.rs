#![windows_subsystem = "windows"]

//! CapsLockX – Windows adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-windows
//! Build (release): cargo build -p capslockx-windows --release

mod commands;
mod config_store;
mod cursor_visibility;
mod hook;
mod output;
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
use tauri::{AppHandle, Manager as _};

// ── Embedded tray icons ─────────────────────────────────────────────────────

static ICON_OFF: &[u8] = include_bytes!("../../../../Data/XIconWhite.ico");
static ICON_ON:  &[u8] = include_bytes!("../../../../Data/XIconBlue.ico");

// ── Global AppHandle so hook.rs can update the tray icon ────────────────────

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Set when main() is about to return so background helper threads
/// (e.g. the quit-watch thread) can exit their poll loops cleanly instead
/// of blocking forever on kernel waits.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

const TRAY_ID: &str = "main";

/// Switch tray icon between on (blue) and off (white).
pub fn update_tray_icon(active: bool) {
    let Some(app) = APP_HANDLE.get() else { return };
    let id = TrayIconId::new(TRAY_ID);
    let Some(tray) = app.tray_by_id(&id) else { return };
    let bytes = if active { ICON_ON } else { ICON_OFF };
    if let Ok(icon) = Image::from_bytes(bytes) {
        let _ = tray.set_icon(Some(icon));
    }
}

/// Open (or focus) the preferences webview window. Same code path as the
/// tray's "Preferences…" menu — used for the Space+, hotkey from
/// `WinPlatform::open_preferences()`.
pub fn open_prefs_window() {
    let Some(app) = APP_HANDLE.get() else { return };
    let app = app.clone();
    // Tauri's webview construction must run off the hook thread; spawn.
    std::thread::spawn(move || {
        if let Some(w) = app.get_webview_window("prefs") {
            let _ = w.show();
            let _ = w.set_focus();
            return;
        }
        use tauri::webview::WebviewWindowBuilder;
        use tauri::WebviewUrl;
        let _ = WebviewWindowBuilder::new(
            &app,
            "prefs",
            WebviewUrl::App("index.html".into()),
        )
        .title("CapsLockX Preferences")
        .inner_size(560.0, 640.0)
        .resizable(false)
        .center()
        .build();
    });
}

fn main() {
    // Log panics to file since we're a windows subsystem app with no console.
    std::panic::set_hook(Box::new(|info| {
        hook::debug_log(&format!("[PANIC] {}", info));
    }));
    hook::debug_log("[main] started");
    // ── CLI subcommands (delegate to external tools, no GUI) ───────────
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
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

    // ── Ensure only one instance runs at a time. ───────────────────────
    // If we hit a previous instance we can't terminate (it's elevated and
    // we're not), re-launch self elevated and let the elevated child retry.
    // This mirrors the AHK version's behavior of UAC-prompting on demand.
    let needs_elevation_for_kill = shm::SharedState::kill_previous();

    // ── Elevate to admin if configured, or if a stuck old instance demands it ─
    let cfg_pre = config_store::load();
    if (cfg_pre.request_admin || needs_elevation_for_kill) && !is_elevated() {
        if needs_elevation_for_kill {
            eprintln!("[CLX] previous elevated instance detected — requesting elevation to kill it");
        } else {
            eprintln!("[CLX] requesting elevation …");
        }
        relaunch_elevated();
        return;
    }

    let cfg = config_store::load();
    hook::init_engine(cfg.clone().into_clx_config());

    // Create shared memory for IPC with AHK before installing the hook.
    if let Some(shm) = shm::SharedState::create() {
        eprintln!("[CLX] shared memory IPC created");
        hook::init_shared_state(shm);
    } else {
        eprintln!("[CLX] shared memory creation failed (standalone mode)");
    }

    vd_api::init();

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
        unsafe { let _ = windows::Win32::Foundation::CloseHandle(ev); }
    }
    // Spawn the tray/cursor worker BEFORE install_hook so the first hook
    // callback already has a channel to send edge events into. The worker
    // absorbs all Tauri and SystemParametersInfoW work that used to run
    // inside the WH_KEYBOARD_LL callback — keeping the hook callback fast
    // and avoiding deadlocks that could leave the process unkillable.
    hook::init_tray_worker();

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
            if let Some(evt) = shm::SharedState::create_quit_event() {
                let app_handle = app.handle().clone();
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
                                    app_handle.exit(0);
                                    return;
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

            let prefs_item  = MenuItemBuilder::with_id("prefs",      "Preferences…").build(app)?;
            let config_item = MenuItemBuilder::with_id("config_dir", "Open Config Folder…").build(app)?;
            let quit_item   = MenuItemBuilder::with_id("quit",       "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&prefs_item, &config_item, &quit_item])
                .build()?;

            let icon = Image::from_bytes(ICON_OFF)
                .expect("embedded ICO must be valid");
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "prefs" => {
                        // Re-show if already open.
                        if let Some(w) = app.get_webview_window("prefs") {
                            let _ = w.show();
                            let _ = w.set_focus();
                            return;
                        }
                        // Create on demand (must spawn thread to avoid deadlock on Windows).
                        let app = app.clone();
                        std::thread::spawn(move || {
                            use tauri::webview::WebviewWindowBuilder;
                            use tauri::WebviewUrl;
                            let _ = WebviewWindowBuilder::new(
                                &app,
                                "prefs",
                                WebviewUrl::App("index.html".into()),
                            )
                            .title("CapsLockX Preferences")
                            .inner_size(560.0, 640.0)
                            .resizable(false)
                            .center()
                            .build();
                        });
                    }
                    "config_dir" => {
                        if let Some(dir) = config_store::config_path().parent() {
                            let _ = std::fs::create_dir_all(dir);
                            let _ = Command::new("explorer").arg(dir).spawn();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
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

    // Signal helper threads to wind down before we tear down Win32 state.
    SHUTDOWN.store(true, Ordering::Relaxed);

    hook::uninstall_hook();
    cursor_visibility::disable();

    // Terminate AHK child process on shutdown.
    if let Some(ref mut child) = ahk_child {
        eprintln!("[CLX] terminating AHK child");
        let _ = child.kill();
        let _ = child.wait();
    }
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
        eprintln!("[CLX] GenerateModuleRunner.ahk or ModuleLoader.ahk not found, AHK modules disabled");
        return None;
    }

    // Step 1: generate module runner/functions files (blocking).
    eprintln!("[CLX] running GenerateModuleRunner.ahk …");
    match Command::new(exe).arg(r"Core\GenerateModuleRunner.ahk").status() {
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

// ── Elevation helpers ────────────────────────────────────────────────────────

fn is_elevated() -> bool {
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
    let exe_w: Vec<u16> = exe.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    let args: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let args_w: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();
    let dir = std::env::current_dir().unwrap_or_default();
    let dir_w: Vec<u16> = dir.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();

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
