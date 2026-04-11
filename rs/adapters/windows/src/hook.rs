/// Windows WH_KEYBOARD_LL hook – bridges Win32 key events to ClxEngine.
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    SetTimer,
};

use capslockx_core::{ClxConfig, ClxEngine, CoreResponse};
use crate::output::{WinPlatform, CLX_EXTRA_INFO};
use crate::shm::SharedState;
use crate::vk::vk_to_keycode;

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

const WM_KEYDOWN:    u32 = 0x0100;
const WM_KEYUP:      u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP:   u32 = 0x0105;
const LLKHF_UP:      u32 = 0x80;
const LLKHF_INJECTED:u32 = 0x10;

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
    ENGINE.get().expect("init_engine must be called before engine()").clone()
}

/// Install the WH_KEYBOARD_LL hook and a 16 ms timer for AccModel ticks.
///
/// **IMPORTANT:** Must be called from the thread that runs the Win32 message
/// loop (e.g. inside Tauri's `setup` callback).  WH_KEYBOARD_LL hooks require
/// the installing thread to pump messages — installing on a thread that doesn't
/// call GetMessage/DispatchMessage causes Windows to silently disable the hook.
pub fn install_hook() {
    let hmod = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hhook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0)
            .expect("SetWindowsHookExW failed")
    };
    HOOK_RAW.store(hhook.0 as usize, Ordering::SeqCst);

    // Drive AccModel ticks on the main/hook thread (16 ms timer).
    // This replaces background ticker threads so SendInput runs on the same
    // thread as the keyboard hook, avoiding phantom modifier key-up events.
    unsafe {
        SetTimer(None, 0, 16, Some(tick_timer_proc));
    }
}

/// Timer callback — drives AccModel physics on the main/hook thread, and
/// periodically nudges cursor visibility while CLX mode is held. Nudging used
/// to happen inside the hook callback; moving it here keeps the hook callback
/// fast and keeps us well under the WH_KEYBOARD_LL ~300 ms budget.
unsafe extern "system" fn tick_timer_proc(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    if let Some(engine) = ENGINE.get() {
        engine.tick();
    }
    let active = LAST_TRAY_ACTIVE.load(Ordering::Relaxed);
    if active != 0 && active != u32::MAX {
        crate::cursor_visibility::nudge();
    }
}

pub fn uninstall_hook() {
    let raw = HOOK_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(raw, std::ptr::null()) {
        unsafe { let _ = UnhookWindowsHookEx(HHOOK(raw)); }
    }
}

// ── Hook callback ─────────────────────────────────────────────────────────────

unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 { return call_next(n_code, w_param, l_param); }

    let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
    let flags = kb.flags.0;

    let msg = w_param.0 as u32;
    let is_up   = (flags & LLKHF_UP) != 0;
    let pressed = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && !is_up;
    let released= matches!(msg, WM_KEYUP   | WM_SYSKEYUP)   &&  is_up;

    let code = vk_to_keycode(kb.vkCode);
    let injected = (flags & LLKHF_INJECTED) != 0;
    let is_ours = injected && kb.dwExtraInfo == CLX_EXTRA_INFO;

    // Debug: log ALL key events
    if pressed || released {
        debug_log(&format!("[hook] vk=0x{:02X} {:?} {} inj={} ours={} extra=0x{:X}",
            kb.vkCode, code, if pressed { "DN" } else { "UP" }, injected, is_ours, kb.dwExtraInfo));
    }

    // Skip events injected by us
    if is_ours {
        return call_next(n_code, w_param, l_param);
    }

    if !pressed && !released { return call_next(n_code, w_param, l_param); }

    let engine = ENGINE.get().expect("init_engine not called");
    let resp = engine.on_key_event(code, pressed);
    debug_log(&format!("[hook] -> {:?} mode={}", resp, engine.state().mode()));

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
        CoreResponse::Suppress    => LRESULT(1),
        CoreResponse::PassThrough => call_next(n_code, w_param, l_param),
    }
}

#[allow(dead_code)]
pub fn debug_log(msg: &str) {
    use std::io::Write as _;
    if let Ok(tmp) = std::env::var("TEMP") {
        let path = format!(r"{}\capslockx_hook.log", tmp);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

#[inline(always)]
unsafe fn call_next(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let raw = HOOK_RAW.load(Ordering::Relaxed) as *mut _;
    CallNextHookEx(HHOOK(raw), n_code, w_param, l_param)
}
