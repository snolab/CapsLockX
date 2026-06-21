use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc;
/// Windows WH_KEYBOARD_LL hook – bridges Win32 key events to ClxEngine.
use std::sync::{Arc, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, SetTimer,
    SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
};

use crate::output::{WinPlatform, CLX_EXTRA_INFO};
use crate::shm::SharedState;
use crate::vk::vk_to_keycode;
use capslockx_core::{ClxConfig, ClxEngine, CoreResponse};

// ── Raw HHOOK stored as usize for atomic access ───────────────────────────────

static HOOK_RAW: AtomicUsize = AtomicUsize::new(0);

// ── Engine (initialised once via init_engine before hook installs) ────────────

static ENGINE: OnceLock<Arc<ClxEngine>> = OnceLock::new();

// ── Shared memory (set from main before hook install) ────────────────────────

static SHM: OnceLock<SharedState> = OnceLock::new();

/// Last tray-active state for edge detection (0 = off, 1 = on, u32::MAX = uninitialised).
static LAST_TRAY_ACTIVE: AtomicU32 = AtomicU32::new(u32::MAX);

/// Channel sender to the tray/cursor worker thread. `None` until `init_tray_worker`
/// is called. The hook callback must NEVER call Tauri or SystemParametersInfoW
/// directly — those can block on GUI locks held by threads waiting on the hook,
/// creating a 3-way deadlock that leaves the process unkillable.
static TRAY_TX: OnceLock<mpsc::Sender<bool>> = OnceLock::new();

/// Spawn the tray/cursor-visibility worker thread. Must be called once before
/// `install_hook()`. The worker coalesces bursts of edge events so a rapid
/// on/off/on flicker only triggers the most recent state.
pub fn init_tray_worker() {
    let (tx, rx) = mpsc::channel::<bool>();
    if TRAY_TX.set(tx).is_err() {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("clx-tray-worker".into())
        .spawn(move || {
            while let Ok(first) = rx.recv() {
                // Coalesce any backlog so we only apply the latest state.
                let mut latest = first;
                while let Ok(next) = rx.try_recv() {
                    latest = next;
                }
                crate::update_tray_icon(latest);
                if latest {
                    crate::cursor_visibility::enable();
                } else {
                    crate::cursor_visibility::disable();
                }
            }
        });
}

/// Store the shared memory handle so the hook callback can publish mode changes.
pub fn init_shared_state(shm: SharedState) {
    let _ = SHM.set(shm);
}

/// Get a reference to the shared memory state (if initialised).
pub fn get_shared_state() -> Option<&'static SharedState> {
    SHM.get()
}

// ── Win32 message constants ───────────────────────────────────────────────────

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const LLKHF_UP: u32 = 0x80;
const LLKHF_INJECTED: u32 = 0x10;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn init_engine(config: ClxConfig) {
    // Drive AccModel ticks from the main thread (via SetTimer) instead of
    // background threads.  This ensures SendInput runs on the hook thread,
    // avoiding phantom modifier key-up events from cross-thread injection.
    capslockx_core::acc_model::set_external_tick(true);
    let platform = Arc::new(WinPlatform::new());
    ENGINE.set(ClxEngine::with_config(platform, config)).ok();
}

pub fn engine() -> Arc<ClxEngine> {
    ENGINE
        .get()
        .expect("init_engine must be called before engine()")
        .clone()
}

