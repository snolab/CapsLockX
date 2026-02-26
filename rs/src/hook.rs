/// Low-level keyboard hook and the CapsLockX state machine.
///
/// The hook is installed with `WH_KEYBOARD_LL`.  It runs on the main thread
/// (the one that drives the Win32 message loop).  Heavy work (AccModel ticks,
/// SendInput) happens in separate threads spawned by the modules.
///
/// State machine (mirrors CLX_Dn / CLX_Up in CapsLockX-Core.ahk):
///
/// ```
///  [NORMAL] ── trigger dn ──▶ [FN]  ── trigger up (no other key acted) ──▶ passthrough tap
///                               │
///                 CapsLock+Space within 250 ms
///                               │
///                               ▼
///                           [CLX locked]  ── any trigger tap ──▶ [NORMAL]
/// ```
use parking_lot::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT,
};

use crate::input::{self, CLX_EXTRA_INFO};
use crate::modules;
use crate::state::{self, *};
use crate::vk::*;

// ──────────────────────────────── shared state ───────────────────────────────

/// Raw HHOOK pointer stored as usize for atomic access (0 = not installed)
static HOOK_HANDLE_RAW: AtomicUsize = AtomicUsize::new(0);

/// Set of VK codes currently held by the physical keyboard
static HELD_KEYS: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

// Win32 message constants (not all exported at the top level in windows crate)
const WM_KEYDOWN: u32    = 0x0100;
const WM_KEYUP: u32      = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32   = 0x0105;

// ──────────────────────────────── public API ─────────────────────────────────

/// Install the low-level keyboard hook.  Must be called from the message-loop thread.
pub fn install_hook() {
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowsHookExW, WH_KEYBOARD_LL};

    let hmod = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hhook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0)
            .expect("SetWindowsHookExW failed – are you running without a message loop?")
    };
    HOOK_HANDLE_RAW.store(hhook.0 as usize, Ordering::SeqCst);
    eprintln!("[CLX] keyboard hook installed (handle={:?})", hhook.0);
}

/// Remove the keyboard hook on exit.
pub fn uninstall_hook() {
    use windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;
    let raw = HOOK_HANDLE_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(raw, std::ptr::null()) {
        unsafe { let _ = UnhookWindowsHookEx(HHOOK(raw)); }
        eprintln!("[CLX] keyboard hook removed");
    }
}

// ──────────────────────────────── hook callback ──────────────────────────────

/// Windows low-level keyboard hook procedure.
///
/// # Safety
/// Called by Windows; `l_param` always points to a valid `KBDLLHOOKSTRUCT`.
unsafe extern "system" fn keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // Must pass through if n_code < 0
    if n_code < 0 {
        return call_next(n_code, w_param, l_param);
    }

    let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
    let vk    = kb.vkCode;
    let flags = kb.flags.0;           // raw u32
    let is_key_up   = (flags & LLKHF_UP)       != 0;
    let is_injected = (flags & LLKHF_INJECTED)  != 0;

    // Skip events that we injected ourselves
    if is_injected && kb.dwExtraInfo == CLX_EXTRA_INFO {
        return call_next(n_code, w_param, l_param);
    }

    let msg = w_param.0 as u32;
    let is_real_down = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && !is_key_up;
    let is_real_up   = matches!(msg, WM_KEYUP   | WM_SYSKEYUP)   &&  is_key_up;

    if !is_real_down && !is_real_up {
        return call_next(n_code, w_param, l_param);
    }

    // Maintain held-key set; detect auto-repeat
    let is_repeat = {
        let mut held = HELD_KEYS.lock();
        if is_real_down {
            !held.insert(vk)       // insert returns false if already present → repeat
        } else {
            held.remove(&vk);
            false
        }
    };

    // Track prior key (mirrors AHK A_PriorKey)
    let prior_vk = crate::state::PRIOR_VK.load(Ordering::Relaxed);
    if is_real_down && !is_repeat {
        crate::state::PRIOR_VK.store(vk, Ordering::Relaxed);
    }

    // ── Trigger key handling ──────────────────────────────────────────────────
    if is_trigger_vk(vk) {
        if is_real_down && !is_repeat {
            clx_dn(vk, prior_vk);
        } else if is_real_up {
            clx_up(vk);
        }
        // Always suppress the raw trigger key from reaching applications
        return suppress();
    }

    // ── Non-trigger key in CLX mode ───────────────────────────────────────────
    if is_clx_active() {
        if is_real_down && !is_repeat {
            crate::state::CLX_FN_ACTED.store(true, Ordering::Relaxed);
            if modules::is_mapped_key(vk) {
                modules::on_key_down(vk);
                return suppress();
            }
        } else if is_real_up {
            if modules::is_mapped_key(vk) {
                modules::on_key_up(vk);
                return suppress();
            }
        }
    }

    call_next(n_code, w_param, l_param)
}

