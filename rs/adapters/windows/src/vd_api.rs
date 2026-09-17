//! Virtual-desktop COM internal API – raw vtable approach.
//!
//! Mirrors the AHK version: queries `IVirtualDesktopManagerInternal` from the
//! undocumented `ImmersiveShell` service and calls `SwitchDesktop` directly,
//! with no need to know the current desktop position.
//!
//! Handles Win10 / Win11 / Win12 GUID variants automatically.
//!
//! # Threading
//!
//! `ImmersiveShell` is hosted in explorer.exe, so every method call is an
//! outbound cross-process COM call. Those are *refused* from inside the
//! `WH_KEYBOARD_LL` callback (an input-synchronous context –
//! `RPC_E_CANTCALLOUT_ININPUTSYNCCALL`), which is why an earlier design that
//! cached a manager pointer on the main thread and called it from the hook
//! logged `get_desktops failed` on every keypress and always fell back to
//! Win+Ctrl+Arrow. AHK never hits this because its hotkey bodies run on the
//! main thread *after* the hook has returned.
//!
//! So the hook never touches COM: [`switch_desktop`], [`step_desktop`] and
//! [`move_window_to_desktop`] just post a request to the `clx-vd-worker`
//! thread, which owns its own STA apartment and its own manager pointer and
//! performs the COM call (falling back to the hotkey path when COM fails).
//! Owning the pointer on the worker also makes it safe to re-acquire when the
//! pointer goes stale (explorer restart) – something the hook-side design
//! could not do.

use std::ffi::c_void;
use std::sync::mpsc;
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

/// IApplicationViewCollection – service GUID == IID, stable across Win10/11.
/// Needed to turn an HWND into the `IApplicationView` that
/// `IVirtualDesktopManagerInternal::MoveViewToDesktop` wants.
const IID_APP_VIEW_COLLECTION: GUID = g(
    0x1841C6D7,
    0x4F9D,
    0x42C0,
    [0xAF, 0x41, 0x87, 0x47, 0x53, 0x8F, 0x10, 0xE5],
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

// ── Manager acquisition ───────────────────────────────────────────────────────

/// Do the `CoCreateInstance(ImmersiveShell)` → `QueryService(VDMI)` dance and
/// return the manager, or `None` if the COM interfaces are unavailable.
///
/// Must be called on a thread that has initialised COM and is *not* inside an
/// input-synchronous callback (see module docs).
fn acquire_fresh() -> Option<Manager> {
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
                "[vd_api] acquire: CoCreateInstance FAILED hr=0x{:08X}",
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
                log(&format!("[vd_api] acquire: VDMI ver={}", ver.name()));
                result = Some(Manager(ComPtr(mgr), ver, None));
                break;
            }
        }

        // The view collection is optional: without it window moves fall back
        // to hide/switch/show, but desktop switching still works.
        if let Some(m) = result.as_mut() {
            let mut views: *mut c_void = std::ptr::null_mut();
            let hr3 = qs(
                sp,
                &IID_APP_VIEW_COLLECTION,
                &IID_APP_VIEW_COLLECTION,
                &mut views,
            );
            if hr3 == S_OK && !views.is_null() {
                m.2 = Some(ComPtr(views));
            } else {
                log(&format!(
                    "[vd_api] acquire: IApplicationViewCollection FAILED hr=0x{:08X}",
                    hr3 as u32
                ));
            }
        }

        // Release IServiceProvider (manager has its own ref)
        let release: unsafe extern "system" fn(*mut c_void) -> u32 = vt(sp, 2);
        release(sp);

        if result.is_none() {
            log("[vd_api] acquire: QueryService failed for all version GUIDs");
        }
        result
    }
}

/// Initialise COM on the calling thread (STA, like AHK and the main thread).
fn co_init_sta() {
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED);
    }
}

// ── Worker thread ─────────────────────────────────────────────────────────────

/// A request from the hook to the worker. Everything the hook needs to know
/// is captured up front (e.g. the foreground HWND), so the hook returns
/// immediately and the worker does all the slow / COM work.
enum Req {
    /// Switch to the 1-based desktop index.
    Switch(usize),
    /// Step one desktop left (-1) or right (+1) of the current one.
    Step(i32),
    /// Like `Step` but wrapping around at the ends, then focus the first (+1)
    /// / last (-1) app window on the desktop we land on, so window cycling
    /// forms one closed loop across every desktop.
    StepFocus(i32),
    /// Move `hwnd` to the 1-based desktop index and follow it.
    MoveWindow(isize, usize),
}

