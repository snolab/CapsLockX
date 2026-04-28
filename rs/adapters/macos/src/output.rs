//! MacPlatform – implements the `Platform` trait using Core Graphics.
//!
//! Uses `CGEventPost` to inject keyboard and mouse events.
//! Tags injected events with a user-data field so the hook can skip them.

use core_graphics::display::CGDisplay;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton,
    EventField, ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

use capslockx_core::{KeyCode, Platform};
use capslockx_core::platform::{ArrangeMode, MouseButton};

use std::sync::Mutex;
use std::time::Instant;

use crate::key_map::keycode_to_cg_keycode;

// ── Window cycle state ──────────────────────────────────────────────────────
// Snapshot the pid list on first press, then walk through it on subsequent
// presses.  Resets after 1 second of inactivity so a fresh press gets a
// fresh snapshot.

/// Cached window order settings — updated by prefs hot-reload,
/// read by the AccModel ticker thread without touching the filesystem.
static CYCLE_ORDER: Mutex<Option<String>> = Mutex::new(None);
static ARRANGE_ORDER: Mutex<Option<String>> = Mutex::new(None);

// ── Background window-list cache ─────────────────────────────────────────────
// list_all_windows() takes ~1.5s because macOS serializes per-app AX IPC.
// We pre-warm by refreshing every 500ms in a background thread so that
// Space+Z / Space+C reads from cache and feels instant.

struct WindowListCache {
    windows: Vec<WindowEntry>,
    at: Instant,
}
static WINDOW_LIST_CACHE: Mutex<Option<WindowListCache>> = Mutex::new(None);
static WINDOW_LIST_REFRESHING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Start an async refresh of the window-list cache (no-op if one is already running).
fn refresh_window_cache_async() {
    use std::sync::atomic::Ordering;
    if WINDOW_LIST_REFRESHING.swap(true, Ordering::SeqCst) { return; }
    std::thread::spawn(|| {
        let windows = list_all_windows();
        *WINDOW_LIST_CACHE.lock().unwrap() = Some(WindowListCache { windows, at: Instant::now() });
        WINDOW_LIST_REFRESHING.store(false, std::sync::atomic::Ordering::SeqCst);
    });
}

/// Start the background polling loop that keeps the cache warm.
/// Call once at startup from MacPlatform::new().
pub fn start_window_cache_warmer() {
    // Kick off the first refresh immediately.
    refresh_window_cache_async();
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            refresh_window_cache_async();
        }
    });
}

/// Stale-while-revalidate: always return from cache instantly, kick background refresh.
/// Only blocks synchronously on the very first call (before any cache exists).
fn list_all_windows_cached() -> Vec<WindowEntry> {
    // Always trigger a background refresh (no-op if one is already running).
    refresh_window_cache_async();

    let cached = {
        let guard = WINDOW_LIST_CACHE.lock().unwrap();
        guard.as_ref().map(|c| c.windows.clone())
    };

    if let Some(windows) = cached {
        // Return immediately — may be slightly stale, background refresh is running.
        windows
    } else {
        // First-ever call (startup): must wait once. After this the cache is always warm.
        let windows = list_all_windows();
        *WINDOW_LIST_CACHE.lock().unwrap() = Some(WindowListCache { windows: windows.clone(), at: Instant::now() });
        windows
    }
}

fn get_cycle_order() -> String {
    CYCLE_ORDER.lock().unwrap().clone().unwrap_or_else(|| "y,x".to_string())
}

pub fn set_cycle_order(order: &str) {
    *CYCLE_ORDER.lock().unwrap() = Some(order.to_string());
}

fn get_arrange_order() -> String {
    ARRANGE_ORDER.lock().unwrap().clone().unwrap_or_else(|| "y,x".to_string())
}

pub fn set_arrange_order(order: &str) {
    *ARRANGE_ORDER.lock().unwrap() = Some(order.to_string());
}

struct CycleState {
    windows: Vec<WindowEntry>,
    index: usize,
    last_use: Instant,
    last_activated_wid: u32, // the window_id we last activated
}

static CYCLE: Mutex<Option<CycleState>> = Mutex::new(None);

// Raw FFI bindings not exposed by the core-graphics crate.
extern "C" {
    fn CGEventSourceKeyState(
        source_state_id: core_graphics::event_source::CGEventSourceStateID,
        key: core_graphics::event::CGKeyCode,
    ) -> bool;
}

// ── Window-cycling FFI ──────────────────────────────────────────────────────

use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::CFString;

type CFArrayRef = *const std::ffi::c_void;

extern "C" {
    fn CGWindowListCopyWindowInfo(
        option: u32,
        relative_to: u32,
    ) -> CFArrayRef;
}

// Objective-C runtime FFI for NSRunningApplication.
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn objc_msgSend(receiver: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, ...) -> *mut std::ffi::c_void;
}

// ── Accessibility API FFI for per-window cycling ────────────────────────────

type AXUIElementRef = *mut std::ffi::c_void;
type CFStringRefRaw = *const std::ffi::c_void;

extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRefRaw,
        value: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn AXUIElementPerformAction(
        element: AXUIElementRef,
        action: CFStringRefRaw,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRefRaw,
        value: *const std::ffi::c_void,
    ) -> i32;
    fn AXValueCreate(value_type: i32, value: *const std::ffi::c_void) -> *mut std::ffi::c_void;
}

/// A single window: app pid + CGWindowID for stable tracking.
#[derive(Clone)]
struct WindowEntry {
    pid: i64,
    window_id: u32, // CGWindowID — stable across z-order changes
    window_index: usize,
    title: String,
    display_id: u32, // CGDirectDisplayID of the screen this window is on
    cx: f64,         // center X in global screen coordinates
    cy: f64,         // center Y
}

/// Bucket size for position-based window ordering (pixels).
/// Windows within this many pixels of each other on the bucket axis
/// are considered to be on the same "row" or "column".
const POSITION_BUCKET_PX: f64 = 150.0;

/// Sort windows by the configured cycle order.
fn sort_windows_by_order(windows: &mut Vec<WindowEntry>, order: &str) {
    match order {
        "column" => {
            // x-bucket, then y — column-major (default)
            windows.sort_by(|a, b| {
                let ax = (a.cx / POSITION_BUCKET_PX) as i64;
                let bx = (b.cx / POSITION_BUCKET_PX) as i64;
                ax.cmp(&bx).then_with(|| a.cy.partial_cmp(&b.cy).unwrap_or(std::cmp::Ordering::Equal))
            });
        }
        "row" => {
            // y-bucket, then x — row-major (reading order)
            windows.sort_by(|a, b| {
                let ay = (a.cy / POSITION_BUCKET_PX) as i64;
                let by = (b.cy / POSITION_BUCKET_PX) as i64;
                ay.cmp(&by).then_with(|| a.cx.partial_cmp(&b.cx).unwrap_or(std::cmp::Ordering::Equal))
            });
        }
        "diagonal" => {
            // x+y, then x — diagonal sweep
            windows.sort_by(|a, b| {
                let da = ((a.cx + a.cy) * 100.0) as i64;
                let db = ((b.cx + b.cy) * 100.0) as i64;
                da.cmp(&db).then_with(|| {
                    let xa = (a.cx * 100.0) as i64;
                    let xb = (b.cx * 100.0) as i64;
                    xa.cmp(&xb)
                })
            });
        }
        "linear" => {
            // x + 2*y — weighted linear combination
            windows.sort_by(|a, b| {
                let va = ((a.cx + 2.0 * a.cy) * 100.0) as i64;
                let vb = ((b.cx + 2.0 * b.cy) * 100.0) as i64;
                va.cmp(&vb)
            });
        }
        "x,y" => {
            // Pure x then y — no bucketing
            windows.sort_by(|a, b| {
                a.cx.partial_cmp(&b.cx).unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cy.partial_cmp(&b.cy).unwrap_or(std::cmp::Ordering::Equal))
            });
        }
        "y,x" => {
            // Pure y then x — no bucketing
            windows.sort_by(|a, b| {
                a.cy.partial_cmp(&b.cy).unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cx.partial_cmp(&b.cx).unwrap_or(std::cmp::Ordering::Equal))
            });
        }
        _ => {
            // "id" — legacy: display, pid, window_id
            windows.sort_by_key(|w| (w.display_id, w.pid, w.window_id));
        }
    }
}

