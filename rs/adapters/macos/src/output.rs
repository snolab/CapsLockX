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

use crate::key_map::keycode_to_cg_keycode;

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

/// Visible, on-screen windows only (kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements).
const WINDOW_LIST_OPTIONS: u32 = (1 << 0) | (1 << 4);

/// A visible window: (pid, window_layer, window_id).
#[derive(Debug)]
struct WinInfo {
    pid: i64,
    #[allow(dead_code)]
    layer: i64,
}

/// Enumerate visible, normal (layer 0) windows in front-to-back order.
fn list_visible_windows() -> Vec<WinInfo> {
    unsafe {
        let list_ref = CGWindowListCopyWindowInfo(WINDOW_LIST_OPTIONS, 0);
        if list_ref.is_null() {
            return Vec::new();
        }

        let count: isize = {
            extern "C" { fn CFArrayGetCount(arr: CFArrayRef) -> isize; }
            CFArrayGetCount(list_ref)
        };

        let key_pid = CFString::new("kCGWindowOwnerPID");
        let key_layer = CFString::new("kCGWindowLayer");

        let mut results = Vec::new();
        for i in 0..count {
            extern "C" {
                fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
                fn CFDictionaryGetValue(dict: *const std::ffi::c_void, key: *const std::ffi::c_void) -> *const std::ffi::c_void;
            }
            let dict = CFArrayGetValueAtIndex(list_ref, i);
            if dict.is_null() { continue; }

            let pid_ref = CFDictionaryGetValue(dict, key_pid.as_concrete_TypeRef() as *const _);
            let layer_ref = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
            if pid_ref.is_null() || layer_ref.is_null() { continue; }

            extern "C" {
                fn CFNumberGetValue(number: *const std::ffi::c_void, the_type: i32, value_ptr: *mut std::ffi::c_void) -> bool;
            }
            let mut pid: i64 = 0;
            let mut layer: i64 = 0;
            // kCFNumberSInt64Type = 4
            CFNumberGetValue(pid_ref, 4, &mut pid as *mut _ as *mut _);
            CFNumberGetValue(layer_ref, 4, &mut layer as *mut _ as *mut _);

            // Layer 0 = normal windows (skip menu bar, dock, desktop, etc.)
            if layer == 0 {
                results.push(WinInfo { pid, layer });
            }
        }

        CFRelease(list_ref);

        // Deduplicate by pid, keeping front-to-back order (first occurrence wins).
        let mut seen = std::collections::HashSet::new();
        results.retain(|w| seen.insert(w.pid));

        results
    }
}

/// Activate the app with the given pid using NSRunningApplication.
fn activate_pid(pid: i64) {
    unsafe {
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr() as *const _);
        if cls.is_null() { return; }
        let sel_running = sel_registerName(
            b"runningApplicationWithProcessIdentifier:\0".as_ptr() as *const _,
        );
        let app: *mut std::ffi::c_void = {
            // NSRunningApplication class method with pid_t argument.
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, i32) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls, sel_running, pid as i32)
        };
        if app.is_null() { return; }

        let sel_activate = sel_registerName(
            b"activateWithOptions:\0".as_ptr() as *const _,
        );
        // NSApplicationActivateIgnoringOtherApps = 2
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, u64) -> bool
            = std::mem::transmute(objc_msgSend as *const ());
        f(app, sel_activate, 2);
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
        // Enumerate visible windows (front-to-back) and activate the next/prev app.
        let windows = list_visible_windows();
        if windows.is_empty() { return; }

        // Find which pid is currently frontmost (index 0 in front-to-back list).
        // Cycle to the next or previous app in the z-order.
        let target_idx = if dir > 0 { 1 } else { windows.len() - 1 };
        if let Some(w) = windows.get(target_idx) {
            activate_pid(w.pid);
        }
    }

    fn arrange_windows(&self, mode: ArrangeMode) {
        // macOS Sequoia+ tiling: use Ctrl+Cmd+arrow shortcuts (Globe key).
        // Stacked = fill screen (Globe+F or Ctrl+Cmd+F via green button menu).
        // SideBySide: no single-key shortcut; use Mission Control (Ctrl+Up).
        match mode {
            ArrangeMode::Stacked => {
                // Ctrl+Cmd+F = toggle fullscreen (works in most apps).
                if let Some(cg_key) = keycode_to_cg_keycode(KeyCode::F) {
                    Self::tap_with_flags(
                        cg_key,
                        CGEventFlags::CGEventFlagControl | CGEventFlags::CGEventFlagCommand,
                    );
                }
            }
            ArrangeMode::SideBySide => {
                // Ctrl+Up = Mission Control (show all windows side by side).
                if let Some(cg_key) = keycode_to_cg_keycode(KeyCode::Up) {
                    Self::tap_with_flags(cg_key, CGEventFlags::CGEventFlagControl);
                }
            }
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
}