/// Install the WH_KEYBOARD_LL hook on the **calling (main/UI) thread** and spawn
/// the AccModel ticker.
///
/// **IMPORTANT — must be installed on the thread that owns clx's own windows
/// (the Tauri/WebView2 UI thread).** A low-level keyboard hook installed on a
/// *different* thread than the one owning the foreground window will NOT receive
/// callbacks while one of clx's *own* windows (e.g. the Preferences window) has
/// focus — Windows cannot deliver the cross-thread hook callback through the
/// process's serialized input queue, so the hook silently goes dead until focus
/// leaves. Installing on the UI thread (same as AutoHotkey's GUI-thread hook)
/// makes the hook fire for clx's own windows too. Tauri's `run()` loop on this
/// thread pumps the messages that keep the hook serviced.
///
/// The original close-freeze (closing prefs starving the hook during WebView2
/// teardown) is solved separately by hiding the prefs window instead of
/// destroying it — see `open_prefs_window` in main.rs.
pub fn install_hook() {
    let hmod = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hhook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0)
            .expect("SetWindowsHookExW failed")
    };
    HOOK_RAW.store(hhook.0 as usize, Ordering::SeqCst);

    // Cursor-visibility nudge timer on the UI thread; Tauri's loop dispatches it.
    unsafe {
        SetTimer(None, 0, 250, Some(nudge_timer_proc));
    }

    // High-frequency ticker thread for AccModel physics (~166 FPS).
    // Uses timeBeginPeriod(1) for 1ms Sleep resolution, then sleeps ~6ms
    // per tick. SendInput for mouse movement works fine cross-thread.
    // A low-frequency SetTimer (250ms) handles cursor-visibility nudges
    // that don't need high frequency.
    std::thread::Builder::new()
        .name("clx-ticker".into())
        .spawn(|| {
            use std::sync::atomic::AtomicU64;
            static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
            static LAST_LOG: AtomicU64 = AtomicU64::new(0);

            // Request 1ms timer resolution from Windows.
            unsafe {
                windows::Win32::Media::timeBeginPeriod(1);
            }

            loop {
                std::thread::sleep(std::time::Duration::from_millis(6)); // ~166 FPS

                if let Some(engine) = ENGINE.get() {
                    engine.tick();
                }

                // FPS logging — every 2 seconds.
                let count = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let last = LAST_LOG.load(Ordering::Relaxed);
                if last == 0 {
                    LAST_LOG.store(now_ms, Ordering::Relaxed);
                    TICK_COUNT.store(0, Ordering::Relaxed);
                } else if now_ms - last >= 2000 {
                    let elapsed_s = (now_ms - last) as f64 / 1000.0;
                    let fps = count as f64 / elapsed_s;
                    debug_log(&format!(
                        "[CLX] tick: {:.1} FPS ({} ticks in {:.1}s)",
                        fps, count, elapsed_s
                    ));
                    LAST_LOG.store(now_ms, Ordering::Relaxed);
                    TICK_COUNT.store(0, Ordering::Relaxed);
                }
            }
        })
        .expect("failed to spawn ticker thread");
}

unsafe extern "system" fn nudge_timer_proc(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    let active = LAST_TRAY_ACTIVE.load(Ordering::Relaxed);
    if active != 0 && active != u32::MAX {
        crate::cursor_visibility::nudge();
    }
}

pub fn uninstall_hook() {
    let raw = HOOK_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(raw, std::ptr::null()) {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(raw));
        }
    }
}

// ── Hook callback ─────────────────────────────────────────────────────────────

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code < 0 {
        return call_next(n_code, w_param, l_param);
    }

    let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

    let flags = kb.flags.0;

    let msg = w_param.0 as u32;
    let is_up = (flags & LLKHF_UP) != 0;
    let pressed = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && !is_up;
    let released = matches!(msg, WM_KEYUP | WM_SYSKEYUP) && is_up;

    let code = vk_to_keycode(kb.vkCode);
    let injected = (flags & LLKHF_INJECTED) != 0;
    let is_ours = injected && kb.dwExtraInfo == CLX_EXTRA_INFO;

    // Debug: log ALL key events with the currently focused window so we can tell
    // whether the hook fires while a given window (e.g. the prefs WebView2) has
    // focus. Gated by debug_enabled() so the GetForegroundWindow/class lookup
    // never runs on the hot path in production.
    if debug_enabled() && (pressed || released) {
        debug_log(&format!(
            "[hook] vk=0x{:02X} {:?} {} inj={} ours={} fg=[{}]",
            kb.vkCode,
            code,
            if pressed { "DN" } else { "UP" },
            injected,
            is_ours,
            fg_window_desc()
        ));
    }

    // Skip events injected by us
    if is_ours {
        return call_next(n_code, w_param, l_param);
    }

    if !pressed && !released {
        return call_next(n_code, w_param, l_param);
    }

    let engine = ENGINE.get().expect("init_engine not called");
    let resp = engine.on_key_event(code, pressed);
    if debug_enabled() {
        debug_log(&format!(
            "[hook] -> {:?} mode={}",
            resp,
            engine.state().mode()
        ));
    }

    // Publish current mode to shared memory so AHK extensions can read it.
    let mode = engine.state().mode();
    if let Some(shm) = SHM.get() {
        shm.write_mode(mode);
    }

    // Dispatch mode-edge transitions to the tray worker. The hook callback
    // must never call Tauri or SystemParametersInfoW directly — sending on an
    // mpsc channel is wait-free (one atomic CAS + a malloc) and safe here.
    // Cursor-visibility nudges are handled by the SetTimer tick instead.
    let active = u32::from(mode != 0);
    let prev = LAST_TRAY_ACTIVE.swap(active, Ordering::Relaxed);
    if prev != active {
        if let Some(tx) = TRAY_TX.get() {
            let _ = tx.send(active != 0);
        }
    }

    match resp {
        CoreResponse::Suppress => LRESULT(1),
        CoreResponse::PassThrough => call_next(n_code, w_param, l_param),
    }
}

