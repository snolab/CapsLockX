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
// presses.  Resets after 2 seconds of inactivity so a fresh press gets a
// fresh snapshot.

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

/// List all individual windows across all GUI apps, in z-order (on-screen first)
/// then by app launch order for off-screen apps. Each window is a separate entry.
fn list_all_windows() -> Vec<WindowEntry> {
    let bounds_map = build_window_bounds_map();
    let main_display = CGDisplay::main().id;
    unsafe {
        let mut entries = Vec::new();
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

        // Private API: get CGWindowID directly from AXUIElement.
        extern "C" {
            fn _AXUIElementGetWindow(element: AXUIElementRef, wid: *mut u32) -> i32;
        }

        // 3. For each app pid, enumerate its AX windows
        for &pid in &ordered_pids {
            if !seen_pids.insert(pid) { continue; }
            if let Some((app_ref, arr, count)) = ax_windows_for_pid(pid as i32) {
                for wi in 0..count {
                    let win = CFArrayGetValueAtIndex(arr as CFArrayRef, wi as isize);
                    if win.is_null() { continue; }
                    let title = ax_window_title(win as *mut _);
                    if !title.is_empty() {
                        // Get CGWindowID directly from AXUIElement (private API, reliable).
                        let mut wid: u32 = 0;
                        _AXUIElementGetWindow(win as AXUIElementRef, &mut wid);
                        // Only include windows on the current Space.
                        // onscreen_wids has all CGWindowIDs with isOnscreen=true.
                        // This filters out minimized windows and (on some macOS
                        // versions) windows on other Spaces.
                        // Skip wid==0 windows — they can't be reliably matched
                        // or activated, causing phantom "skips" during cycling.
                        let on_current_space = wid != 0 && onscreen_wids.contains(&wid);
                        if on_current_space {
                            let display_id = if wid != 0 {
                                if let Some(&(cx, cy)) = bounds_map.get(&wid) {
                                    display_for_point(cx, cy)
                                } else { main_display }
                            } else { main_display };
                            entries.push(WindowEntry { pid, window_id: wid, window_index: wi, title, display_id });
                        }
                    }
                }
                CFRelease(arr as *const _);
                CFRelease(app_ref as *const _);
            }
        }

        // Keep z-order from CGWindowList (front-to-back, most recently used first).
        // Don't sort — natural order is what Alt+Tab uses.

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
fn get_all_ax_window_refs_stable(display_id: u32) -> Vec<AXUIElementRef> {
    unsafe {
        extern "C" {
            fn CFRetain(cf: *const std::ffi::c_void) -> *const std::ffi::c_void;
        }

        let mut entries = list_all_windows();
        entries.retain(|w| w.display_id == display_id);
        entries.sort_by_key(|w| (w.pid, w.window_id));

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
    eprintln!("[cycle] activating wid={} pid={} {:?}", entry.window_id, entry.pid, entry.title);
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

        eprintln!("[cycle] app activated, now raising window via AX...");
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
        let mut guard = CYCLE.lock().unwrap();

        let now = Instant::now();
        let stale = guard.as_ref().map_or(true, |s| now.duration_since(s.last_use).as_secs() > 5);

        if stale {
            let mut windows = list_all_windows();
            if windows.is_empty() { return; }

            let front_wid = frontmost_window_id();

            // Cycle ALL on-screen windows across all monitors.
            // Sort by (display, pid, window_id) — group by monitor, then app.
            windows.sort_by_key(|w| (w.display_id, w.pid, w.window_id));

            // Try to find the currently focused window in the new list.
            let prev_wid = guard.as_ref().map_or(0, |s| s.last_activated_wid);
            let anchor_wid = if prev_wid != 0 { prev_wid } else { front_wid };
            let start_idx = windows.iter()
                .position(|w| w.window_id == anchor_wid)
                .or_else(|| windows.iter().position(|w| w.window_id == front_wid))
                .unwrap_or(0);

                eprintln!("[cycle] FRESH snapshot ({} windows), anchor_wid={} front_wid={} start_idx={}",
                windows.len(), anchor_wid, front_wid, start_idx);
            for (i, w) in windows.iter().enumerate() {
                eprintln!("[cycle]   [{}] wid={} {:?}{}", i, w.window_id, w.title,
                    if i == start_idx { " <-- anchor" } else { "" });
            }

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

        // Detect current frontmost window to handle external switches
        // (mouse click, Cmd+Tab, etc.).  BUT if the frontmost window is
        // still the one we last activated, trust our stored index — macOS
        // may not have finished raising the window yet, so re-querying
        // would re-anchor to the *previous* position and skip a window.
        let front_wid = frontmost_window_id();
        let current_idx = if front_wid != 0
            && front_wid != state.last_activated_wid
            && state.windows.iter().any(|w| w.window_id == front_wid)
        {
            // User switched windows externally — re-anchor to that window.
            state.windows.iter().position(|w| w.window_id == front_wid).unwrap()
        } else {
            state.index
        };

        // Advance from detected position.
        let next_idx = if dir > 0 {
            current_idx + 1
        } else {
            current_idx.wrapping_sub(1)
        };

        // Check if we've hit a boundary (past first/last window).
        if next_idx >= len || (dir < 0 && current_idx == 0) {
            // At boundary — switch to next/prev Space on the focused monitor,
            // then re-enumerate on next press.
            // Inject Ctrl+Right/Left to trigger macOS Space switching.
            drop(guard);
            eprintln!("[cycle] boundary reached — switching Space (dir={})", dir);
            let arrow_key: u16 = if dir > 0 { 0x7C } else { 0x7B }; // Right : Left
            let flags = core_graphics::event::CGEventFlags::CGEventFlagControl;
            Self::tap_with_flags(arrow_key, flags);
            // Invalidate the cached snapshot so next press re-enumerates.
            if let Ok(mut g) = CYCLE.lock() {
                *g = None;
            }
            return;
        }

        let idx = next_idx;
        state.index = idx;
        state.last_activated_wid = state.windows[idx].window_id;
        let entry = state.windows[idx].clone();
        drop(guard);

        eprintln!("[cycle] idx={} wid={} pid={} title={:?}", idx, entry.window_id, entry.pid, entry.title);
        activate_window(&entry);
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
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

        // Determine which screen to arrange on: use the frontmost window's screen.
        let all_windows = list_all_windows();
        let front_wid = frontmost_window_id();
        let current_display = all_windows.iter()
            .find(|w| w.window_id == front_wid)
            .map(|w| w.display_id)
            .unwrap_or_else(|| CGDisplay::main().id);

        let windows = get_all_ax_window_refs_stable(current_display);
        let n = windows.len();
        if n == 0 { return; }

        let (ax, ay, aw, ah) = visible_work_area_for_display(current_display);

        // Phase 1: Compute all target frames (pure math, instant).
        let mut frames: Vec<(AXUIElementRef, f64, f64, f64, f64)> = Vec::with_capacity(n);

        match mode {
            ArrangeMode::Stacked => {
                let dx = 72.0_f64.min(aw / n as f64);
                let dy = (48.0_f64 * 2.0 / 3.0).min(ah / n as f64);
                let w = (aw / 2.0).max(aw - 2.0 * dx - (n as f64 - 2.0) * dx + dx);
                let h = (ah / 2.0).max(ah - 2.0 * dy - (n as f64 - 2.0) * dy + dy);
                for (k, win) in windows.iter().enumerate() {
                    let x = ax + dx * k as f64;
                    let y = ay + dy * (n - k - 1) as f64;
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

        // Phase 2: Resize all windows in PARALLEL (each window is independent).
        // AX resize is slow (~20-50ms per window). Parallel = total time ≈ one window.
        {
            let handles: Vec<_> = frames.iter().map(|&(win, x, y, w, h)| {
                let win = win as usize; // cast to usize to make it Send
                std::thread::spawn(move || {
                    unsafe { ax_set_window_frame(win as *mut std::ffi::c_void, x, y, w, h); }
                })
            }).collect();
            for h in handles { let _ = h.join(); }
        }

        // Phase 3: Z-order — raise windows in reverse order (last = frontmost).
        // Must be sequential (each depends on previous z-position).
        unsafe {
            for &(win, _, _, _, _) in frames.iter().rev() {
                let attr = CFString::new("AXMain");
                AXUIElementSetAttributeValue(
                    win,
                    attr.as_concrete_TypeRef() as CFStringRefRaw,
                    core_foundation::boolean::CFBoolean::true_value().as_CFTypeRef(),
                );
                AXUIElementPerformAction(win, CFString::new("AXRaise").as_concrete_TypeRef() as CFStringRefRaw);
            }

            // Release all retained refs.
            for win in &windows {
                CFRelease(*win as *const _);
            }
        }

        // Restore focus to the window that was active before arrange.
        if let Some(ref entry) = restore_entry {
            activate_window(entry);
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
        match crate::system_audio::SystemAudioCapture::new() {
            Ok(cap) => Some(Box::new(cap)),
            Err(e) => {
                eprintln!("[CLX] system audio: {e}");
                None
            }
        }
    }

    fn show_voice_overlay(&self) { crate::voice_overlay::show_overlay(); }
    fn hide_voice_overlay(&self) { crate::voice_overlay::hide_overlay(); }
    fn update_voice_overlay(&self, mic_levels: &[f32], mic_vad: bool, sys_levels: &[f32], sys_vad: bool) {
        crate::voice_overlay::push_dual_audio_levels(mic_levels, mic_vad, sys_levels, sys_vad, None);
    }
    fn update_voice_subtitle(&self, text: &str) {
        crate::voice_overlay::push_audio_levels_with_text(&[], false, Some(text));
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