/// Get the CGWindowID of the frontmost (topmost layer-0) window.
/// Returns 0 if unable to determine.
fn frontmost_window_id() -> u32 {
    unsafe {
        extern "C" {
            fn CFArrayGetCount(arr: CFArrayRef) -> isize;
            fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            fn CFDictionaryGetValue(dict: *const std::ffi::c_void, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
            fn CFNumberGetValue(number: *const std::ffi::c_void, the_type: i32, value_ptr: *mut std::ffi::c_void) -> bool;
        }

        let on_screen_opts: u32 = (1 << 0) | (1 << 4); // kCGWindowListOptionOnScreenOnly | ExcludeDesktopElements
        let list_ref = CGWindowListCopyWindowInfo(on_screen_opts, 0);
        if list_ref.is_null() { return 0; }

        let count = CFArrayGetCount(list_ref);
        let key_layer = CFString::new("kCGWindowLayer");
        let key_wid = CFString::new("kCGWindowNumber");

        let mut result: u32 = 0;
        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list_ref, i);
            if dict.is_null() { continue; }
            let layer_ref = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
            if layer_ref.is_null() { continue; }
            let mut layer: i64 = 0;
            CFNumberGetValue(layer_ref, 4, &mut layer as *mut _ as *mut _);
            if layer == 0 {
                let wid_ref = CFDictionaryGetValue(dict, key_wid.as_concrete_TypeRef() as *const _);
                if !wid_ref.is_null() {
                    let mut wid: i64 = 0;
                    CFNumberGetValue(wid_ref, 4, &mut wid as *mut _ as *mut _);
                    result = wid as u32;
                }
                break; // First layer-0 window = frontmost
            }
        }
        CFRelease(list_ref);
        result
    }
}

/// Get the AXWindows array for an app, return (element_ref, array_ref, count).
/// Caller must CFRelease both refs when done.
unsafe fn ax_windows_for_pid(pid: i32) -> Option<(AXUIElementRef, *mut std::ffi::c_void, usize)> {
    let app_ref = AXUIElementCreateApplication(pid);
    if app_ref.is_null() { return None; }

    let attr = CFString::new("AXWindows");
    let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
    let err = AXUIElementCopyAttributeValue(
        app_ref,
        attr.as_concrete_TypeRef() as CFStringRefRaw,
        &mut value,
    );
    if err != 0 || value.is_null() {
        CFRelease(app_ref as *const _);
        return None;
    }

    extern "C" { fn CFArrayGetCount(arr: CFArrayRef) -> isize; }
    let count = CFArrayGetCount(value as CFArrayRef) as usize;
    Some((app_ref, value, count))
}

/// Get the title of an AX window element.
unsafe fn ax_window_title(win: *mut std::ffi::c_void) -> String {
    let attr = CFString::new("AXTitle");
    let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
    let err = AXUIElementCopyAttributeValue(
        win,
        attr.as_concrete_TypeRef() as CFStringRefRaw,
        &mut value,
    );
    if err != 0 || value.is_null() { return String::new(); }

    let sel_utf8 = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
    let cstr: *const std::ffi::c_char = {
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char
            = std::mem::transmute(objc_msgSend as *const ());
        f(value, sel_utf8)
    };
    let title = if !cstr.is_null() {
        std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
    } else {
        String::new()
    };
    CFRelease(value as *const _);
    title
}

/// Determine which display contains the given point (center of a window).
/// Returns the CGDirectDisplayID, falling back to the main display.
fn display_for_point(cx: f64, cy: f64) -> u32 {
    let displays = CGDisplay::active_displays().unwrap_or_default();
    for id in &displays {
        let r = CGDisplay::new(*id).bounds();
        if cx >= r.origin.x && cx < r.origin.x + r.size.width
            && cy >= r.origin.y && cy < r.origin.y + r.size.height
        {
            return *id;
        }
    }
    CGDisplay::main().id
}

/// Build a map from CGWindowID → (center_x, center_y) using CGWindowList bounds.
fn build_window_bounds_map() -> std::collections::HashMap<u32, (f64, f64)> {
    unsafe {
        extern "C" {
            fn CFArrayGetCount(arr: CFArrayRef) -> isize;
            fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            fn CFDictionaryGetValue(dict: *const std::ffi::c_void, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
            fn CFNumberGetValue(number: *const std::ffi::c_void, the_type: i32, value_ptr: *mut std::ffi::c_void) -> bool;
            fn CGRectMakeWithDictionaryRepresentation(dict: *const std::ffi::c_void, rect: *mut [f64; 4]) -> bool;
        }

        let mut map = std::collections::HashMap::new();
        let opts: u32 = (1 << 0) | (1 << 4); // OnScreenOnly | ExcludeDesktopElements
        let list_ref = CGWindowListCopyWindowInfo(opts, 0);
        if list_ref.is_null() { return map; }

        let count = CFArrayGetCount(list_ref);
        let key_wid = CFString::new("kCGWindowNumber");
        let key_bounds = CFString::new("kCGWindowBounds");
        let key_layer = CFString::new("kCGWindowLayer");

        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list_ref, i);
            if dict.is_null() { continue; }
            let layer_ref = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
            if layer_ref.is_null() { continue; }
            let mut layer: i64 = 0;
            CFNumberGetValue(layer_ref, 4, &mut layer as *mut _ as *mut _);
            if layer != 0 { continue; }

            let wid_ref = CFDictionaryGetValue(dict, key_wid.as_concrete_TypeRef() as *const _);
            if wid_ref.is_null() { continue; }
            let mut wid: i64 = 0;
            CFNumberGetValue(wid_ref, 4, &mut wid as *mut _ as *mut _);

            let bounds_ref = CFDictionaryGetValue(dict, key_bounds.as_concrete_TypeRef() as *const _);
            if bounds_ref.is_null() { continue; }
            let mut rect = [0.0f64; 4]; // x, y, w, h
            if CGRectMakeWithDictionaryRepresentation(bounds_ref, &mut rect) {
                let cx = rect[0] + rect[2] / 2.0;
                let cy = rect[1] + rect[3] / 2.0;
                map.insert(wid as u32, (cx, cy));
            }
        }
        CFRelease(list_ref);
        map
    }
}

/// Per-pid AX window collection. Called in parallel from list_all_windows.
/// Each call creates its own AX refs — safe for concurrent invocation across different pids.
unsafe fn collect_windows_for_pid(
    pid: i64,
    onscreen_wids: &std::collections::HashSet<u32>,
    bounds_map: &std::collections::HashMap<u32, (f64, f64)>,
    main_display: u32,
) -> Vec<WindowEntry> {
    extern "C" {
        fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
        fn _AXUIElementGetWindow(element: AXUIElementRef, wid: *mut u32) -> i32;
    }
    let mut entries = Vec::new();
    if let Some((app_ref, arr, count)) = ax_windows_for_pid(pid as i32) {
        for wi in 0..count {
            let win = CFArrayGetValueAtIndex(arr as CFArrayRef, wi as isize);
            if win.is_null() { continue; }
            let title = ax_window_title(win as *mut _);
            let mut wid: u32 = 0;
            _AXUIElementGetWindow(win as AXUIElementRef, &mut wid);
            if title.is_empty() { continue; }
            // Skip minimized windows — they still appear in the AX list
            // (and often in CGWindowList with IsOnscreen=true on the Space
            // where they were minimized) but cycling to them silently
            // un-minimizes something the user can't see.
            if ax_bool_attr(win as *mut _, "AXMinimized") { continue; }
            let on_current_space = wid != 0 && onscreen_wids.contains(&wid);
            if on_current_space {
                let (cx, cy) = bounds_map.get(&wid).copied().unwrap_or((0.0, 0.0));
                let display_id = if bounds_map.contains_key(&wid) {
                    display_for_point(cx, cy)
                } else { main_display };
                entries.push(WindowEntry { pid, window_id: wid, window_index: wi, title, display_id, cx, cy });
            }
        }
        CFRelease(arr as *const _);
        CFRelease(app_ref as *const _);
    }
    entries
}

