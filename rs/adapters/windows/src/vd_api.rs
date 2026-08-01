//! Virtual-desktop COM internal API – raw vtable approach.
//!
//! Mirrors the AHK version: queries `IVirtualDesktopManagerInternal` from the
//! undocumented `ImmersiveShell` service and calls `SwitchDesktop` directly,
//! with no need to know the current desktop position.
//!
//! Handles Win10 / Win11 / Win12 GUID variants automatically.
//! Falls back silently (returns false / None) if the COM interfaces fail.

use std::ffi::c_void;
use std::sync::OnceLock;
use windows::core::GUID;

// ── GUID helpers ──────────────────────────────────────────────────────────────

const fn g(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> GUID {
    GUID {
        data1: d1,
        data2: d2,
        data3: d3,
        data4: d4,
    }
}

const CLSID_IMMERSIVE_SHELL: GUID = g(
    0xC2F03A33,
    0x21F5,
    0x47FA,
    [0xB4, 0xBB, 0x15, 0x63, 0x62, 0xA2, 0xF2, 0x39],
);
const IID_ISERVICE_PROVIDER: GUID = g(
    0x6D5140C1,
    0x7436,
    0x11CE,
    [0x80, 0x34, 0x00, 0xAA, 0x00, 0x60, 0x09, 0xFA],
);
/// Service GUID for IVirtualDesktopManagerInternal
const SID_VDMI: GUID = g(
    0xC5E0CDCA,
    0x7B6E,
    0x41B2,
    [0x9F, 0xC4, 0xD9, 0x39, 0x75, 0xCC, 0x46, 0x7B],
);

// IVirtualDesktopManagerInternal – different IID per Windows build
const IID_VDMI_W10: GUID = g(
    0xF31574D6,
    0xB682,
    0x4CDC,
    [0xBD, 0x56, 0x18, 0x27, 0x86, 0x0A, 0xBE, 0xC6],
);
const IID_VDMI_W11: GUID = g(
    0xB2F925B9,
    0x5A0F,
    0x4D2E,
    [0x9F, 0x4D, 0x2B, 0x15, 0x07, 0x59, 0x3C, 0x10],
);
const IID_VDMI_W12: GUID = g(
    0x53F5CA0B,
    0x158F,
    0x4124,
    [0x90, 0x0C, 0x05, 0x71, 0x58, 0x06, 0x0B, 0x27],
);

// IVirtualDesktop – different IID per Windows build
const IID_VD_W10: GUID = g(
    0xFF72FFDD,
    0xBE7E,
    0x43FC,
    [0x9C, 0x03, 0xAD, 0x81, 0x68, 0x1E, 0x88, 0xE4],
);
const IID_VD_W11: GUID = g(
    0x536D3495,
    0xB208,
    0x4CC9,
    [0xAE, 0x26, 0xDE, 0x81, 0x11, 0x27, 0x5B, 0xF8],
);
const IID_VD_W12: GUID = g(
    0x3F07F4BE,
    0xB107,
    0x441A,
    [0xAF, 0x0F, 0x39, 0xD8, 0x25, 0x29, 0x07, 0x2C],
);

// ── Raw COM import (ole32 is already linked by the windows crate) ─────────────

#[link(name = "ole32")]
extern "system" {
    fn CoCreateInstance(
        rclsid: *const GUID,
        outer: *mut c_void,
        ctx: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> i32;
    fn CoInitializeEx(reserved: *const c_void, co_init: u32) -> i32;
}

const CLSCTX_ALL: u32 = 0x17;
const COINIT_APARTMENTTHREADED: u32 = 0x2;
const S_OK: i32 = 0;

// ── File logger (appends to %TEMP%\capslockx_vd.log) ─────────────────────────

/// Public wrapper so other modules (e.g. the `vd-test` diagnostic) can append
/// to the same virtual-desktop log file.
pub fn log_line(msg: &str) {
    log(msg);
}

fn log(msg: &str) {
    use std::io::Write as _;
    if let Ok(tmp) = std::env::var("TEMP") {
        let path = format!(r"{}\capslockx_vd.log", tmp);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

// ── RAII COM pointer ──────────────────────────────────────────────────────────

struct ComPtr(*mut c_void);

impl Drop for ComPtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let f: unsafe extern "system" fn(*mut c_void) -> u32 = vt(self.0, 2);
                f(self.0);
            }
        }
    }
}