/// Channel to the background log writer. `Some(_)` only when CLX_DEBUG is set.
/// `None` → logging disabled → `debug_log` is a no-op (no disk I/O, no alloc).
static DEBUG_TX: OnceLock<Option<mpsc::Sender<String>>> = OnceLock::new();

fn log_path() -> Option<std::path::PathBuf> {
    std::env::var("TEMP")
        .ok()
        .map(|tmp| std::path::PathBuf::from(format!(r"{}\capslockx_hook.log", tmp)))
}

/// Initialise debug logging. Call once early in `main`.
///
/// Logging is **off by default** and only enabled when `CLX_DEBUG` is set (and
/// not empty/"0"). This matters because `debug_log` is called several times per
/// keystroke from inside the WH_KEYBOARD_LL callback — the old version did a
/// synchronous file open+append+write there, stalling the system input path and
/// bloating a multi-MB log. When enabled, writes are handed to a background
/// thread so the hook callback never touches the disk.
pub fn init_debug_log() {
    let enabled = std::env::var("CLX_DEBUG")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if !enabled {
        let _ = DEBUG_TX.set(None);
        return;
    }
    let (tx, rx) = mpsc::channel::<String>();
    let _ = std::thread::Builder::new()
        .name("clx-debug-log".into())
        .spawn(move || {
            use std::io::Write as _;
            let Some(path) = log_path() else { return };
            let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            else {
                return;
            };
            while let Ok(line) = rx.recv() {
                let _ = writeln!(file, "{} {}", now_ms(), line);
                // Drain any backlog before flushing so bursts batch into one flush.
                while let Ok(next) = rx.try_recv() {
                    let _ = writeln!(file, "{} {}", now_ms(), next);
                }
                let _ = file.flush();
            }
        });
    let _ = DEBUG_TX.set(Some(tx));
}

/// Milliseconds since the Unix epoch — used to timestamp log lines so a
/// separate focus poller can be time-correlated with hook activity.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Cheap check used to gate expensive per-key logging (string formatting +
/// GetForegroundWindow) so it only runs when CLX_DEBUG is active.
#[inline]
fn debug_enabled() -> bool {
    matches!(DEBUG_TX.get(), Some(Some(_)))
}

/// Describe the currently focused (foreground) window: class name + PID. Uses
/// only non-blocking calls — notably NOT GetWindowTextW, which sends WM_GETTEXT
/// and could block the hook thread if the target window is busy.
fn fg_window_desc() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 128];
        let n = GetClassNameW(hwnd, &mut buf);
        let class = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        format!("hwnd=0x{:X} class='{}' pid={}", hwnd.0 as usize, class, pid)
    }
}

/// Hot-path log: a cheap no-op unless CLX_DEBUG enabled the background writer.
/// Safe to call from the keyboard-hook callback — never blocks on disk.
#[inline]
pub fn debug_log(msg: &str) {
    if let Some(Some(tx)) = DEBUG_TX.get() {
        let _ = tx.send(msg.to_string());
    }
}

/// Synchronous, always-on log for rare critical events (panics) where we want
/// the record on disk even if the async writer isn't up. Not for the hot path.
pub fn debug_log_sync(msg: &str) {
    use std::io::Write as _;
    if let Some(path) = log_path() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

#[inline(always)]
unsafe fn call_next(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let raw = HOOK_RAW.load(Ordering::Relaxed) as *mut _;
    CallNextHookEx(HHOOK(raw), n_code, w_param, l_param)
}