/// Read a boolean AX attribute (e.g. AXMinimized, AXFocused). Returns false
/// on any error or when the attribute is absent — a conservative default
/// that keeps the window in the list rather than silently dropping it.
unsafe fn ax_bool_attr(win: *mut std::ffi::c_void, attr: &str) -> bool {
    extern "C" {
        fn CFBooleanGetValue(boolean: *const std::ffi::c_void) -> bool;
    }
    let name = CFString::new(attr);
    let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
    let rc = AXUIElementCopyAttributeValue(
        win as AXUIElementRef,
        name.as_concrete_TypeRef() as CFStringRefRaw,
        &mut value,
    );
    if rc != 0 || value.is_null() { return false; }
    let b = CFBooleanGetValue(value as *const _);
    CFRelease(value as *const _);
    b
}

/// List all individual windows across all GUI apps, in z-order (on-screen first)
/// then by app launch order for off-screen apps. Each window is a separate entry.
fn list_all_windows() -> Vec<WindowEntry> {
    let bounds_map = std::sync::Arc::new(build_window_bounds_map());
    let main_display = CGDisplay::main().id;
    unsafe {
        let mut seen_pids = std::collections::HashSet::new();

        extern "C" {
            fn CFArrayGetCount(arr: CFArrayRef) -> isize;
            fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            fn CFDictionaryGetValue(dict: *const std::ffi::c_void, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
            fn CFNumberGetValue(number: *const std::ffi::c_void, the_type: i32, value_ptr: *mut std::ffi::c_void) -> bool;
        }

        // 1. On-screen windows in z-order with CGWindowID
        let on_screen_opts: u32 = (1 << 0) | (1 << 4);
        let list_ref = CGWindowListCopyWindowInfo(on_screen_opts, 0);
        let mut ordered_pids: Vec<i64> = Vec::new();
        // Set of all on-screen window IDs (including those without titles).
        let mut onscreen_wids: std::collections::HashSet<u32> = std::collections::HashSet::new();
        if !list_ref.is_null() {
            let count = CFArrayGetCount(list_ref);
            let key_pid = CFString::new("kCGWindowOwnerPID");
            let key_layer = CFString::new("kCGWindowLayer");
            let key_wid = CFString::new("kCGWindowNumber");
            let key_onscreen = CFString::new("kCGWindowIsOnscreen");
            let mut pid_seen = std::collections::HashSet::new();
            for i in 0..count {
                let dict = CFArrayGetValueAtIndex(list_ref, i);
                if dict.is_null() { continue; }
                let pid_ref = CFDictionaryGetValue(dict, key_pid.as_concrete_TypeRef() as *const _);
                let layer_ref = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
                let wid_ref = CFDictionaryGetValue(dict, key_wid.as_concrete_TypeRef() as *const _);
                if pid_ref.is_null() || layer_ref.is_null() { continue; }
                let mut pid: i64 = 0;
                let mut layer: i64 = 0;
                let mut wid: i64 = 0;
                CFNumberGetValue(pid_ref, 4, &mut pid as *mut _ as *mut _);
                CFNumberGetValue(layer_ref, 4, &mut layer as *mut _ as *mut _);
                if !wid_ref.is_null() { CFNumberGetValue(wid_ref, 4, &mut wid as *mut _ as *mut _); }

                if layer == 0 {
                    // Skip windows not on the current Space (kCGWindowIsOnscreen).
                    extern "C" {
                        fn CFBooleanGetValue(boolean: *const std::ffi::c_void) -> bool;
                    }
                    let onscreen_ref = CFDictionaryGetValue(dict, key_onscreen.as_concrete_TypeRef() as *const _);
                    let is_onscreen = if !onscreen_ref.is_null() {
                        CFBooleanGetValue(onscreen_ref)
                    } else {
                        true // absent = assume on-screen
                    };
                    if !is_onscreen { continue; }

                    // Track all on-screen wids (even without title) for Space filtering.
                    if wid != 0 { onscreen_wids.insert(wid as u32); }

                    if pid_seen.insert(pid) {
                        ordered_pids.push(pid);
                    }
                }
            }
            CFRelease(list_ref);
        }

        // 2. Add off-screen GUI apps from NSWorkspace
        let ws_cls = objc_getClass(b"NSWorkspace\0".as_ptr() as *const _);
        if !ws_cls.is_null() {
            let sel_shared = sel_registerName(b"sharedWorkspace\0".as_ptr() as *const _);
            let sel_apps = sel_registerName(b"runningApplications\0".as_ptr() as *const _);
            let sel_policy = sel_registerName(b"activationPolicy\0".as_ptr() as *const _);
            let sel_pid = sel_registerName(b"processIdentifier\0".as_ptr() as *const _);
            let sel_count = sel_registerName(b"count\0".as_ptr() as *const _);
            let sel_obj_at = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);

            let ws: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(ws_cls, sel_shared)
            };
            if !ws.is_null() {
                let apps_arr: *mut std::ffi::c_void = {
                    let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                        = std::mem::transmute(objc_msgSend as *const ());
                    f(ws, sel_apps)
                };
                if !apps_arr.is_null() {
                    let n: usize = {
                        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> usize
                            = std::mem::transmute(objc_msgSend as *const ());
                        f(apps_arr, sel_count)
                    };
                    let mut extra_pids = std::collections::HashSet::new();
                    for idx in 0..n {
                        let app: *mut std::ffi::c_void = {
                            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, usize) -> *mut std::ffi::c_void
                                = std::mem::transmute(objc_msgSend as *const ());
                            f(apps_arr, sel_obj_at, idx)
                        };
                        if app.is_null() { continue; }
                        let policy: i64 = {
                            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> i64
                                = std::mem::transmute(objc_msgSend as *const ());
                            f(app, sel_policy)
                        };
                        if policy != 0 { continue; }
                        let pid: i32 = {
                            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> i32
                                = std::mem::transmute(objc_msgSend as *const ());
                            f(app, sel_pid)
                        };
                        if !ordered_pids.contains(&(pid as i64)) {
                            extra_pids.insert(pid as i64);
                        }
                    }
                    for pid in extra_pids {
                        ordered_pids.push(pid);
                    }
                }
            }
        }

        // 3. Enumerate AX windows per-pid IN PARALLEL.
        // Each app's AX request is an independent IPC call — concurrent calls to
        // different apps don't interfere. Sequential queries take ~130ms × N apps;
        // parallel reduces that to ~max(single app latency) ≈ 130-200ms.
        let onscreen_wids = std::sync::Arc::new(onscreen_wids);
        let mut handles: Vec<(usize, std::thread::JoinHandle<Vec<WindowEntry>>)> = Vec::new();
        for (order, &pid) in ordered_pids.iter().enumerate() {
            if !seen_pids.insert(pid) { continue; }
            let ow = std::sync::Arc::clone(&onscreen_wids);
            let bm = std::sync::Arc::clone(&bounds_map);
            handles.push((order, std::thread::spawn(move || {
                unsafe { collect_windows_for_pid(pid, &ow, &bm, main_display) }
            })));
        }

        // Collect results preserving z-order (ordered_pids order).
        let mut ordered: Vec<(usize, Vec<WindowEntry>)> = handles.into_iter()
            .filter_map(|(order, h)| h.join().ok().map(|e| (order, e)))
            .collect();
        ordered.sort_by_key(|(i, _)| *i);
        let entries: Vec<WindowEntry> = ordered.into_iter()
            .flat_map(|(_, e)| e)
            .collect();

        // Keep z-order from CGWindowList (front-to-back, most recently used first).
        // Don't sort — natural order is what Alt+Tab uses.

        // Dedup phantom tab entries: Chrome (and other tabbed apps) expose
        // every tab as a separate AX window with the same screen position.
        // Cycling onto a non-foreground tab silently focuses nothing visible
        // — the user perceives it as "Space+Z went to an invisible window".
        // Keep only the first (topmost in z-order) entry per (pid, cx, cy).
        let mut seen_positions = std::collections::HashSet::<(i64, i64, i64)>::new();
        let entries: Vec<WindowEntry> = entries
            .into_iter()
            .filter(|w| {
                // Windows whose wid wasn't in bounds_map get (0.0, 0.0). Don't
                // collapse those — we have no positional signal to dedup by.
                if w.cx == 0.0 && w.cy == 0.0 { return true; }
                // Round to whole pixels — float NaN/precision noise breaks Hash.
                let key = (w.pid, w.cx.round() as i64, w.cy.round() as i64);
                seen_positions.insert(key)
            })
            .collect();

        entries
    }
}