impl ComPtr {
    #[inline]
    fn ptr(&self) -> *mut c_void {
        self.0
    }
}

/// Read vtable function pointer at slot `n` from a COM object.
unsafe fn vt<F: Copy>(obj: *mut c_void, n: usize) -> F {
    let vtbl = *(obj as *mut *const usize);
    std::mem::transmute_copy(&*vtbl.add(n))
}

// ── Cached VDMI pointer (acquired once at startup, reused from hook callback) ─
//
// CRITICAL: the manager is acquired exactly ONCE, on the main STA thread, at
// startup — and the raw pointer is then reused for the life of the process. We
// deliberately do NOT re-`CoCreateInstance` at runtime.
//
// vd_api's public functions run inside the WH_KEYBOARD_LL callback, which is an
// *input-synchronous* context. An outbound COM activation there fails with
// RPC_E_CANTCALLOUT_ININPUTSYNCCALL (0x8001010D) and/or CO_E_NOTINITIALIZED
// (0x800401F0), and worse, blocks the hook callback long enough to risk Windows
// silently evicting the hook (LowLevelHooksTimeout) — i.e. all hotkeys go dead
// while the process stays alive. Reusing the already-created pointer's vtable is
// safe (no outbound activation). If the pointer goes stale (e.g. Explorer
// restarted) calls fail gracefully and we fall back to the Win+Ctrl+Arrow path;
// recovering the pointer requires a clx restart. (A previous attempt to
// auto-re-acquire on failure was reverted for exactly the reasons above.)

struct CachedVdmi {
    ptr: *mut c_void,
    ver: Ver,
}

// SAFETY: COM object lives on main STA thread; hook callback also runs on main thread.
unsafe impl Send for CachedVdmi {}
unsafe impl Sync for CachedVdmi {}

static VDMI_CACHE: OnceLock<Option<CachedVdmi>> = OnceLock::new();

/// Do the `CoCreateInstance(ImmersiveShell)` → `QueryService(VDMI)` dance and
/// return the manager, or `None` if the COM interfaces are unavailable. Called
/// exactly once, from `init`, on the main STA thread — never from the hot path.
fn acquire_fresh() -> Option<CachedVdmi> {
    unsafe {
        let mut sp: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_IMMERSIVE_SHELL,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IID_ISERVICE_PROVIDER,
            &mut sp,
        );
        if hr != S_OK || sp.is_null() {
            log(&format!(
                "[vd_api] init: CoCreateInstance FAILED hr=0x{:08X}",
                hr as u32
            ));
            return None;
        }

        type QsFn = unsafe extern "system" fn(
            *mut c_void,
            *const GUID,
            *const GUID,
            *mut *mut c_void,
        ) -> i32;
        let qs: QsFn = vt(sp, 3);

        let mut result = None;
        for &(ref iid, ver) in &[
            (IID_VDMI_W12, Ver::W12),
            (IID_VDMI_W11, Ver::W11),
            (IID_VDMI_W10, Ver::W10),
        ] {
            let mut mgr: *mut c_void = std::ptr::null_mut();
            let hr2 = qs(sp, &SID_VDMI, iid, &mut mgr);
            if hr2 == S_OK && !mgr.is_null() {
                let ver_name = match ver {
                    Ver::W10 => "W10",
                    Ver::W11 => "W11",
                    Ver::W12 => "W12",
                };
                log(&format!("[vd_api] init: acquired VDMI ver={}", ver_name));
                result = Some(CachedVdmi { ptr: mgr, ver });
                break;
            }
        }

        // Release IServiceProvider (manager has its own ref)
        let release: unsafe extern "system" fn(*mut c_void) -> u32 = vt(sp, 2);
        release(sp);

        if result.is_none() {
            log("[vd_api] init: QueryService failed for all version GUIDs");
        }
        result
    }
}

/// Initialize COM and pre-acquire the virtual desktop manager interface.
/// Must be called on the main thread before the keyboard hook is installed.
pub fn init() {
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED);
    }
    VDMI_CACHE.get_or_init(acquire_fresh);
}

// ── Windows-version variant ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Ver {
    W10,
    W11,
    W12,
}

