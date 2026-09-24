//! Independent, release-only reconciliation for a stalled/removed LL hook.
//!
//! Suppressed keys never reach raw input. That is fine: their normal UP is
//! already handled by the hook. An unsuppressed BREAK can recover a hook-latched
//! key when the hook stops being serviced. Never infer "up" from silence or
//! GetAsyncKeyState; the latter also reads suppressed, held keys as up.

use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use windows::core::{w, Result};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, GetThreadDesktop, GetUserObjectInformationW, OpenInputDesktop,
    DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, UOI_NAME,
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageTime, GetMessageW,
    RegisterClassW, SetTimer, HWND_MESSAGE, MSG, WM_INPUT, WM_TIMER, WNDCLASSW,
};

use crate::output::CLX_EXTRA_INFO;
use crate::release_state::{key_slot, KEY_SLOTS};

// Fixed slots avoid an unbounded queue when the hook/UI thread is stalled.
// Keeping the latest release of a physical key suffices for reconciliation.
const PRESENT: u64 = 1 << 32;
static RELEASES: [AtomicU64; KEY_SLOTS] = [const { AtomicU64::new(0) }; KEY_SLOTS];
static DIRTY: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

/// Every physical keyboard event seen here, make and break alike.
///
/// This exists as an independent witness that the input path is live. Raw input
/// only receives what no hook suppressed, so in normal operation each event
/// counted here was also offered to `WH_KEYBOARD_LL` — see
/// `hook::check_hook_alive`, which compares the two.
static KEY_EVENTS: AtomicU64 = AtomicU64::new(0);

pub fn key_event_count() -> u64 {
    KEY_EVENTS.load(Ordering::Relaxed)
}

pub fn take_cancel() -> bool {
    CANCEL.swap(false, Ordering::AcqRel)
}

pub fn drain_releases(mut release: impl FnMut(usize, u32)) {
    if !DIRTY.swap(false, Ordering::AcqRel) {
        return;
    }
    for (slot, pending) in RELEASES.iter().enumerate() {
        let value = pending.swap(0, Ordering::AcqRel);
        if value & PRESENT != 0 {
            release(slot, value as u32);
        }
    }
}

pub fn start() {
    // The HWND MUST be created on this thread. Owning it on the Tauri thread
    // would queue WM_INPUT behind the same stalled pump as WH_KEYBOARD_LL.
    if let Err(error) = std::thread::Builder::new()
        .name("clx-raw-release".into())
        .spawn(|| {
            if let Err(error) = unsafe { run() } {
                crate::hook::crash_log_sync(&format!(
                    "[input-reconcile] receiver unavailable: {error}"
                ));
            }
        })
    {
        crate::hook::crash_log_sync(&format!("[input-reconcile] cannot start receiver: {error}"));
    }
}

unsafe fn desktop_name(desktop: HANDLE) -> Option<Vec<u16>> {
    let mut name = [0u16; 256];
    GetUserObjectInformationW(
        desktop,
        UOI_NAME,
        Some(name.as_mut_ptr().cast()),
        size_of_val(&name) as u32,
        None,
    )
    .ok()?;
    let end = name.iter().position(|c| *c == 0)?;
    Some(name[..end].to_vec())
}

unsafe fn input_desktop_available(ours: &[u16]) -> bool {
    let Ok(desktop) = OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS) else {
        return false;
    };
    let name = desktop_name(HANDLE(desktop.0));
    let _ = CloseDesktop(desktop);
    name.as_deref() == Some(ours)
}

unsafe fn run() -> Result<()> {
    let instance = GetModuleHandleW(None)?;
    let class = w!("CapsLockX.RawRelease");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance.into(),
        lpszClassName: class,
        ..Default::default()
    };
    if RegisterClassW(&wc) == 0 {
        return Err(windows::core::Error::from_win32());
    }
    let hwnd = CreateWindowExW(
        Default::default(),
        class,
        w!(""),
        Default::default(),
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        None,
        instance,
        None,
    )?;
    let device = RAWINPUTDEVICE {
        usUsagePage: 1,
        usUsage: 6,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };
    if let Err(error) = RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32) {
        let _ = DestroyWindow(hwnd);
        return Err(error);
    }
    crate::hook::crash_log_sync("[input-reconcile] dedicated raw release receiver ready");

    // Failure to identify our own desktop disables this optional guard, not
    // normal input. Once identified, losing access (UAC/lock/session change)
    // deliberately cancels even a legitimately held gesture: stop is safe.
    let ours = GetThreadDesktop(GetCurrentThreadId())
        .ok()
        .and_then(|desktop| desktop_name(HANDLE(desktop.0)));
    let mut available = true;
    let mut last_check = GetTickCount64();
    SetTimer(hwnd, 1, 100, None);
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
        if crate::SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }
        if msg.message == WM_TIMER {
            let now = GetTickCount64(); // Includes suspend time on Windows.
            let resumed = now.saturating_sub(last_check) > 2_000;
            last_check = now;
            if let Some(ref ours) = ours {
                let next = input_desktop_available(ours);
                if (available && !next) || resumed {
                    CANCEL.store(true, Ordering::Release);
                }
                available = next;
            } else if resumed {
                CANCEL.store(true, Ordering::Release);
            }
        }
        DispatchMessageW(&msg);
    }
    let remove = RAWINPUTDEVICE {
        dwFlags: RIDEV_REMOVE,
        hwndTarget: HWND::default(),
        ..device
    };
    let _ = RegisterRawInputDevices(&[remove], size_of::<RAWINPUTDEVICE>() as u32);
    let _ = DestroyWindow(hwnd);
    Ok(())
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_INPUT {
        // Never unwind through a Win32 callback. This path only publishes
        // atomics; engine callbacks and input injection happen elsewhere.
        let _ = std::panic::catch_unwind(|| read_release(lp));
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

unsafe fn read_release(lp: LPARAM) {
    let mut raw = RAWINPUT::default();
    let mut size = size_of::<RAWINPUT>() as u32;
    let n = GetRawInputData(
        HRAWINPUT(lp.0 as *mut _),
        RID_INPUT,
        Some((&mut raw as *mut RAWINPUT).cast()),
        &mut size,
        size_of::<RAWINPUTHEADER>() as u32,
    );
    if n == u32::MAX
        || n < (size_of::<RAWINPUTHEADER>() + 16) as u32
        || raw.header.dwType != RIM_TYPEKEYBOARD.0
    {
        return;
    }
    let key = raw.data.keyboard;
    if key.ExtraInformation == CLX_EXTRA_INFO as u32 {
        return;
    }
    // Counted before the release-only filter below: the liveness witness wants
    // every physical event, whereas reconciliation only cares about breaks.
    KEY_EVENTS.fetch_add(1, Ordering::Relaxed);
    if key.Flags & 1 == 0 {
        return;
    }
    if let Some(slot) = key_slot(key.MakeCode as u32, key.Flags & 2 != 0, key.VKey as u32) {
        RELEASES[slot].store(PRESENT | GetMessageTime() as u32 as u64, Ordering::Release);
        DIRTY.store(true, Ordering::Release);
    }
}