/// Set an AX window's position and size using the Accessibility API.
/// Creates AXValue objects for CGPoint (position) and CGSize (size) and sets
/// the AXPosition and AXSize attributes on the window element.
unsafe fn ax_set_window_frame(win: AXUIElementRef, x: f64, y: f64, w: f64, h: f64) {
    // kAXValueCGPointType = 1, kAXValueCGSizeType = 2
    #[repr(C)]
    struct CGSizeRaw { width: f64, height: f64 }

    let point = CGPoint::new(x, y);
    let size = CGSizeRaw { width: w, height: h };

    let ax_pos = AXValueCreate(1, &point as *const _ as *const std::ffi::c_void);
    if !ax_pos.is_null() {
        let attr = CFString::new("AXPosition");
        AXUIElementSetAttributeValue(
            win,
            attr.as_concrete_TypeRef() as CFStringRefRaw,
            ax_pos as *const _,
        );
        CFRelease(ax_pos as *const _);
    }

    let ax_size = AXValueCreate(2, &size as *const _ as *const std::ffi::c_void);
    if !ax_size.is_null() {
        let attr = CFString::new("AXSize");
        AXUIElementSetAttributeValue(
            win,
            attr.as_concrete_TypeRef() as CFStringRefRaw,
            ax_size as *const _,
        );
        CFRelease(ax_size as *const _);
    }
}

/// Find the AXUIElement in an app's window array matching `entry`.
/// Tries window_id first, then title, then index. Returns raw pointer (not retained).
unsafe fn find_ax_window(arr: CFArrayRef, count: usize, entry: &WindowEntry) -> *const std::ffi::c_void {
    extern "C" {
        fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
        fn _AXUIElementGetWindow(element: AXUIElementRef, wid: *mut u32) -> i32;
    }
    // Primary: match by CGWindowID.
    if entry.window_id != 0 {
        for wi in 0..count {
            let win = CFArrayGetValueAtIndex(arr, wi as isize);
            if win.is_null() { continue; }
            let mut wid: u32 = 0;
            _AXUIElementGetWindow(win as AXUIElementRef, &mut wid);
            if wid == entry.window_id { return win; }
        }
    }
    // Fallback: match by title.
    if !entry.title.is_empty() {
        for wi in 0..count {
            let win = CFArrayGetValueAtIndex(arr, wi as isize);
            if win.is_null() { continue; }
            if ax_window_title(win as *mut _) == entry.title { return win; }
        }
    }
    // Last resort: by index.
    if entry.window_index < count {
        return CFArrayGetValueAtIndex(arr, entry.window_index as isize);
    }
    std::ptr::null()
}

/// Collect AXUIElementRef handles for all titled windows on the given display,
/// sorted by (pid, window_id) for stable ordering (same order as cycle_windows).
/// The returned refs are retained — caller must CFRelease each one.
/// Takes a pre-built window list to avoid a redundant list_all_windows() call.
fn get_all_ax_window_refs_stable(display_id: u32, all_windows: &[WindowEntry]) -> Vec<AXUIElementRef> {
    unsafe {
        extern "C" {
            fn CFRetain(cf: *const std::ffi::c_void) -> *const std::ffi::c_void;
        }

        let mut entries: Vec<WindowEntry> = all_windows.iter()
            .filter(|w| w.display_id == display_id)
            .cloned()
            .collect();
        sort_windows_by_order(&mut entries, &get_arrange_order());

        let mut refs: Vec<AXUIElementRef> = Vec::new();

        for entry in &entries {
            if let Some((app_ref, arr, count)) = ax_windows_for_pid(entry.pid as i32) {
                let win = find_ax_window(arr as CFArrayRef, count, entry);
                if !win.is_null() {
                    CFRetain(win);
                    refs.push(win as AXUIElementRef);
                }
                CFRelease(arr as *const _);
                CFRelease(app_ref as *const _);
            }
        }

        refs
    }
}

/// Activate a specific window: bring the app to front, then raise the specific window.
fn activate_window(entry: &WindowEntry) {
    // No eprintln here — called from CGEventTap callback; blocking stderr deadlocks the event tap.
    unsafe {
        // 1. Activate the app
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr() as *const _);
        if cls.is_null() { return; }
        let sel_running = sel_registerName(
            b"runningApplicationWithProcessIdentifier:\0".as_ptr() as *const _,
        );
        let app: *mut std::ffi::c_void = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, i32) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls, sel_running, entry.pid as i32)
        };
        if !app.is_null() {
            let sel_activate = sel_registerName(
                b"activateWithOptions:\0".as_ptr() as *const _,
            );
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, u64) -> bool
                = std::mem::transmute(objc_msgSend as *const ());
            // NSApplicationActivateAllWindows (1) | NSApplicationActivateIgnoringOtherApps (2) = 3
            f(app, sel_activate, 3);
        }

        // (no eprintln in CGEventTap callback path)
        // 2. Raise the specific window via AXUIElement — match by window_id (CGWindowID).
        if let Some((app_ref, arr, count)) = ax_windows_for_pid(entry.pid as i32) {
            let target = find_ax_window(arr as CFArrayRef, count, entry);
            if !target.is_null() {
                let cf_true: *const std::ffi::c_void = {
                    extern "C" { #[allow(non_upper_case_globals)]
                    static kCFBooleanTrue: *const std::ffi::c_void; }
                    kCFBooleanTrue
                };

                // Set AXMain FIRST — tells the app which window should be main.
                let attr_main = CFString::new("AXMain");
                AXUIElementSetAttributeValue(
                    target as AXUIElementRef,
                    attr_main.as_concrete_TypeRef() as CFStringRefRaw,
                    cf_true,
                );

                // AXRaise brings the window to front within the app.
                let action = CFString::new("AXRaise");
                AXUIElementPerformAction(
                    target as AXUIElementRef,
                    action.as_concrete_TypeRef() as CFStringRefRaw,
                );

                // Set AXFocused to give keyboard focus.
                let attr_focused = CFString::new("AXFocused");
                AXUIElementSetAttributeValue(
                    target as AXUIElementRef,
                    attr_focused.as_concrete_TypeRef() as CFStringRefRaw,
                    cf_true,
                );

                // Re-activate the app AFTER raising — this forces macOS to
                // redraw the window stack with our target on top.
                if !app.is_null() {
                    let sel_act2 = sel_registerName(
                        b"activateWithOptions:\0".as_ptr() as *const _,
                    );
                    let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, u64) -> bool
                        = std::mem::transmute(objc_msgSend as *const ());
                    f(app, sel_act2, 3);
                }
            }
            CFRelease(arr as *const _);
            CFRelease(app_ref as *const _);
        }
    }
}

/// Tag value written to EVENT_SOURCE_USER_DATA on injected events so the
/// hook callback can recognise and pass them through.
pub const SELF_INJECT_TAG: i64 = 0x434C5831; // "CLX1"