// ──────────────────────────────── CLX_Dn ─────────────────────────────────────

/// Called when a trigger key is pressed.  Mirrors AHK `CLX_Dn()`.
fn clx_dn(vk: u32, prior_vk: u32) {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

    // CapsLock+Space or Space+CapsLock chorded within the prior-key window
    let clx_and_space =
        (vk == VK_CAPITAL && prior_vk == VK_SPACE)
        || (vk == VK_SPACE && prior_vk == VK_CAPITAL);

    if clx_and_space {
        state::enter_clx_mode();
        store_trigger(vk);
        return;
    }

    // Bypass: another non-modifier key is currently held while trigger goes down
    // → pass the trigger through as a normal key (don't activate CLX mode)
    let trigger_is_space = vk == VK_SPACE;
    let prior_held = prior_vk != 0 && prior_vk != vk && {
        let ks = unsafe { GetKeyState(prior_vk as i32) };
        (ks as u16) & 0x8000 != 0
    };
    let bypass = !is_modifier_vk(prior_vk) && !trigger_is_space && prior_held;
    if bypass {
        input::key_down(vk);
        input::key_up(vk);
        return;
    }

    // Normal path: enter FN mode
    state::enter_fn_mode();
    store_trigger(vk);
}

// ──────────────────────────────── CLX_Up ─────────────────────────────────────

/// Called when a trigger key is released.  Mirrors AHK `CLX_Up()`.
fn clx_up(vk: u32) {
    let stored_trigger = crate::state::TRIGGER_VK.load(Ordering::Relaxed);
    let fn_acted       = crate::state::CLX_FN_ACTED.load(Ordering::Relaxed);

    state::exit_fn_mode();

    // Single-tap: no mapped key was pressed while trigger was held
    if stored_trigger == vk && !fn_acted {
        if clx_mode() & CM_CLX != 0 {
            // Tap in locked mode → exit CLX mode
            state::exit_clx_mode();
        } else {
            // Tap in FN mode → restore the trigger key's native function
            match vk {
                VK_CAPITAL => toggle_capslock(),
                VK_SPACE   => input::tap_key(VK_SPACE),
                _          => {}
            }
        }
    }

    crate::state::TRIGGER_VK.store(0, Ordering::Relaxed);
    crate::state::CLX_FN_ACTED.store(false, Ordering::Relaxed);
}

// ──────────────────────────────── helpers ────────────────────────────────────

fn store_trigger(vk: u32) {
    crate::state::TRIGGER_VK.store(vk, Ordering::Relaxed);
    crate::state::TRIGGER_PRESS_TICK.store(get_tick64(), Ordering::Relaxed);
    crate::state::CLX_FN_ACTED.store(false, Ordering::Relaxed);
}

/// Toggle the physical CapsLock LED / toggle state by injecting a synthetic
/// CapsLock key press (tagged with CLX_EXTRA_INFO so our hook skips it).
fn toggle_capslock() {
    input::tap_key(VK_CAPITAL);
}

#[inline(always)]
unsafe fn call_next(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let raw = HOOK_HANDLE_RAW.load(Ordering::Relaxed) as *mut _;
    CallNextHookEx(HHOOK(raw), n_code, w_param, l_param)
}

#[inline(always)]
fn suppress() -> LRESULT {
    LRESULT(1)
}

fn get_tick64() -> u64 {
    unsafe { windows::Win32::System::SystemInformation::GetTickCount64() }
}
