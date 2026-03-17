//! CGEventTap keyboard hook – bridges macOS input events to ClxEngine.
//!
//! Creates a CGEventTap via raw FFI that intercepts keyboard events at the HID
//! level.  Events can be **suppressed** by returning NULL from the callback
//! (the `core-graphics` Rust wrapper has a bug where `None` still passes the
//! original event through, so we bypass it).
//!
//! Requires Accessibility permission (System Settings → Privacy & Security
//! → Accessibility).

use std::ptr;
use std::sync::Arc;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};

use once_cell::sync::Lazy;

use capslockx_core::{ClxEngine, CoreResponse};

use crate::key_map::cg_keycode_to_keycode;
use crate::output::MacPlatform;

// ── Engine (created once on first use) ───────────────────────────────────────

static ENGINE: Lazy<Arc<ClxEngine>> = Lazy::new(|| {
    let platform = Arc::new(MacPlatform::new());
    ClxEngine::new(platform)
});

// ── Raw FFI for CGEventTap (bypasses the buggy Rust wrapper) ────────────────

type CGEventRef = *mut std::ffi::c_void;
type CGEventMask = u64;
type CGEventTapProxy = *mut std::ffi::c_void;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    etype: CGEventType,
    event: CGEventRef,
    user_info: *mut std::ffi::c_void,
) -> CGEventRef;

extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBack,
        user_info: *mut std::ffi::c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

/// Build a CGEventMask from event types.
fn event_mask(types: &[CGEventType]) -> CGEventMask {
    types.iter().fold(0u64, |mask, &t| mask | (1u64 << t as u64))
}

// ── Raw callback ─────────────────────────────────────────────────────────────
// Returns the event pointer to pass through, or NULL to suppress.

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    etype: CGEventType,
    event: CGEventRef,
    _user_info: *mut std::ffi::c_void,
) -> CGEventRef {
    use foreign_types::ForeignType;

    // Wrap the raw pointer so we can call core-graphics methods on it.
    // ManuallyDrop ensures we don't free the event (the OS still owns it).
    let cg_event = std::mem::ManuallyDrop::new(CGEvent::from_ptr(event as *mut _));

    match handle_event(etype, &cg_event) {
        Some(_) => event,       // pass through: return original pointer
        None    => ptr::null_mut(),  // suppress: return NULL
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Install the CGEventTap hook and run the CFRunLoop.
/// This function blocks forever.
pub fn install_and_run() {
    // Force engine initialisation.
    Lazy::force(&ENGINE);

    let mask = event_mask(&[
        CGEventType::KeyDown,
        CGEventType::KeyUp,
        CGEventType::FlagsChanged,
    ]);

    let tap: CFMachPortRef = unsafe {
        CGEventTapCreate(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            mask,
            raw_callback,
            ptr::null_mut(),
        )
    };

    if tap.is_null() {
        eprintln!("[CLX] ERROR: Failed to create CGEventTap.");
        eprintln!("[CLX] Grant Accessibility permission in:");
        eprintln!("[CLX]   System Settings → Privacy & Security → Accessibility");
        std::process::exit(1);
    }

    unsafe {
        // Wrap in CFMachPort so we can create a run-loop source.
        let mach_port = core_foundation::mach_port::CFMachPort::wrap_under_create_rule(tap);
        let loop_source = mach_port
            .create_runloop_source(0)
            .expect("failed to create run loop source from CGEventTap");
        let run_loop = CFRunLoop::get_current();
        run_loop.add_source(&loop_source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
    }

    eprintln!("[CLX] CGEventTap installed – running…");

    // Block forever on the run loop.
    CFRunLoop::run_current();
}

// ── Event handler ────────────────────────────────────────────────────────────

fn handle_event(event_type: CGEventType, event: &CGEvent) -> Option<()> {
    // Ignore our own injected events.
    if event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA)
        == crate::output::SELF_INJECT_TAG
    {
        return Some(());
    }

    match event_type {
        CGEventType::KeyDown | CGEventType::KeyUp => {
            let cg_keycode =
                event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            let pressed = matches!(event_type, CGEventType::KeyDown);
            let code = cg_keycode_to_keycode(cg_keycode);

            match ENGINE.on_key_event(code, pressed) {
                CoreResponse::Suppress => None,
                CoreResponse::PassThrough => Some(()),
            }
        }
        CGEventType::FlagsChanged => {
            let cg_keycode =
                event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
            let flags = event.get_flags();
            let code = cg_keycode_to_keycode(cg_keycode);
            let pressed = is_modifier_pressed(cg_keycode, flags);

            match ENGINE.on_key_event(code, pressed) {
                CoreResponse::Suppress => None,
                CoreResponse::PassThrough => Some(()),
            }
        }
        _ => Some(()),
    }
}

/// Determine whether a modifier key is pressed or released from CGEventFlags.
fn is_modifier_pressed(keycode: u16, flags: CGEventFlags) -> bool {
    match keycode {
        0x38 | 0x3C => flags.contains(CGEventFlags::CGEventFlagShift),
        0x3B | 0x3E => flags.contains(CGEventFlags::CGEventFlagControl),
        0x3A | 0x3D => flags.contains(CGEventFlags::CGEventFlagAlternate),
        0x37 | 0x36 => flags.contains(CGEventFlags::CGEventFlagCommand),
        0x39        => flags.contains(CGEventFlags::CGEventFlagAlphaShift),
        _ => false,
    }
}