/// Compute the bounding rectangle (union) of all active displays.
/// Returns `(min_x, min_y, max_x, max_y)` in global (Quartz) coordinates.
/// Falls back to the main display bounds if enumeration fails.
fn screen_bounds() -> (f64, f64, f64, f64) {
    let displays = CGDisplay::active_displays().unwrap_or_default();
    if displays.is_empty() {
        let main = CGDisplay::main().bounds();
        return (
            main.origin.x,
            main.origin.y,
            main.origin.x + main.size.width,
            main.origin.y + main.size.height,
        );
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for id in displays {
        let r = CGDisplay::new(id).bounds();
        min_x = min_x.min(r.origin.x);
        min_y = min_y.min(r.origin.y);
        max_x = max_x.max(r.origin.x + r.size.width);
        max_y = max_y.max(r.origin.y + r.size.height);
    }
    (min_x, min_y, max_x, max_y)
}

/// Get the visible work area (excluding menu bar and Dock) in Quartz coordinates.
/// Returns `(x, y, width, height)`.
/// Uses NSScreen.mainScreen.visibleFrame, converted from AppKit (y-up) to Quartz (y-down).
fn visible_work_area() -> (f64, f64, f64, f64) {
    unsafe {
        let cls = objc_getClass(b"NSScreen\0".as_ptr() as *const _);
        if cls.is_null() {
            // Fallback to full screen bounds
            let (min_x, min_y, max_x, max_y) = screen_bounds();
            return (min_x, min_y, max_x - min_x, max_y - min_y);
        }
        let sel_main = sel_registerName(b"mainScreen\0".as_ptr() as *const _);
        let screen: *mut std::ffi::c_void = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls, sel_main)
        };
        if screen.is_null() {
            let (min_x, min_y, max_x, max_y) = screen_bounds();
            return (min_x, min_y, max_x - min_x, max_y - min_y);
        }

        // NSScreen.frame (full screen in AppKit coords, y-up from bottom-left)
        let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct NSRect { x: f64, y: f64, w: f64, h: f64 }

        // On ARM64 macOS, NSRect return uses normal struct return convention
        #[cfg(target_arch = "aarch64")]
        let full: NSRect = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                = std::mem::transmute(objc_msgSend as *const ());
            f(screen, sel_frame)
        };
        #[cfg(target_arch = "x86_64")]
        let full: NSRect = {
            extern "C" {
                fn objc_msgSend_stret(ret: *mut NSRect, receiver: *mut std::ffi::c_void, sel: *mut std::ffi::c_void);
            }
            let mut r = NSRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            objc_msgSend_stret(&mut r, screen, sel_frame);
            r
        };

        // NSScreen.visibleFrame (excluding menu bar & Dock, AppKit coords)
        let sel_visible = sel_registerName(b"visibleFrame\0".as_ptr() as *const _);
        #[cfg(target_arch = "aarch64")]
        let vis: NSRect = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                = std::mem::transmute(objc_msgSend as *const ());
            f(screen, sel_visible)
        };
        #[cfg(target_arch = "x86_64")]
        let vis: NSRect = {
            let mut r = NSRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            objc_msgSend_stret(&mut r, screen, sel_visible);
            r
        };

        // Convert AppKit coords (y-up, origin at bottom-left) to Quartz (y-down, origin at top-left).
        // Quartz y = full_height - appkit_y - rect_height
        let quartz_x = vis.x;
        let quartz_y = (full.y + full.h) - vis.y - vis.h;

        // eprintln!("[CLX] work area: x={} y={} w={} h={} (screen: {}x{})",
        //     quartz_x, quartz_y, vis.w, vis.h, full.w, full.h);

        (quartz_x, quartz_y, vis.w, vis.h)
    }
}

/// Get the visible work area for a specific display (by CGDirectDisplayID).
/// Iterates NSScreen.screens to find the matching screen, falls back to mainScreen.
fn visible_work_area_for_display(target_display_id: u32) -> (f64, f64, f64, f64) {
    unsafe {
        let cls = objc_getClass(b"NSScreen\0".as_ptr() as *const _);
        if cls.is_null() { return visible_work_area(); }

        let sel_screens = sel_registerName(b"screens\0".as_ptr() as *const _);
        let screens_arr: *mut std::ffi::c_void = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls, sel_screens)
        };
        if screens_arr.is_null() { return visible_work_area(); }

        let sel_count = sel_registerName(b"count\0".as_ptr() as *const _);
        let sel_obj_at = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const _);
        let sel_desc = sel_registerName(b"deviceDescription\0".as_ptr() as *const _);
        let sel_obj_for_key = sel_registerName(b"objectForKey:\0".as_ptr() as *const _);
        let sel_uint = sel_registerName(b"unsignedIntValue\0".as_ptr() as *const _);

        let n: usize = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> usize
                = std::mem::transmute(objc_msgSend as *const ());
            f(screens_arr, sel_count)
        };

        let key_screen_number = CFString::new("NSScreenNumber");

        for idx in 0..n {
            let screen: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, usize) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(screens_arr, sel_obj_at, idx)
            };
            if screen.is_null() { continue; }

            // Get deviceDescription dictionary
            let desc: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(screen, sel_desc)
            };
            if desc.is_null() { continue; }

            // Get NSScreenNumber from deviceDescription
            let num_obj: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_void) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(desc, sel_obj_for_key, key_screen_number.as_concrete_TypeRef() as *const _)
            };
            if num_obj.is_null() { continue; }

            let display_id: u32 = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> u32
                    = std::mem::transmute(objc_msgSend as *const ());
                f(num_obj, sel_uint)
            };

            if display_id != target_display_id { continue; }

            // Found the matching screen — get its visibleFrame and frame.
            let sel_frame = sel_registerName(b"frame\0".as_ptr() as *const _);
            #[repr(C)]
            #[derive(Clone, Copy)]
            struct NSRect { x: f64, y: f64, w: f64, h: f64 }

            #[cfg(target_arch = "aarch64")]
            let full: NSRect = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                    = std::mem::transmute(objc_msgSend as *const ());
                f(screen, sel_frame)
            };
            #[cfg(target_arch = "x86_64")]
            let full: NSRect = {
                extern "C" {
                    fn objc_msgSend_stret(ret: *mut NSRect, receiver: *mut std::ffi::c_void, sel: *mut std::ffi::c_void);
                }
                let mut r = NSRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
                objc_msgSend_stret(&mut r, screen, sel_frame);
                r
            };

            let sel_visible = sel_registerName(b"visibleFrame\0".as_ptr() as *const _);
            #[cfg(target_arch = "aarch64")]
            let vis: NSRect = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> NSRect
                    = std::mem::transmute(objc_msgSend as *const ());
                f(screen, sel_visible)
            };
            #[cfg(target_arch = "x86_64")]
            let vis: NSRect = {
                let mut r = NSRect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
                objc_msgSend_stret(&mut r, screen, sel_visible);
                r
            };

            let quartz_x = vis.x;
            let quartz_y = (full.y + full.h) - vis.y - vis.h;
            return (quartz_x, quartz_y, vis.w, vis.h);
        }

        // Fallback if display not found in NSScreen list.
        visible_work_area()
    }
}

// ── MacPlatform ──────────────────────────────────────────────────────────────

pub struct MacPlatform;

impl MacPlatform {
    pub fn new() -> Self {
        // Pre-spawn clx-prompt daemon so the Tauri WebView is warm before first Space+B.
        std::thread::spawn(|| crate::brainstorm_overlay::spawn_prompt_daemon());
        // Pre-warm window list cache so Space+Z / Space+C feel instant from first press.
        start_window_cache_warmer();
        Self
    }

    /// Create an event source for synthetic events.
    fn source() -> CGEventSource {
        CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .expect("failed to create CGEventSource")
    }

