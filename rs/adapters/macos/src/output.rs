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

/// A single window: app pid + index into the app's AXWindows array.
#[derive(Clone)]
struct WindowEntry {
    pid: i64,
    window_index: usize,
    title: String,
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

/// List all individual windows across all GUI apps, in z-order (on-screen first)
/// then by app launch order for off-screen apps. Each window is a separate entry.
fn list_all_windows() -> Vec<WindowEntry> {
    unsafe {
        let mut entries = Vec::new();
        let mut seen_pids = std::collections::HashSet::new();

        extern "C" {
            fn CFArrayGetCount(arr: CFArrayRef) -> isize;
            fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            fn CFDictionaryGetValue(dict: *const std::ffi::c_void, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
            fn CFNumberGetValue(number: *const std::ffi::c_void, the_type: i32, value_ptr: *mut std::ffi::c_void) -> bool;
        }

        // 1. On-screen windows in z-order — get pids with layer==0
        let on_screen_opts: u32 = (1 << 0) | (1 << 4);
        let list_ref = CGWindowListCopyWindowInfo(on_screen_opts, 0);
        let mut ordered_pids: Vec<i64> = Vec::new();
        if !list_ref.is_null() {
            let count = CFArrayGetCount(list_ref);
            let key_pid = CFString::new("kCGWindowOwnerPID");
            let key_layer = CFString::new("kCGWindowLayer");
            let mut pid_seen = std::collections::HashSet::new();
            for i in 0..count {
                let dict = CFArrayGetValueAtIndex(list_ref, i);
                if dict.is_null() { continue; }
                let pid_ref = CFDictionaryGetValue(dict, key_pid.as_concrete_TypeRef() as *const _);
                let layer_ref = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
                if pid_ref.is_null() || layer_ref.is_null() { continue; }
                let mut pid: i64 = 0;
                let mut layer: i64 = 0;
                CFNumberGetValue(pid_ref, 4, &mut pid as *mut _ as *mut _);
                CFNumberGetValue(layer_ref, 4, &mut layer as *mut _ as *mut _);
                if layer == 0 && pid_seen.insert(pid) {
                    ordered_pids.push(pid);
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

        // 3. For each app pid, enumerate its AX windows
        for &pid in &ordered_pids {
            if !seen_pids.insert(pid) { continue; }
            if let Some((app_ref, arr, count)) = ax_windows_for_pid(pid as i32) {
                for wi in 0..count {
                    let win = CFArrayGetValueAtIndex(arr as CFArrayRef, wi as isize);
                    if win.is_null() { continue; }
                    let title = ax_window_title(win as *mut _);
                    // Skip windows with empty titles (usually utility/palette windows)
                    if !title.is_empty() {
                        entries.push(WindowEntry { pid, window_index: wi, title });
                    }
                }
                CFRelease(arr as *const _);
                CFRelease(app_ref as *const _);
            }
        }

        // Sort by (pid, title) for stable ordering across cycle and arrange.
        entries.sort_by(|a, b| a.pid.cmp(&b.pid).then_with(|| a.title.cmp(&b.title)));

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

/// Collect AXUIElementRef handles for all titled windows across all GUI apps,
/// sorted by (pid, title) for stable ordering regardless of z-order.
/// The returned refs are retained — caller must CFRelease each one.
fn get_all_ax_window_refs_stable() -> Vec<AXUIElementRef> {
    unsafe {
        extern "C" {
            fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            fn CFRetain(cf: *const std::ffi::c_void) -> *const std::ffi::c_void;
        }

        let entries = list_all_windows(); // already sorted by (pid, title)

        let mut refs: Vec<AXUIElementRef> = Vec::new();

        for entry in &entries {
            if let Some((app_ref, arr, count)) = ax_windows_for_pid(entry.pid as i32) {
                // Match by title rather than index for stability.
                let mut found = false;
                for wi in 0..count {
                    let win = CFArrayGetValueAtIndex(arr as CFArrayRef, wi as isize);
                    if win.is_null() { continue; }
                    let title = ax_window_title(win as *mut _);
                    if title == entry.title {
                        CFRetain(win);
                        refs.push(win as AXUIElementRef);
                        found = true;
                        break;
                    }
                }
                // Fallback to index if title match fails.
                if !found && entry.window_index < count {
                    let win = CFArrayGetValueAtIndex(arr as CFArrayRef, entry.window_index as isize);
                    if !win.is_null() {
                        CFRetain(win);
                        refs.push(win as AXUIElementRef);
                    }
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
            f(app, sel_activate, 2); // NSApplicationActivateIgnoringOtherApps
        }

        // 2. Raise the specific window via AXUIElement — match by title for stability.
        if let Some((app_ref, arr, count)) = ax_windows_for_pid(entry.pid as i32) {
            extern "C" {
                fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
            }
            // Find the window by title first, fall back to index.
            let mut target: *const std::ffi::c_void = std::ptr::null();
            if !entry.title.is_empty() {
                for wi in 0..count {
                    let win = CFArrayGetValueAtIndex(arr as CFArrayRef, wi as isize);
                    if win.is_null() { continue; }
                    if ax_window_title(win as *mut _) == entry.title {
                        target = win;
                        break;
                    }
                }
            }
            if target.is_null() && entry.window_index < count {
                target = CFArrayGetValueAtIndex(arr as CFArrayRef, entry.window_index as isize);
            }
            if !target.is_null() {
                // AXRaise brings the window to front within the app.
                let action = CFString::new("AXRaise");
                AXUIElementPerformAction(
                    target as AXUIElementRef,
                    action.as_concrete_TypeRef() as CFStringRefRaw,
                );

                // Also set AXMain to true to make it the main window.
                let attr_main = CFString::new("AXMain");
                let cf_true: *const std::ffi::c_void = {
                    extern "C" { #[allow(non_upper_case_globals)]
                    static kCFBooleanTrue: *const std::ffi::c_void; }
                    kCFBooleanTrue
                };
                AXUIElementSetAttributeValue(
                    target as AXUIElementRef,
                    attr_main.as_concrete_TypeRef() as CFStringRefRaw,
                    cf_true,
                );
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

    /// Ctrl+key — set CGEventFlagControl on the event itself.
    fn key_tap_ctrl(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagControl);
        }
    }

    /// Ctrl+Shift+key — set both flags on the event itself.
    fn key_tap_ctrl_shifted(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            Self::tap_with_flags(
                cg_key,
                CGEventFlags::CGEventFlagControl | CGEventFlags::CGEventFlagShift,
            );
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

        // Always take a fresh snapshot. Find the currently-focused window
        // (which may have changed via manual click) and start cycling from there.
        let windows = list_all_windows();
        if windows.is_empty() { return; }

        // Find the currently-focused window by checking frontmost app pid.
        // This handles manual clicks, Cmd+Tab, etc. between Z presses.
        let frontmost_pid = unsafe {
            let ws = objc_getClass(b"NSWorkspace\0".as_ptr() as *const _);
            let shared: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(ws, sel_registerName(b"sharedWorkspace\0".as_ptr() as *const _))
            };
            let front_app: *mut std::ffi::c_void = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(shared, sel_registerName(b"frontmostApplication\0".as_ptr() as *const _))
            };
            if !front_app.is_null() {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> i32
                    = std::mem::transmute(objc_msgSend as *const ());
                f(front_app, sel_registerName(b"processIdentifier\0".as_ptr() as *const _)) as i64
            } else {
                0
            }
        };

        // Find the frontmost app's first window in our sorted list.
        let start_idx = windows.iter().position(|w| w.pid == frontmost_pid).unwrap_or(0);

        *guard = Some(CycleState {
            windows,
            index: start_idx,
            last_use: Instant::now(),
        });

        let state = guard.as_mut().unwrap();
        let len = state.windows.len();
        if len == 0 { return; }

        // Advance index in the snapshot.
        let new_idx = if dir > 0 {
            (state.index + 1) % len
        } else {
            (state.index + len - 1) % len
        };
        state.index = new_idx;
        state.last_use = Instant::now();

        let entry = state.windows[new_idx].clone();
        drop(guard); // release lock before FFI call
        activate_window(&entry);
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
        let windows = get_all_ax_window_refs_stable();
        let n = windows.len();
        if n == 0 { return; }

        let (ax, ay, aw, ah) = visible_work_area();

        unsafe {
            match mode {
                ArrangeMode::Stacked => {
                    // Cascade: offset each window by ~48px, sized to fill most of work area.
                    let dx = 48.0_f64.min(aw / n as f64);
                    let dy = 48.0_f64.min(ah / n as f64);
                    let w = (aw / 2.0).max(aw - 2.0 * dx - (n as f64 - 2.0) * dx + dx);
                    let h = (ah / 2.0).max(ah - 2.0 * dy - (n as f64 - 2.0) * dy + dy);
                    for (k, win) in windows.iter().enumerate() {
                        let x = ax + dx * k as f64;
                        let y = ay + dy * (n - k - 1) as f64;
                        ax_set_window_frame(*win, x, y, w, h);
                    }
                }
                ArrangeMode::SideBySide => {
                    // Grid: sqrt-based columns/rows like the Windows adapter.
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
                        ax_set_window_frame(*win, x, y, cell_w, cell_h);
                    }
                }
            }

            // Release all retained refs.
            for win in &windows {
                CFRelease(*win as *const _);
            }
        }

        // eprintln!("[CLX] arrange_windows({:?}): tiled {} windows in work area ({},{} {}x{})",
        //     mode, n, ax, ay, aw, ah);
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
}