static VD_TX: OnceLock<mpsc::Sender<Req>> = OnceLock::new();

/// Spawn the `clx-vd-worker` thread. Call once at startup, before the
/// keyboard hook is installed, so the first Space+digit already has somewhere
/// to go. Idempotent.
pub fn init() {
    let (tx, rx) = mpsc::channel::<Req>();
    if VD_TX.set(tx).is_err() {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("clx-vd-worker".into())
        .spawn(move || {
            co_init_sta();
            let mut worker = Worker {
                mgr: acquire_fresh(),
                tracked_idx: 1,
            };
            while let Ok(req) = rx.recv() {
                // Firewall each request like the tray worker does: a panic in
                // a COM call must not kill the worker for the rest of the
                // session (every later Space+digit would silently no-op).
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    worker.handle(req);
                }));
                if r.is_err() {
                    crate::hook::crash_log_sync("[PANIC] recovered in clx-vd-worker");
                }
            }
        });
}

fn post(req: Req) {
    match VD_TX.get() {
        Some(tx) => {
            let _ = tx.send(req);
        }
        None => log("[vd_api] request dropped: worker not initialised"),
    }
}

/// Worker-thread state: the COM manager (re-acquired on demand) plus the last
/// desktop index we believe we are on, used only when COM cannot tell us.
struct Worker {
    mgr: Option<Manager>,
    tracked_idx: usize,
}

impl Worker {
    /// Run `f` against the manager; if it fails, re-acquire once (explorer may
    /// have restarted and left us with a dead proxy) and retry.
    fn with_mgr<T>(&mut self, f: impl Fn(&Manager) -> Option<T>) -> Option<T> {
        if let Some(m) = &self.mgr {
            if let Some(v) = f(m) {
                return Some(v);
            }
        }
        log("[vd_api] manager call failed – re-acquiring");
        self.mgr = acquire_fresh();
        self.mgr.as_ref().and_then(|m| f(m))
    }

    fn current_idx(&mut self) -> Option<usize> {
        self.with_mgr(|m| unsafe { m.current_index() })
    }

    /// COM switch with fallback to Win+Ctrl+Arrow.
    fn switch(&mut self, idx: usize) {
        if self
            .with_mgr(|m| unsafe { m.switch_to_index(idx) })
            .is_some()
        {
            log(&format!("[vd_api] switch_desktop({}) via COM", idx));
            self.tracked_idx = idx;
            return;
        }
        let cur = self.current_idx().unwrap_or(self.tracked_idx);
        log(&format!(
            "[vd_api] switch_desktop({}) via hotkey fallback from {}",
            idx, cur
        ));
        if cur != idx {
            crate::output::navigate_desktops(cur, idx);
        }
        self.tracked_idx = idx;
    }

    /// Step one desktop in `dir`. With `wrap`, stepping past the last desktop
    /// lands on the first (and vice versa) so window cycling forms a closed
    /// loop; without it the step clamps at the edges. Returns whether a
    /// switch was issued.
    fn step(&mut self, dir: i32, wrap: bool) -> bool {
        if let Some(cur) = self.current_idx() {
            let count = self
                .with_mgr(|m| unsafe { m.count() })
                .unwrap_or(usize::MAX)
                .max(1) as i64;
            let raw = cur as i64 + dir as i64;
            let next = if wrap {
                ((raw - 1).rem_euclid(count) + 1) as usize
            } else {
                raw.clamp(1, count) as usize
            };
            if next == cur {
                return false;
            }
            self.switch(next);
        } else {
            // No COM at all: blind Win+Ctrl+Arrow, one step (cannot wrap).
            crate::output::navigate_desktops_step(dir);
        }
        true
    }

    fn handle(&mut self, req: Req) {
        match req {
            Req::Switch(idx) => self.switch(idx),
            Req::Step(dir) => {
                self.step(dir, false);
            }
            Req::StepFocus(dir) => {
                if self.step(dir, true) {
                    focus_edge_window(dir);
                }
            }
            Req::MoveWindow(hwnd, idx) => self.move_window(hwnd, idx),
        }
    }