    /// Tag an event so our hook knows it's self-injected.
    fn tag(event: &CGEvent) {
        event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, SELF_INJECT_TAG);
    }

    /// Post a key tap (down+up) with explicit modifier flags on the event.
    /// On macOS, modifier flags must be embedded in the CGEvent itself —
    /// sending separate key_down(Shift) + key_tap(Tab) doesn't work reliably.
    ///
    /// After the tap, post a flag-reset event so any synthetic modifier flags
    /// (e.g. shift added for P=Shift+Tab even when user isn't holding shift)
    /// don't leak into the OS's persistent modifier state. Without this reset,
    /// subsequent user keystrokes can be incorrectly interpreted as Shift+key.
    fn tap_with_flags(cg_key: u16, flags: CGEventFlags) {
        let source = Self::source();
        if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, true) {
            event.set_flags(flags);
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
        let source = Self::source();
        if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, false) {
            event.set_flags(flags);
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
        // Reset modifier state to match the user's actual physical modifier
        // keys. We post one more key-up of the same key (a no-op since the key
        // is already up) carrying the real physical modifier flags. macOS uses
        // each event's flags to update its persistent modifier state, so this
        // clears any leaked synthetic modifiers.
        let real_flags = Self::physical_mod_flags();
        if real_flags != flags {
            let source = Self::source();
            if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, false) {
                event.set_flags(real_flags);
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    /// Compute CGEventFlags reflecting which modifier keys the user is
    /// physically holding right now (independent of any synthetic state).
    fn physical_mod_flags() -> CGEventFlags {
        let mut flags = CGEventFlags::CGEventFlagNull;
        unsafe {
            let s = CGEventSourceStateID::CombinedSessionState;
            // Shift: 0x38 left, 0x3C right
            if CGEventSourceKeyState(s, 0x38) || CGEventSourceKeyState(s, 0x3C) {
                flags = flags | CGEventFlags::CGEventFlagShift;
            }
            // Control: 0x3B left, 0x3E right
            if CGEventSourceKeyState(s, 0x3B) || CGEventSourceKeyState(s, 0x3E) {
                flags = flags | CGEventFlags::CGEventFlagControl;
            }
            // Option/Alt: 0x3A left, 0x3D right
            if CGEventSourceKeyState(s, 0x3A) || CGEventSourceKeyState(s, 0x3D) {
                flags = flags | CGEventFlags::CGEventFlagAlternate;
            }
            // Command: 0x37 left, 0x36 right
            if CGEventSourceKeyState(s, 0x37) || CGEventSourceKeyState(s, 0x36) {
                flags = flags | CGEventFlags::CGEventFlagCommand;
            }
        }
        flags
    }
}

impl Platform for MacPlatform {
    // ── Keyboard output ───────────────────────────────────────────────────────

    fn key_down(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            let source = Self::source();
            if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, true) {
                // Explicitly clear flags so residual modifier state from
                // tap_with_flags doesn't leak into plain key events.
                event.set_flags(CGEventFlags::CGEventFlagNull);
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    fn key_up(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            let source = Self::source();
            if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, false) {
                event.set_flags(CGEventFlags::CGEventFlagNull);
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    fn is_key_physically_down(&self, key: KeyCode) -> bool {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            unsafe {
                CGEventSourceKeyState(
                    CGEventSourceStateID::CombinedSessionState,
                    cg_key,
                )
            }
        } else {
            false
        }
    }

    /// Tap a key with multiple modifiers — all flags set on the CGEvent itself.
    fn key_tap_with_mods(&self, key: KeyCode, mods: &[KeyCode], n: i32) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            let mut flags = CGEventFlags::CGEventFlagNull;
            for m in mods {
                flags = flags | match m {
                    KeyCode::LShift | KeyCode::RShift | KeyCode::Shift
                        => CGEventFlags::CGEventFlagShift,
                    KeyCode::LCtrl | KeyCode::RCtrl
                        => CGEventFlags::CGEventFlagControl,
                    KeyCode::LAlt | KeyCode::RAlt
                        => CGEventFlags::CGEventFlagAlternate,
                    KeyCode::LWin | KeyCode::RWin
                        => CGEventFlags::CGEventFlagCommand,
                    _ => CGEventFlags::CGEventFlagNull,
                };
            }
            for _ in 0..n.clamp(0, 128) {
                Self::tap_with_flags(cg_key, flags);
            }
        }
    }

    /// Type a Unicode string by setting the string directly on CGEvents.
    fn type_text(&self, text: &str) {
        let source = Self::source();
        // Process text in chunks — CGEventKeyboardSetUnicodeString supports
        // up to ~20 UTF-16 code units per event reliably.
        for ch in text.chars() {
            let mut utf16_buf = [0u16; 2];
            let utf16 = ch.encode_utf16(&mut utf16_buf);
            let len = utf16.len();

            if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), 0, true) {
                unsafe {
                    extern "C" {
                        fn CGEventKeyboardSetUnicodeString(
                            event: *mut std::ffi::c_void,
                            len: u64,
                            str: *const u16,
                        );
                    }
                    use foreign_types::ForeignType;
                    CGEventKeyboardSetUnicodeString(
                        event.as_ptr() as *mut _,
                        len as u64,
                        utf16.as_ptr(),
                    );
                }
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
            // Key up
            if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), 0, false) {
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    /// Shift+key — set CGEventFlagShift on the event itself.
    fn key_tap_shifted(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagShift);
        }
    }

    /// Cmd+key on macOS (Command, not Control).
    fn key_tap_cmd_or_ctrl(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagCommand);
        }
    }

    /// Modifier+key repeated n times with flags on each event.
    fn key_tap_n_with_mod(&self, mod_key: KeyCode, key: KeyCode, n: i32) {
        let flags = match mod_key {
            KeyCode::LShift | KeyCode::RShift | KeyCode::Shift
                => CGEventFlags::CGEventFlagShift,
            KeyCode::LCtrl | KeyCode::RCtrl
                => CGEventFlags::CGEventFlagControl,
            KeyCode::LAlt | KeyCode::RAlt
                => CGEventFlags::CGEventFlagAlternate,
            KeyCode::LWin | KeyCode::RWin
                => CGEventFlags::CGEventFlagCommand,
            _ => CGEventFlags::CGEventFlagNull,
        };
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            for _ in 0..n.clamp(0, 128) {
                Self::tap_with_flags(cg_key, flags);
            }
        }
    }

    // ── Mouse output ──────────────────────────────────────────────────────────

    fn mouse_move(&self, dx: i32, dy: i32) {
        let source = Self::source();
        // Get current mouse position.
        if let Ok(dummy) = CGEvent::new(source.clone()) {
            let loc = dummy.location();
            let new_x = loc.x + dx as f64;
            let new_y = loc.y + dy as f64;

            // Clamp to union of all display bounds so the cursor can't leave the screen.
            let (min_x, min_y, max_x, max_y) = screen_bounds();
            let new_loc = CGPoint::new(
                new_x.clamp(min_x, (max_x - 1.0).max(min_x)),
                new_y.clamp(min_y, (max_y - 1.0).max(min_y)),
            );

            if let Ok(event) = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                new_loc,
                CGMouseButton::Left,
            ) {
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    fn scroll_v(&self, delta: i32) {
        let source = Self::source();
        if let Ok(event) = CGEvent::new_scroll_event(
            source,
            ScrollEventUnit::PIXEL,
            1,
            delta,
            0,
            0,
        ) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    fn scroll_h(&self, delta: i32) {
        let source = Self::source();
        if let Ok(event) = CGEvent::new_scroll_event(
            source,
            ScrollEventUnit::PIXEL,
            2,
            0,
            delta,
            0,
        ) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let source = Self::source();
        // Get current position.
        let loc = if let Ok(dummy) = CGEvent::new(source.clone()) {
            dummy.location()
        } else {
            CGPoint::new(0.0, 0.0)
        };

        let (event_type, cg_button) = match (button, pressed) {
            (MouseButton::Left, true)    => (CGEventType::LeftMouseDown,  CGMouseButton::Left),
            (MouseButton::Left, false)   => (CGEventType::LeftMouseUp,    CGMouseButton::Left),
            (MouseButton::Right, true)   => (CGEventType::RightMouseDown, CGMouseButton::Right),
            (MouseButton::Right, false)  => (CGEventType::RightMouseUp,   CGMouseButton::Right),
            (MouseButton::Middle, true)  => (CGEventType::OtherMouseDown, CGMouseButton::Center),
            (MouseButton::Middle, false) => (CGEventType::OtherMouseUp,   CGMouseButton::Center),
        };

        if let Ok(event) = CGEvent::new_mouse_event(source, event_type, loc, cg_button) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    // ── Window management: use Cmd shortcuts on macOS ──────────────────────────

    fn cycle_windows(&self, dir: i32) {
        let bench_total = Instant::now();
        let mut guard = CYCLE.lock().unwrap();

        let now = Instant::now();
        // 1s TTL — position-based ordering needs reasonably fresh coordinates.
        let stale = guard.as_ref().map_or(true, |s| now.duration_since(s.last_use).as_millis() > 1000);

        if stale {
            let t0 = Instant::now();
            let mut windows = list_all_windows_cached();
            eprintln!("[BENCH clx+z] list_all_windows: {}ms ({} windows)", t0.elapsed().as_millis(), windows.len());
            // Diagnostic: dump each entry so we can catch invisible/phantom
            // windows sneaking into the cycle. Temporary — remove once the
            // "clx+z goes to an invisible window" report is resolved.
            for (i, w) in windows.iter().enumerate() {
                eprintln!(
                    "[clx+z]   [{i}] pid={} wid={} disp={} pos=({:.0},{:.0}) title={:?}",
                    w.pid, w.window_id, w.display_id, w.cx, w.cy, w.title
                );
            }
            if windows.is_empty() { return; }

            let t1 = Instant::now();
            let front_wid = frontmost_window_id();
            eprintln!("[BENCH clx+z] frontmost_window_id: {}ms", t1.elapsed().as_millis());

            // Sort by configured position-based order (read from cached static,
            // NOT from disk — ticker thread must not do file I/O or eprintln).
            let order = get_cycle_order();
            sort_windows_by_order(&mut windows, &order);

            // Try to find the currently focused window in the new list.
            let prev_wid = guard.as_ref().map_or(0, |s| s.last_activated_wid);
            let anchor_wid = if prev_wid != 0 { prev_wid } else { front_wid };
            let start_idx = windows.iter()
                .position(|w| w.window_id == anchor_wid)
                .or_else(|| windows.iter().position(|w| w.window_id == front_wid))
                .unwrap_or(0);

            *guard = Some(CycleState {
                windows,
                index: start_idx,
                last_use: now,
                last_activated_wid: anchor_wid,
            });
        }

        let state = guard.as_mut().unwrap();
        state.last_use = now;
        let len = state.windows.len();
        if len == 0 { return; }

        // Detect external window switches (mouse click, Cmd+Tab, etc.).
        let front_wid = frontmost_window_id();
        let current_idx = if front_wid != 0
            && front_wid != state.last_activated_wid
            && state.windows.iter().any(|w| w.window_id == front_wid)
        {
            state.windows.iter().position(|w| w.window_id == front_wid).unwrap()
        } else {
            state.index.min(len - 1)
        };

        // Advance index, wrapping around.
        let next_idx = if dir > 0 {
            (current_idx + 1) % len
        } else {
            if current_idx == 0 { len - 1 } else { current_idx - 1 }
        };

        state.index = next_idx;
        state.last_activated_wid = state.windows[next_idx].window_id;
        let entry = state.windows[next_idx].clone();
        drop(guard);

        let t_act = Instant::now();
        activate_window(&entry);
        eprintln!("[BENCH clx+z] activate_window: {}ms | total: {}ms", t_act.elapsed().as_millis(), bench_total.elapsed().as_millis());
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
        let bench_total = Instant::now();
        // Remember the cycle's current window so we can restore focus after arrange.
        let restore_entry = if let Ok(mut guard) = CYCLE.lock() {
            if let Some(ref mut s) = *guard {
                s.last_use = Instant::now();
                let idx = s.index;
                if idx < s.windows.len() {
                    Some(s.windows[idx].clone())
                } else { None }
            } else { None }
        } else { None };

        // Group windows by display so each display arranges independently.
        let t0 = Instant::now();
        let all_windows = list_all_windows_cached();
        eprintln!("[BENCH clx+c] list_all_windows #1: {}ms ({} windows)", t0.elapsed().as_millis(), all_windows.len());
        let mut displays: std::collections::HashMap<u32, Vec<&WindowEntry>> = std::collections::HashMap::new();
        for w in &all_windows {
            displays.entry(w.display_id).or_default().push(w);
        }

        let mut frames: Vec<(AXUIElementRef, f64, f64, f64, f64)> = Vec::new();

        for (&display_id, _display_windows) in &displays {
            let t_ax = Instant::now();
            let windows = get_all_ax_window_refs_stable(display_id, &all_windows);
            eprintln!("[BENCH clx+c] get_all_ax_window_refs_stable (display {}): {}ms ({} windows)", display_id, t_ax.elapsed().as_millis(), windows.len());
            let n = windows.len();
            if n == 0 { continue; }

            let (ax, ay, aw, ah) = visible_work_area_for_display(display_id);

            match mode {
                ArrangeMode::Stacked => {
                    let dx = 72.0_f64.min(aw / n as f64);
                    let dy = (48.0_f64 * 2.0 / 3.0).min(ah / n as f64);
                    let w = (aw / 2.0).max(aw - 2.0 * dx - (n as f64 - 2.0) * dx + dx);
                    let h = (ah / 2.0).max(ah - 2.0 * dy - (n as f64 - 2.0) * dy + dy);
                    for (k, win) in windows.iter().enumerate() {
                        let x = ax + dx * k as f64;
                        let y = ay + dy * k as f64;
                        frames.push((*win, x, y, w, h));
                    }
                }
                ArrangeMode::SideBySide => {
                    let cols = if aw <= ah {
                        let c = (n as f64).sqrt() as usize;
                        c.max(1)
                    } else {
                        let r = (n as f64).sqrt() as usize;
                        let r = r.max(1);
                        (n + r - 1) / r
                    };
                    let rows = (n + cols - 1) / cols;
                    let cell_w = aw / cols as f64;
                    let cell_h = ah / rows as f64;
                    for (k, win) in windows.iter().enumerate() {
                        let col = k % cols;
                        let row = k / cols;
                        let x = ax + col as f64 * cell_w;
                        let y = ay + row as f64 * cell_h;
                        frames.push((*win, x, y, cell_w, cell_h));
                    }
                }
            }
        }

        // Phase 2: Resize all windows in PARALLEL (each window is independent).
        // AX resize is slow (~20-50ms per window). Parallel = total time ≈ one window.
        {
            let t_resize = Instant::now();
            let handles: Vec<_> = frames.iter().map(|&(win, x, y, w, h)| {
                let win = win as usize; // cast to usize to make it Send
                std::thread::spawn(move || {
                    let t = Instant::now();
                    unsafe { ax_set_window_frame(win as *mut std::ffi::c_void, x, y, w, h); }
                    t.elapsed().as_millis()
                })
            }).collect();
            let times: Vec<u128> = handles.into_iter().filter_map(|h| h.join().ok()).collect();
            eprintln!("[BENCH clx+c] parallel resize: {}ms wall | per-window: {:?}ms", t_resize.elapsed().as_millis(), times);
        }

        // Phase 3: Z-order — card deck fan-out from current window.
        // Current window = topmost, then alternating next/prev neighbors.
        // Example: windows [0,1,2,3,4], current=2 → raise order: 0,4,1,3,2 (2 raised last = top)
        //
        // Find current window from cycle state.
        let n = frames.len();
        let current_idx = restore_entry.as_ref()
            .and_then(|_e| {
                // Use the cycle state index if valid
                if let Ok(guard) = CYCLE.lock() {
                    if let Some(ref s) = *guard {
                        if s.index < n { return Some(s.index); }
                    }
                }
                None
            })
            .unwrap_or(0);

        // Build raise order: farthest from current first, current last (topmost).
        // Distance = |idx - current_idx|, raise in descending distance order.
        let mut z_order: Vec<usize> = (0..n).collect();
        z_order.sort_by(|&a, &b| {
            let da = (a as isize - current_idx as isize).unsigned_abs();
            let db = (b as isize - current_idx as isize).unsigned_abs();
            db.cmp(&da) // farthest first (raised first = bottom), closest last (raised last = top)
        });

        unsafe {
            for &idx in &z_order {
                let (win, _, _, _, _) = frames[idx];
                let attr = CFString::new("AXMain");
                AXUIElementSetAttributeValue(
                    win,
                    attr.as_concrete_TypeRef() as CFStringRefRaw,
                    core_foundation::boolean::CFBoolean::true_value().as_CFTypeRef(),
                );
                AXUIElementPerformAction(win, CFString::new("AXRaise").as_concrete_TypeRef() as CFStringRefRaw);
            }

            // Release all retained refs.
            for &(win, _, _, _, _) in &frames {
                CFRelease(win as *const _);
            }
        }

        // Restore focus to the window that was active before arrange.
        if let Some(ref entry) = restore_entry {
            let t_act = Instant::now();
            activate_window(entry);
            eprintln!("[BENCH clx+c] activate_window: {}ms", t_act.elapsed().as_millis());
        }
        eprintln!("[BENCH clx+c] total: {}ms", bench_total.elapsed().as_millis());

        // Invalidate the cycle snapshot so Space+Z picks up new window positions.
        if let Ok(mut g) = CYCLE.lock() {
            *g = None;
        }
    }

    fn close_tab(&self) {
        // Cmd+W
        if let Some(cg_key) = keycode_to_cg_keycode(KeyCode::W) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagCommand);
        }
    }

    fn close_window(&self) {
        // Cmd+W
        if let Some(cg_key) = keycode_to_cg_keycode(KeyCode::W) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagCommand);
        }
    }

    fn kill_window(&self) {
        // Cmd+Q
        if let Some(cg_key) = keycode_to_cg_keycode(KeyCode::Q) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagCommand);
        }
    }

    fn start_aec_mic(&self) -> Option<Box<dyn capslockx_core::platform::SystemAudioStream>> {
        match crate::voice_capture::VoiceCapture::new() {
            Ok(cap) => {
                if let Err(e) = cap.start() {
                    eprintln!("[CLX] VoiceProcessingIO start failed: {e}");
                    return None;
                }
                eprintln!("[CLX] VoiceProcessingIO mic with AEC started");
                Some(Box::new(cap))
            }
            Err(e) => {
                eprintln!("[CLX] VoiceProcessingIO unavailable: {e}, falling back to cpal");
                None
            }
        }
    }

    fn start_system_audio(&self) -> Option<Box<dyn capslockx_core::platform::SystemAudioStream>> {
        // Try Core Audio Taps first (macOS 14.2+, cleaner digital reference signal).
        match crate::audio_tap::AudioTapCapture::new() {
            Ok(cap) => {
                eprintln!("[CLX] system audio: using Core Audio Tap (digital reference)");
                return Some(Box::new(cap));
            }
            Err(e) => {
                eprintln!("[CLX] system audio: Core Audio Tap unavailable ({e}), falling back to ScreenCaptureKit");
            }
        }
        // Fallback: ScreenCaptureKit (macOS 12.3+).
        match crate::system_audio::SystemAudioCapture::new() {
            Ok(cap) => Some(Box::new(cap)),
            Err(e) => {
                eprintln!("[CLX] system audio: {e}");
                None
            }
        }
    }

    fn open_preferences(&self) { crate::prefs::open_preferences(); }
    fn show_voice_overlay(&self) { crate::voice_overlay::show_overlay(); }
    fn hide_voice_overlay(&self) { crate::voice_overlay::hide_overlay(); }
    fn update_voice_overlay(&self, mic_levels: &[f32], mic_vad: bool, sys_levels: &[f32], sys_vad: bool) {
        crate::voice_overlay::push_dual_audio_levels(mic_levels, mic_vad, sys_levels, sys_vad, None);
    }
    fn update_voice_subtitle(&self, text: &str) {
        crate::voice_overlay::push_audio_levels_with_text(&[], false, Some(text));
    }
    fn update_voice_subtitle_translation(&self, translation: &str) {
        crate::voice_overlay::push_translation(translation);
    }
    fn set_ptt_tray_state(&self, state: capslockx_core::platform::PttTrayState) {
        // Route PTT state changes to the otoji-tray menu-bar icon (via the
        // shared Unix datagram socket) instead of the CLX tray. The CLX tray
        // is reserved for CapsLock on/off; otoji owns voice/PTT visuals.
        use capslockx_core::platform::PttTrayState::*;
        use capslockx_core::modules::voice_otoji::{notify_tray, TrayState};
        let ts = match state {
            Idle => TrayState::Idle,
            Recording => TrayState::ListenSilent,
            Processing => TrayState::Decoding,
            NoteMode => TrayState::ListenSilent,
        };
        notify_tray(ts);
    }

    fn get_selected_text(&self) -> String {
        unsafe {
            // Get the focused app's AXUIElement, then read AXSelectedText.
            let sys_wide = AXUIElementCreateApplication(0); // system-wide element
            // Actually, use AXUIElementCreateSystemWide for the focused element.
            extern "C" {
                fn AXUIElementCreateSystemWide() -> AXUIElementRef;
            }
            let sys = AXUIElementCreateSystemWide();
            if sys.is_null() { return String::new(); }

            // Get focused element.
            let attr_focused = CFString::new("AXFocusedUIElement");
            let mut focused: *mut std::ffi::c_void = std::ptr::null_mut();
            let err = AXUIElementCopyAttributeValue(
                sys,
                attr_focused.as_concrete_TypeRef() as CFStringRefRaw,
                &mut focused,
            );
            CFRelease(sys as *const _);
            if err != 0 || focused.is_null() { return String::new(); }

            // Get selected text from focused element.
            let attr_sel = CFString::new("AXSelectedText");
            let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
            let err = AXUIElementCopyAttributeValue(
                focused as AXUIElementRef,
                attr_sel.as_concrete_TypeRef() as CFStringRefRaw,
                &mut value,
            );
            CFRelease(focused as *const _);
            if err != 0 || value.is_null() { return String::new(); }

            // Convert NSString to Rust string.
            let sel_utf8 = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char
                = std::mem::transmute(objc_msgSend as *const ());
            let cstr = f(value, sel_utf8);
            let result = if !cstr.is_null() {
                std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
            } else {
                String::new()
            };
            CFRelease(value as *const _);
            result
        }
    }

    fn get_clipboard_text(&self) -> String {
        unsafe {
            let sel_gp = sel_registerName(b"generalPasteboard\0".as_ptr() as *const _);
            let sel_str = sel_registerName(b"stringForType:\0".as_ptr() as *const _);
            let sel_utf8 = sel_registerName(b"UTF8String\0".as_ptr() as *const _);
            let pb_cls = objc_getClass(b"NSPasteboard\0".as_ptr() as *const _);

            let f0: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            let pb = f0(pb_cls, sel_gp);
            if pb.is_null() { return String::new(); }

            // NSString for type
            let ns_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
            let sel_with = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
            let f_ns: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            let ns_type = f_ns(ns_cls, sel_with, b"public.utf8-plain-text\0".as_ptr() as *const _);

            let f1: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            let ns_str = f1(pb, sel_str, ns_type);
            if ns_str.is_null() { return String::new(); }

            let f_utf8: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char
                = std::mem::transmute(objc_msgSend as *const ());
            let cstr = f_utf8(ns_str, sel_utf8);
            if cstr.is_null() { return String::new(); }
            std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
        }
    }

    fn set_clipboard_text(&self, text: &str) {
        unsafe {
            let sel_gp = sel_registerName(b"generalPasteboard\0".as_ptr() as *const _);
            let sel_clear = sel_registerName(b"clearContents\0".as_ptr() as *const _);
            let sel_set = sel_registerName(b"setString:forType:\0".as_ptr() as *const _);
            let pb_cls = objc_getClass(b"NSPasteboard\0".as_ptr() as *const _);

            let f0: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            let pb = f0(pb_cls, sel_gp);
            if pb.is_null() { return; }
            f0(pb, sel_clear);

            let ns_cls = objc_getClass(b"NSString\0".as_ptr() as *const _);
            let sel_with = sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const _);
            let f_ns: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *const std::ffi::c_char) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            let ctext = std::ffi::CString::new(text).unwrap_or_default();
            let ns_str = f_ns(ns_cls, sel_with, ctext.as_ptr());
            let ns_type = f_ns(ns_cls, sel_with, b"public.utf8-plain-text\0".as_ptr() as *const _);

            let f2: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void, *mut std::ffi::c_void) -> bool
                = std::mem::transmute(objc_msgSend as *const ());
            f2(pb, sel_set, ns_str, ns_type);
        }
    }

    fn toggle_keyboard_layout_hud(&self) {
        crate::keyboard_layout_overlay::toggle_overlay();
    }

    fn show_brainstorm_overlay(&self, text: &str) {
        crate::brainstorm_overlay::show_overlay(text);
    }

    fn hide_brainstorm_overlay(&self) {
        crate::brainstorm_overlay::hide_overlay();
    }

    fn show_prompt_input(&self, title: &str, message: &str, prefill: &str) -> Option<String> {
        // Non-modal panel — doesn't block the main run loop.
        // Voice overlay and other UI keeps updating while this is open.
        crate::brainstorm_overlay::show_prompt_panel(title, message, prefill)
    }

}

// Legacy modal prompt removed — now uses subprocess (clx-prompt).
#[allow(dead_code)]
fn _show_prompt_input_modal_legacy(title: &str, _message: &str, _prefill: &str) -> Option<String> {
    eprintln!("[CLX] legacy prompt not available, use clx-prompt subprocess");
    let _ = title;
    None
}