impl Ver {
    /// Only the Win11 vtable variant takes an extra null `*mut c_void` after
    /// `this`. W10 and W12 use the plain `(this, out)` form — this matches the
    /// working AHK implementation (`SwitchToDesktopByInternalAPI`), where the
    /// win12 branch calls GetDesktops/SwitchDesktop with NO extra arg, same as
    /// win10; only the win11 branch inserts `"Ptr", 0`. Grouping W12 with W11
    /// here was the bug that made GetDesktops always fail on Win11 24H2/W12
    /// builds, forcing the slow Win+Ctrl+Arrow hotkey fallback.
    fn needs_extra_arg(self) -> bool {
        matches!(self, Ver::W11)
    }
    fn desktop_iid(self) -> GUID {
        match self {
            Ver::W10 => IID_VD_W10,
            Ver::W11 => IID_VD_W11,
            Ver::W12 => IID_VD_W12,
        }
    }
}

// ── IVirtualDesktopManagerInternal wrapper ────────────────────────────────────

struct Manager(ComPtr, Ver);

/// Return a Manager backed by the cached VDMI pointer (acquired at startup).
/// AddRefs so the caller's Drop doesn't release the cached pointer.
fn acquire() -> Option<Manager> {
    let cached = VDMI_CACHE.get()?.as_ref()?;
    unsafe {
        // AddRef so the caller's ComPtr::drop doesn't release our cached pointer.
        let addref: unsafe extern "system" fn(*mut c_void) -> u32 = vt(cached.ptr, 1);
        addref(cached.ptr);
    }
    Some(Manager(ComPtr(cached.ptr), cached.ver))
}

impl Manager {
    /// vtable[7]: GetDesktops(this, [0,] **IObjectArray)
    unsafe fn get_desktops(&self) -> Option<ComPtr> {
        let mut arr: *mut c_void = std::ptr::null_mut();
        let hr = if self.1.needs_extra_arg() {
            let f: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> i32 =
                vt(self.0.ptr(), 7);
            f(self.0.ptr(), std::ptr::null_mut(), &mut arr)
        } else {
            let f: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32 =
                vt(self.0.ptr(), 7);
            f(self.0.ptr(), &mut arr)
        };
        if hr == S_OK && !arr.is_null() {
            Some(ComPtr(arr))
        } else {
            None
        }
    }

    /// vtable[6]: GetCurrentDesktop(this, [0,] **IVirtualDesktop)
    unsafe fn get_current_desktop(&self) -> Option<ComPtr> {
        let mut d: *mut c_void = std::ptr::null_mut();
        let hr = if self.1.needs_extra_arg() {
            let f: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> i32 =
                vt(self.0.ptr(), 6);
            f(self.0.ptr(), std::ptr::null_mut(), &mut d)
        } else {
            let f: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32 =
                vt(self.0.ptr(), 6);
            f(self.0.ptr(), &mut d)
        };
        if hr == S_OK && !d.is_null() {
            Some(ComPtr(d))
        } else {
            None
        }
    }

    /// vtable[9]: SwitchDesktop(this, [0,] *IVirtualDesktop)
    unsafe fn switch_to(&self, desktop: *mut c_void) -> bool {
        let hr = if self.1.needs_extra_arg() {
            let f: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32 =
                vt(self.0.ptr(), 9);
            f(self.0.ptr(), std::ptr::null_mut(), desktop)
        } else {
            let f: unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32 = vt(self.0.ptr(), 9);
            f(self.0.ptr(), desktop)
        };
        hr == S_OK
    }
}

// ── IObjectArray helpers ──────────────────────────────────────────────────────

unsafe fn arr_count(arr: *mut c_void) -> u32 {
    let mut n: u32 = 0;
    let f: unsafe extern "system" fn(*mut c_void, *mut u32) -> i32 = vt(arr, 3);
    f(arr, &mut n);
    n
}

/// `IVirtualDesktop::GetID(this, GUID* out)` – vtable slot 4 on every Windows
/// build we support (W10 / W11 / W11 24H2 "W12" all keep `IsViewVisible` at 3
/// and `GetID` at 4; only the trailing name/wallpaper methods differ).
unsafe fn desktop_id(d: *mut c_void) -> Option<GUID> {
    let mut id = g(0, 0, 0, [0; 8]);
    let f: unsafe extern "system" fn(*mut c_void, *mut GUID) -> i32 = vt(d, 4);
    if f(d, &mut id) == S_OK {
        Some(id)
    } else {
        None
    }
}

