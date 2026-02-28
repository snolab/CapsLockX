/// Windows WH_KEYBOARD_LL hook – bridges Win32 key events to ClxEngine.
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
};

use capslockx_core::{ClxConfig, ClxEngine, CoreResponse};
use crate::output::{WinPlatform, CLX_EXTRA_INFO};
use crate::vk::vk_to_keycode;

// ── Raw HHOOK stored as usize for atomic access ───────────────────────────────

static HOOK_RAW: AtomicUsize = AtomicUsize::new(0);

// ── Engine (initialised once via init_engine before hook installs) ────────────

static ENGINE: OnceLock<Arc<ClxEngine>> = OnceLock::new();

// ── Win32 message constants ───────────────────────────────────────────────────

const WM_KEYDOWN:    u32 = 0x0100;
const WM_KEYUP:      u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP:   u32 = 0x0105;
const LLKHF_UP:      u32 = 0x80;
const LLKHF_INJECTED:u32 = 0x10;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn init_engine(config: ClxConfig) {
    let platform = Arc::new(WinPlatform::new());
    ENGINE.set(ClxEngine::with_config(platform, config)).ok();
}

pub fn engine() -> Arc<ClxEngine> {
    ENGINE.get().expect("init_engine must be called before engine()").clone()
}

pub fn install_hook() {
    let hmod = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hhook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0)
            .expect("SetWindowsHookExW failed")
    };
    HOOK_RAW.store(hhook.0 as usize, Ordering::SeqCst);
    eprintln!("[CLX] keyboard hook installed ({:?})", hhook.0);
}

pub fn uninstall_hook() {
    let raw = HOOK_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(raw, std::ptr::null()) {
        unsafe { let _ = UnhookWindowsHookEx(HHOOK(raw)); }
        eprintln!("[CLX] keyboard hook removed");
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

    // Skip events injected by us
    if (flags & LLKHF_INJECTED) != 0 && kb.dwExtraInfo == CLX_EXTRA_INFO {
        return call_next(n_code, w_param, l_param);
    }

    let msg = w_param.0 as u32;
    let is_up   = (flags & LLKHF_UP) != 0;
    let pressed = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && !is_up;
    let released= matches!(msg, WM_KEYUP   | WM_SYSKEYUP)   &&  is_up;

    if !pressed && !released { return call_next(n_code, w_param, l_param); }

    let code = vk_to_keycode(kb.vkCode);

    match ENGINE.get().expect("init_engine not called").on_key_event(code, pressed) {
        CoreResponse::Suppress    => LRESULT(1),
        CoreResponse::PassThrough => call_next(n_code, w_param, l_param),
    }
}

#[inline(always)]
unsafe fn call_next(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let raw = HOOK_RAW.load(Ordering::Relaxed) as *mut _;
    CallNextHookEx(HHOOK(raw), n_code, w_param, l_param)
}