    fn move_window(&mut self, hwnd: isize, idx: usize) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            SetForegroundWindow, ShowWindow, SW_HIDE, SW_SHOW,
        };
        let raw = hwnd as *mut c_void;
        if raw.is_null() {
            return;
        }
        let hwnd = HWND(raw);
        let cur = self.current_idx().unwrap_or(self.tracked_idx);
        if cur == idx {
            return;
        }

        // Preferred: re-home the window via MoveViewToDesktop *first*, then
        // switch and activate. The window is already on the target desktop
        // by the time we activate it, so nothing can yank us back.
        if self
            .with_mgr(|m| unsafe { m.move_window_to_index(raw, idx) })
            .is_some()
        {
            log(&format!("[vd_api] move_window({}) via COM", idx));
            self.switch(idx);
            unsafe {
                let _ = SetForegroundWindow(hwnd);
            }
            return;
        }

        // Fallback – AHK's MoveActiveWindowToDesktop trick: hide the window,
        // switch, show it again and it re-appears on the (new) current
        // desktop. SwitchDesktop returns before explorer has actually finished
        // the switch, and re-showing + activating the window while it is still
        // bound to the old desktop makes Windows switch *back* to that desktop
        // (a brief flash of the target desktop, then back). So wait until the
        // OS reports the target desktop as current before showing the window.
        // SW_SHOW, not SW_RESTORE: RESTORE would un-maximize the window.
        log(&format!(
            "[vd_api] move_window({}) via hide/switch/show fallback",
            idx
        ));
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        self.switch(idx);
        for _ in 0..50 {
            if self.current_idx().map_or(true, |c| c == idx) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

// ── Windows-version variant ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Ver {
    W10,
    W11,
    W12,
}

impl Ver {
    fn name(self) -> &'static str {
        match self {
            Ver::W10 => "W10",
            Ver::W11 => "W11",
            Ver::W12 => "W12",
        }
    }
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

/// `IVirtualDesktopManagerInternal`, the vtable variant it was acquired as,
/// and (if available) the `IApplicationViewCollection` used for window moves.
/// Lives on exactly one thread (the worker, or the main thread for
/// `vd-test`) – it is deliberately neither `Send` nor `Sync`.
struct Manager(ComPtr, Ver, Option<ComPtr>);

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
            log(&format!(
                "[vd_api] GetDesktops FAILED hr=0x{:08X}",
                hr as u32
            ));
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
            log(&format!(
                "[vd_api] GetCurrentDesktop FAILED hr=0x{:08X}",
                hr as u32
            ));
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
        if hr != S_OK {
            log(&format!(
                "[vd_api] SwitchDesktop FAILED hr=0x{:08X}",
                hr as u32
            ));
        }
        hr == S_OK
    }

    // ── High-level operations (all run on the owning thread) ──────────────

    /// Switch to the 1-based desktop index. `None` on any COM failure or if
    /// the index is out of range.
    unsafe fn switch_to_index(&self, idx: usize) -> Option<()> {
        let arr = self.get_desktops()?;
        let iid = self.1.desktop_iid();
        let Some(desktop) = arr_get_at(arr.ptr(), (idx - 1) as u32, &iid) else {
            log(&format!("[vd_api] GetAt({}) failed", idx - 1));
            return None;
        };
        self.switch_to(desktop.ptr()).then_some(())
    }

    /// Move the top-level window `hwnd` (any process) to the 1-based desktop
    /// index without switching. This is how every virtual-desktop tool does it
    /// (the public `IVirtualDesktopManager::MoveWindowToDesktop` only accepts
    /// the caller's own windows):
    ///   `IApplicationViewCollection::GetViewForHwnd` (vtable[6]) → view,
    ///   `IVirtualDesktopManagerInternal::MoveViewToDesktop` (vtable[4],
    ///   `(this, view, desktop)` on every build – no monitor arg even on W11).
    unsafe fn move_window_to_index(&self, hwnd: *mut c_void, idx: usize) -> Option<()> {
        let views = self.2.as_ref()?;
        let mut view: *mut c_void = std::ptr::null_mut();
        let get_view: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> i32 =
            vt(views.ptr(), 6);
        let hr = get_view(views.ptr(), hwnd, &mut view);
        if hr != S_OK || view.is_null() {
            log(&format!(
                "[vd_api] GetViewForHwnd FAILED hr=0x{:08X}",
                hr as u32
            ));
            return None;
        }
        let view = ComPtr(view);

        let arr = self.get_desktops()?;
        let iid = self.1.desktop_iid();
        let Some(desktop) = arr_get_at(arr.ptr(), (idx - 1) as u32, &iid) else {
            log(&format!("[vd_api] GetAt({}) failed", idx - 1));
            return None;
        };

        let move_view: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32 =
            vt(self.0.ptr(), 4);
        let hr = move_view(self.0.ptr(), view.ptr(), desktop.ptr());
        if hr != S_OK {
            log(&format!(
                "[vd_api] MoveViewToDesktop FAILED hr=0x{:08X}",
                hr as u32
            ));
            return None;
        }
        Some(())
    }

    /// Total number of virtual desktops.
    unsafe fn count(&self) -> Option<usize> {
        let arr = self.get_desktops()?;
        Some(arr_count(arr.ptr()) as usize)
    }

    /// The real current 1-based desktop index.
    ///
    /// This is the Rust equivalent of AHK's `GetCurrentVirtualDesktopIdxFromAPI`:
    /// `GetCurrentDesktop()` gives an opaque `IVirtualDesktop`, which only
    /// becomes a *position* by locating it inside the `GetDesktops()` array.
    unsafe fn current_index(&self) -> Option<usize> {
        let current = self.get_current_desktop()?;
        let arr = self.get_desktops()?;
        let iid = self.1.desktop_iid();
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
                    return Some(i as usize + 1);
                }
            }
        }
        log(&format!(
            "[vd_api] current_index: no match among {} desktops",
            count
        ));
        None
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