unsafe fn arr_get_at(arr: *mut c_void, i: u32, iid: &GUID) -> Option<ComPtr> {
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let f: unsafe extern "system" fn(*mut c_void, u32, *const GUID, *mut *mut c_void) -> i32 =
        vt(arr, 4);
    if f(arr, i, iid, &mut ptr) == S_OK && !ptr.is_null() {
        Some(ComPtr(ptr))
    } else {
        None
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Switch to the given 1-based desktop index via the internal COM API.
/// Returns `true` on success, `false` if the API is unavailable (e.g. Win7/8)
/// or the index is out of range.
pub fn switch_desktop(idx: usize) -> bool {
    let Some(mgr) = acquire() else { return false };
    unsafe {
        let Some(arr) = mgr.get_desktops() else {
            log("[vd_api] switch_desktop: get_desktops failed");
            return false;
        };
        let iid = mgr.1.desktop_iid();
        let Some(desktop) = arr_get_at(arr.ptr(), (idx - 1) as u32, &iid) else {
            log(&format!(
                "[vd_api] switch_desktop: GetAt({}) failed",
                idx - 1
            ));
            return false;
        };
        let ok = mgr.switch_to(desktop.ptr());
        log(&format!("[vd_api] switch_desktop({}) -> {}", idx, ok));
        ok
    }
}

/// Total number of virtual desktops, or `None` if the API is unavailable.
pub fn desktop_count() -> Option<usize> {
    let mgr = acquire()?;
    unsafe {
        let arr = mgr.get_desktops()?;
        Some(arr_count(arr.ptr()) as usize)
    }
}

/// Query the real current 1-based desktop index from the OS.
/// Returns `None` if the API is unavailable.
///
/// This is the Rust equivalent of AHK's `GetCurrentVirtualDesktopIdxFromAPI`:
/// `GetCurrentDesktop()` gives an opaque `IVirtualDesktop`, which only becomes
/// a *position* by locating it inside the `GetDesktops()` array.
pub fn current_desktop_idx() -> Option<usize> {
    let mgr = acquire()?;
    unsafe {
        let current = mgr.get_current_desktop()?;
        let arr = mgr.get_desktops()?;
        let iid = mgr.1.desktop_iid();
        let count = arr_count(arr.ptr());
        // Match on the desktop's stable GUID rather than on pointer identity.
        // AHK compared raw pointers; that happens to work today but COM only
        // guarantees pointer identity for `IUnknown`, so a proxy handed back by
        // a later `GetAt` call may legitimately differ from the one
        // `GetCurrentDesktop` returned. Pointer equality stays as the fallback
        // for any build where `GetID` isn't at vtable[4].
        let cur_id = desktop_id(current.ptr());
        for i in 0..count {
            if let Some(d) = arr_get_at(arr.ptr(), i, &iid) {
                let hit = match (cur_id, desktop_id(d.ptr())) {
                    (Some(a), Some(b)) => a == b,
                    _ => d.ptr() == current.ptr(),
                };
                if hit {
                    log(&format!(
                        "[vd_api] current_desktop_idx -> {}/{} (by {})",
                        i + 1,
                        count,
                        if cur_id.is_some() { "guid" } else { "ptr" }
                    ));
                    return Some(i as usize + 1);
                }
            }
        }
        log(&format!(
            "[vd_api] current_desktop_idx: no match among {} desktops",
            count
        ));
        None
    }
}

/// Read-only diagnostic dump: manager version, desktop count, current index and
/// every desktop GUID. Written to `%TEMP%\capslockx_vd.log` (`clx vd-test`).
pub fn dump() {
    let Some(mgr) = acquire() else {
        log("[vd_api] dump: no manager (COM interface unavailable)");
        return;
    };
    unsafe {
        let ver = match mgr.1 {
            Ver::W10 => "W10",
            Ver::W11 => "W11",
            Ver::W12 => "W12",
        };
        let cur_id = mgr.get_current_desktop().and_then(|c| desktop_id(c.ptr()));
        let Some(arr) = mgr.get_desktops() else {
            log(&format!("[vd_api] dump: ver={} get_desktops FAILED", ver));
            return;
        };
        let iid = mgr.1.desktop_iid();
        let count = arr_count(arr.ptr());
        log(&format!("[vd_api] dump: ver={} count={}", ver, count));
        for i in 0..count {
            if let Some(d) = arr_get_at(arr.ptr(), i, &iid) {
                let id = desktop_id(d.ptr());
                let mark = if id.is_some() && id == cur_id {
                    " <== CURRENT"
                } else {
                    ""
                };
                log(&format!("[vd_api]   #{} {:?}{}", i + 1, id, mark));
            }
        }
    }
}
