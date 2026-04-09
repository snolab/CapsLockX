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
use std::sync::{Arc, atomic::{AtomicPtr, AtomicU32, Ordering}};

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField,
};

use once_cell::sync::Lazy;

use capslockx_core::{ClxEngine, ClxConfig, CoreResponse, SpeedConfig};

use crate::key_map::cg_keycode_to_keycode;
use crate::output::MacPlatform;

// ── Engine (created once on first use) ───────────────────────────────────────

pub(crate) static ENGINE: Lazy<Arc<ClxEngine>> = Lazy::new(|| {
    let platform = Arc::new(MacPlatform::new());
    // Load saved config, fall back to defaults.
    let saved = crate::config_store::load();
    let config = saved.into_clx_config();
    let (best_key, _) = config.best_llm_key_and_model();
    eprintln!("[CLX] config loaded: stt={}, correction={}, llm_key={}...",
        config.stt_engine, config.stt_correction,
        &best_key[..best_key.len().min(10)]);
    ClxEngine::with_config(platform, config)
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

/// Global tap reference so the callback can re-enable it after secure input.
static TAP_REF: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(ptr::null_mut());

/// Last tray-active state for edge detection (0 = off, 1 = on, u32::MAX = uninitialised).
static LAST_TRAY_ACTIVE: AtomicU32 = AtomicU32::new(u32::MAX);

// CGEventType raw values for tap-disabled notifications (not in the Rust crate enum).
const TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const TAP_DISABLED_BY_USER: u32    = 0xFFFFFFFF;

// ── Raw callback ─────────────────────────────────────────────────────────────
// Returns the event pointer to pass through, or NULL to suppress.

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    etype: CGEventType,
    event: CGEventRef,
    _user_info: *mut std::ffi::c_void,
) -> CGEventRef {
    use foreign_types::ForeignType;

    // When macOS disables our tap (e.g. during secure input / password fields),
    // it sends a special event type. Re-enable the tap so it resumes working
    // after the secure input ends.
    let etype_raw: u32 = std::mem::transmute(etype);
    if etype_raw == TAP_DISABLED_BY_TIMEOUT || etype_raw == TAP_DISABLED_BY_USER {
        let tap = TAP_REF.load(Ordering::Relaxed);
        if !tap.is_null() {
            // Rate-limit logging: only log first event in a burst, and at most
            // once per 5s. The previous version spammed 583k lines into the
            // watchdog log over a few days.
            use std::sync::atomic::AtomicU64;
            use std::time::{SystemTime, UNIX_EPOCH};
            static LAST_LOG_SEC: AtomicU64 = AtomicU64::new(0);
            static DISABLE_COUNT: AtomicU64 = AtomicU64::new(0);
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            let last = LAST_LOG_SEC.load(Ordering::Relaxed);
            let n = DISABLE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            if now.saturating_sub(last) >= 5 {
                LAST_LOG_SEC.store(now, Ordering::Relaxed);
                let cause = if etype_raw == TAP_DISABLED_BY_TIMEOUT {
                    "timeout (handler too slow → key drops/lag)"
                } else {
                    "user (secure input field)"
                };
                eprintln!("[CLX] CGEventTap disabled — {} (total disables: {}) — re-enabling", cause, n);
            }
            // Emergency stop: release all held keys and stop all modules.
            // Without this, AccModel keeps running (tabs/mouse) because we
            // never received the key-up events while the tap was disabled.
            ENGINE.emergency_stop();
            CGEventTapEnable(tap as CFMachPortRef, true);
        }
        return event;
    }

    // Wrap the raw pointer so we can call core-graphics methods on it.
    // ManuallyDrop ensures we don't free the event (the OS still owns it).
    let cg_event = std::mem::ManuallyDrop::new(CGEvent::from_ptr(event as *mut _));

    // catch_unwind prevents panics from unwinding across the FFI boundary
    // (which is instant abort). Instead, log and pass the event through.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handle_event(etype, &cg_event)
    })) {
        Ok(Some(_)) => event,          // pass through
        Ok(None)    => ptr::null_mut(), // suppress
        Err(_)      => {
            let _ = std::io::Write::write_all(
                &mut std::io::stderr(),
                b"[CLX] PANIC in CGEventTap callback - event passed through\n",
            );
            event // pass through on panic — don't break keyboard
        }
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

    // Store tap reference so the callback can re-enable it after secure input.
    TAP_REF.store(tap as *mut _, Ordering::Relaxed);

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

    // Run the NSApplication event loop (which also processes CFRunLoop sources).
    // This is required for AppKit (NSStatusBar/NSMenu) to function.
    unsafe {
        extern "C" {
            fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
            fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
            fn objc_msgSend(receiver: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, ...) -> *mut std::ffi::c_void;
        }
        let nsapp = objc_getClass(b"NSApplication\0".as_ptr() as *const _);
        let app: *mut std::ffi::c_void = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(nsapp, sel_registerName(b"sharedApplication\0".as_ptr() as *const _))
        };
        let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(app, sel_registerName(b"run\0".as_ptr() as *const _));
    }
}

/// Sync the engine's held_keys with CGEvent modifier flags.
/// This ensures modifiers are tracked even if FlagsChanged up arrives before
/// the key-down event (fast Cmd+Space combos).
fn sync_modifier_flags(flags: &CGEventFlags) {
    use capslockx_core::KeyCode;
    let pairs: &[(CGEventFlags, KeyCode)] = &[
        (CGEventFlags::CGEventFlagCommand,   KeyCode::LWin),
        (CGEventFlags::CGEventFlagShift,     KeyCode::LShift),
        (CGEventFlags::CGEventFlagControl,   KeyCode::LCtrl),
        (CGEventFlags::CGEventFlagAlternate, KeyCode::LAlt),
    ];
    for &(flag, code) in pairs {
        if flags.contains(flag) {
            // Ensure modifier is in held_keys (inject key-down if missing).
            ENGINE.ensure_held(code);
        }
    }
}

// ── Tray icon edge detection ─────────────────────────────────────────────────

/// Check the engine mode and update the tray icon on edge transitions.
fn update_tray_on_edge() {
    let active = u32::from(ENGINE.state().is_clx_active());
    let prev = LAST_TRAY_ACTIVE.swap(active, Ordering::Relaxed);
    if prev != active {
        crate::tray::update_tray_icon(active != 0);
    }
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

            // On macOS, CGEvent carries modifier flags even if the modifier key
            // was released by the time the callback fires. Inject modifiers into
            // held_keys based on the event's flags so Cmd+Space bypass works
            // reliably even with fast key combos.
            if pressed {
                let flags = event.get_flags();
                sync_modifier_flags(&flags);
            }

            let resp = ENGINE.on_key_event(code, pressed);
            update_tray_on_edge();
            match resp {
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

            let resp = ENGINE.on_key_event(code, pressed);
            update_tray_on_edge();

            // Suppress CapsLock if the engine says so (prevents real CapsLock
            // toggling). Other modifiers (Cmd, Shift, Ctrl, Alt) must always
            // pass through — suppressing them breaks system shortcuts.
            if cg_keycode == 0x39 {
                match resp {
                    CoreResponse::Suppress => None,
                    CoreResponse::PassThrough => Some(()),
                }
            } else {
                Some(())
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