/// After a desktop switch, activate the first (`dir > 0`) or last (`dir < 0`)
/// app window on the new desktop. The shell un-cloaks the target desktop's
/// windows asynchronously, so poll briefly until the enumeration sees them;
/// an empty desktop just times out and leaves the desktop background focused.
fn focus_edge_window(dir: i32) {
    use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
    for _ in 0..25 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let windows = crate::output::get_app_windows();
        let target = if dir > 0 {
            windows.first()
        } else {
            windows.last()
        };
        if let Some(&w) = target {
            unsafe {
                let _ = SetForegroundWindow(w);
            }
            return;
        }
    }
}

// ── Public API (safe to call from the hook callback) ─────────────────────────
//
// None of these touch COM; they post to the worker and return immediately.

/// Switch to the given 1-based desktop index (COM first, hotkey fallback).
pub fn switch_desktop(idx: usize) {
    post(Req::Switch(idx));
}

/// Step one desktop in `dir` direction (+1 or -1), clamped to the real range.
pub fn step_desktop(dir: i32) {
    post(Req::Step(dir));
}

/// Step one desktop in `dir`, wrapping last → first (and first → last), then
/// focus the first (+1) / last (-1) app window on the desktop we land on.
pub fn step_desktop_and_focus(dir: i32) {
    post(Req::StepFocus(dir));
}

/// Move the window `hwnd` to the 1-based desktop index and switch to it.
pub fn move_window_to_desktop(hwnd: isize, idx: usize) {
    post(Req::MoveWindow(hwnd, idx));
}

// ── Diagnostics (`clx vd-test`; runs on the main thread, no hook installed) ──

/// Read-only diagnostic dump: manager version, desktop count, current index and
/// every desktop GUID. Written to `%TEMP%\capslockx_vd.log`.
pub fn dump() {
    co_init_sta();
    let Some(mgr) = acquire_fresh() else {
        log("[vd_api] dump: no manager (COM interface unavailable)");
        return;
    };
    unsafe {
        let ver = mgr.1.name();
        let cur_idx = mgr.current_index();
        let cur_id = mgr.get_current_desktop().and_then(|c| desktop_id(c.ptr()));
        let Some(arr) = mgr.get_desktops() else {
            log(&format!("[vd_api] dump: ver={} get_desktops FAILED", ver));
            return;
        };
        let iid = mgr.1.desktop_iid();
        let count = arr_count(arr.ptr());
        log(&format!(
            "[vd_api] dump: ver={} current={:?} count={} views={}",
            ver,
            cur_idx,
            count,
            mgr.2.is_some()
        ));
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
