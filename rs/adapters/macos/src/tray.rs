//! macOS menu bar (NSStatusBar) tray icon using raw Objective-C FFI.
//!
//! Provides a status-bar icon that toggles between white (inactive) and blue
//! (CapsLockX active), plus a "Quit" menu item.

use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

// ── Icon data (compiled into the binary) ─────────────────────────────────────

static ICON_WHITE: &[u8] = include_bytes!("../../../../Data/XIconWhite.png");
static ICON_BLUE: &[u8] = include_bytes!("../../../../Data/XIconBlue.png");

// ── Global NSStatusItem reference ────────────────────────────────────────────

static STATUS_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static MIC_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// ── Objective-C runtime FFI ──────────────────────────────────────────────────

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const std::ffi::c_char,
        extra_bytes: usize,
    ) -> *mut c_void;
    fn class_addMethod(
        cls: *mut c_void,
        sel: *mut c_void,
        imp: *const c_void,
        types: *const u8,
    ) -> bool;
    fn class_addProtocol(cls: *mut c_void, protocol: *mut c_void) -> bool;
    fn objc_registerClassPair(cls: *mut c_void);
    fn objc_getProtocol(name: *const std::ffi::c_char) -> *mut c_void;
}

// NSRect return: on aarch64 macOS, small structs are returned in registers.
// We don't need objc_msgSend_stret for NSSize (only 2 f64s).

/// Helper: create an Objective-C selector.
unsafe fn sel(name: &[u8]) -> *mut c_void {
    sel_registerName(name.as_ptr() as *const _)
}

/// Helper: get an Objective-C class by name.
unsafe fn cls(name: &[u8]) -> *mut c_void {
    objc_getClass(name.as_ptr() as *const _)
}

/// Helper: send a zero-arg message returning id.
unsafe fn msg0(obj: *mut c_void, sel: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(obj, sel)
}

/// Helper: send a one-arg (id) message returning id.
unsafe fn msg1_ptr(obj: *mut c_void, sel: *mut c_void, arg: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(obj, sel, arg)
}

/// Create an NSImage from raw PNG bytes, sized to 18x18 for the menu bar.
/// If `template` is true, the image adapts to dark/light menu bar colors.
unsafe fn nsimage_from_bytes_ex(bytes: &[u8], template: bool) -> *mut c_void {
    // NSData *data = [NSData dataWithBytes:bytes length:len];
    let nsdata_cls = cls(b"NSData\0");
    let sel_data = sel(b"dataWithBytes:length:\0");
    let data: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, *const u8, usize) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(nsdata_cls, sel_data, bytes.as_ptr(), bytes.len())
    };
    if data.is_null() {
        eprintln!("[CLX] tray: failed to create NSData for icon");
        return std::ptr::null_mut();
    }

    // NSImage *img = [[NSImage alloc] initWithData:data];
    let nsimage_cls = cls(b"NSImage\0");
    let alloc = msg0(nsimage_cls, sel(b"alloc\0"));
    let img = msg1_ptr(alloc, sel(b"initWithData:\0"), data);
    if img.is_null() {
        eprintln!("[CLX] tray: failed to create NSImage from data");
        return std::ptr::null_mut();
    }

    // [img setSize:NSMakeSize(18, 18)];
    // NSSize is {f64, f64} — on aarch64 passed in registers.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSSize {
        width: f64,
        height: f64,
    }
    let size = NSSize {
        width: 18.0,
        height: 18.0,
    };
    let sel_set_size = sel(b"setSize:\0");
    let f: extern "C" fn(*mut c_void, *mut c_void, NSSize) =
        std::mem::transmute(objc_msgSend as *const ());
    f(img, sel_set_size, size);

    // setTemplate: makes the icon adapt to dark/light menu bar (only for monochrome icons)
    let sel_set_template = sel(b"setTemplate:\0");
    let f: extern "C" fn(*mut c_void, *mut c_void, bool) =
        std::mem::transmute(objc_msgSend as *const ());
    f(img, sel_set_template, template);

    img
}

/// Convenience: create a template image (adapts to dark/light).
unsafe fn nsimage_from_bytes(bytes: &[u8]) -> *mut c_void {
    nsimage_from_bytes_ex(bytes, true)
}

/// Create an NSString from a Rust string slice.
unsafe fn nsstring(s: &str) -> *mut c_void {
    let cls_str = cls(b"NSString\0");
    let sel_utf8 = sel(b"stringWithUTF8String:\0");
    let cstr = std::ffi::CString::new(s).unwrap();
    let f: extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls_str, sel_utf8, cstr.as_ptr())
}

// ── Mic mode label refresh ───────────────────────────────────────────────────

/// Update the "Mic: …" menu item title to reflect the currently active mode and STT engine.
/// Safe to call from the main thread only (NSMenuItem is not thread-safe).
fn refresh_mic_item() {
    unsafe {
        let item = MIC_ITEM.load(Ordering::Acquire);
        if item.is_null() {
            return;
        }
        let mode = crate::mic_mode::active_microphone_mode();
        let mode_name = match mode {
            0 => "Standard",
            1 => "Wide Spectrum",
            2 => "Voice Isolation",
            _ => "Unknown",
        };
        let cfg = crate::config_store::load();
        let engine_name = if cfg.stt_engine == "whisper" {
            "Whisper"
        } else {
            "SenseVoice"
        };
        let title = format!("Mic: {mode_name} \u{00B7} {engine_name}\u{2026}");
        let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        f(item, sel(b"setTitle:\0"), nsstring(&title));
    }
}

/// Register a CLXMenuDelegate ObjC class with `menuWillOpen:` → refresh mic label.
/// Returns a freshly allocated and initialized delegate instance.
unsafe fn create_menu_delegate() -> *mut c_void {
    let nsobject_cls = cls(b"NSObject\0");
    if nsobject_cls.is_null() {
        return std::ptr::null_mut();
    }

    // Register the class only once (guard against double-registration).
    let existing = objc_getClass(b"CLXMenuDelegate\0".as_ptr() as *const _);
    let delegate_cls = if existing.is_null() {
        let new_cls =
            objc_allocateClassPair(nsobject_cls, b"CLXMenuDelegate\0".as_ptr() as *const _, 0);
        if new_cls.is_null() {
            return std::ptr::null_mut();
        }

        // menuWillOpen: — called on main thread before menu is shown.
        unsafe extern "C" fn menu_will_open(
            _this: *mut c_void,
            _cmd: *mut c_void,
            _menu: *mut c_void,
        ) {
            refresh_mic_item();
        }
        let proto = objc_getProtocol(b"NSMenuDelegate\0".as_ptr() as *const _);
        if !proto.is_null() {
            class_addProtocol(new_cls, proto);
        }
        class_addMethod(
            new_cls,
            sel(b"menuWillOpen:\0"),
            menu_will_open as *const c_void,
            b"v@:@\0".as_ptr(),
        );
        objc_registerClassPair(new_cls);
        new_cls
    } else {
        existing
    };

    // [[CLXMenuDelegate alloc] init]
    let obj = {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(delegate_cls, sel(b"alloc\0"))
    };
    {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(obj, sel(b"init\0"))
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Create the NSStatusItem with a white icon and a "Quit" menu.
///
/// Must be called on the main thread before entering the CFRunLoop.
pub fn setup_tray() {
    unsafe {
        // Ensure NSApplication is initialized (required for AppKit/NSStatusBar).
        let nsapp_cls = cls(b"NSApplication\0");
        let app = msg0(nsapp_cls, sel(b"sharedApplication\0"));

        // Set activation policy to Accessory (no dock icon, but allows menu bar).
        // NSApplicationActivationPolicyAccessory = 1
        let sel_policy = sel(b"setActivationPolicy:\0");
        let f: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool =
            std::mem::transmute(objc_msgSend as *const ());
        f(app, sel_policy, 1); // Accessory

        // NSStatusBar *bar = [NSStatusBar systemStatusBar];
        let bar_cls = cls(b"NSStatusBar\0");
        let bar = msg0(bar_cls, sel(b"systemStatusBar\0"));
        if bar.is_null() {
            eprintln!("[CLX] tray: failed to get systemStatusBar");
            return;
        }

        // NSStatusItem *item = [bar statusItemWithLength:NSVariableStatusItemLength];
        // NSVariableStatusItemLength = -1.0
        let sel_item = sel(b"statusItemWithLength:\0");
        let item: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
                std::mem::transmute(objc_msgSend as *const ());
            f(bar, sel_item, -1.0_f64) // NSVariableStatusItemLength
        };
        if item.is_null() {
            eprintln!("[CLX] tray: failed to create NSStatusItem");
            return;
        }

        // Retain the item to prevent it from being deallocated.
        msg0(item, sel(b"retain\0"));
        STATUS_ITEM.store(item, Ordering::Release);

        // Set the initial icon (white = inactive).
        let img = nsimage_from_bytes(ICON_WHITE);
        if !img.is_null() {
            // [item.button setImage:img];
            let button = msg0(item, sel(b"button\0"));
            if !button.is_null() {
                msg1_ptr(button, sel(b"setImage:\0"), img);
            }
        }

        // ── Build menu ──────────────────────────────────────────────────────

        // NSMenu *menu = [[NSMenu alloc] init];
        let menu_cls = cls(b"NSMenu\0");
        let menu = msg0(msg0(menu_cls, sel(b"alloc\0")), sel(b"init\0"));

        let menuitem_cls = cls(b"NSMenuItem\0");
        let sel_init_item = sel(b"initWithTitle:action:keyEquivalent:\0");

        // ── "Preferences…" menu item ────────────────────────────────────
        let prefs_alloc = msg0(menuitem_cls, sel(b"alloc\0"));
        let prefs_title = nsstring("Preferences\u{2026}");
        let prefs_action = sel(b"openPrefs:\0");
        let prefs_key = nsstring(",");
        let prefs_item: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(
                prefs_alloc,
                sel_init_item,
                prefs_title,
                prefs_action,
                prefs_key,
            )
        };

        // Set the target to our CLXPrefsActionTarget instance.
        let action_target = crate::prefs::get_action_target();
        if !action_target.is_null() {
            msg1_ptr(prefs_item, sel(b"setTarget:\0"), action_target);
        }

        // [menu addItem:prefsItem];
        msg1_ptr(menu, sel(b"addItem:\0"), prefs_item);

        // ── "Restart" menu item ─────────────────────────────────────────
        let restart_alloc = msg0(menuitem_cls, sel(b"alloc\0"));
        let restart_title = nsstring("Restart");
        let restart_action = sel(b"restartApp:\0");
        let restart_key = nsstring("r");
        let restart_item: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(
                restart_alloc,
                sel_init_item,
                restart_title,
                restart_action,
                restart_key,
            )
        };
        if !action_target.is_null() {
            msg1_ptr(restart_item, sel(b"setTarget:\0"), action_target);
        }
        msg1_ptr(menu, sel(b"addItem:\0"), restart_item);

        // ── "Voice Recordings…" menu item ───────────────────────────────
        let voice_alloc = msg0(menuitem_cls, sel(b"alloc\0"));
        let voice_title = nsstring("Voice Recordings\u{2026}");
        let voice_action = sel(b"openVoiceFolder:\0");
        let voice_key = nsstring("");
        let voice_item: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(
                voice_alloc,
                sel_init_item,
                voice_title,
                voice_action,
                voice_key,
            )
        };
        if !action_target.is_null() {
            msg1_ptr(voice_item, sel(b"setTarget:\0"), action_target);
        }
        msg1_ptr(menu, sel(b"addItem:\0"), voice_item);

        // ── "Mic: Standard…" menu item (label updated dynamically) ─────
        let mic_alloc = msg0(menuitem_cls, sel(b"alloc\0"));
        let mic_title = nsstring("Mic\u{2026}"); // placeholder; refreshed on menuWillOpen:
        let mic_action = sel(b"showMicPicker:\0");
        let mic_key = nsstring("");
        let mic_item: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(mic_alloc, sel_init_item, mic_title, mic_action, mic_key)
        };
        if !action_target.is_null() {
            msg1_ptr(mic_item, sel(b"setTarget:\0"), action_target);
        }
        msg0(mic_item, sel(b"retain\0"));
        MIC_ITEM.store(mic_item, Ordering::Release);
        msg1_ptr(menu, sel(b"addItem:\0"), mic_item);

        // Set delegate so menuWillOpen: fires before the menu is shown.
        let delegate = create_menu_delegate();
        if !delegate.is_null() {
            msg0(delegate, sel(b"retain\0"));
            msg1_ptr(menu, sel(b"setDelegate:\0"), delegate);
        }
        // Populate label immediately so first open isn't blank.
        refresh_mic_item();

        // ── Separator ───────────────────────────────────────────────────
        let separator = msg0(menuitem_cls, sel(b"separatorItem\0"));
        msg1_ptr(menu, sel(b"addItem:\0"), separator);

        // ── "Quit CapsLockX" menu item ──────────────────────────────────
        let quit_alloc = msg0(menuitem_cls, sel(b"alloc\0"));
        let title = nsstring("Quit CapsLockX");
        let action = sel(b"terminate:\0");
        let key_equiv = nsstring("q");
        let quit_item: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(quit_alloc, sel_init_item, title, action, key_equiv)
        };

        // Set the target to NSApplication.sharedApplication so terminate: is sent there.
        let nsapp_cls = cls(b"NSApplication\0");
        let shared_app = msg0(nsapp_cls, sel(b"sharedApplication\0"));
        if !shared_app.is_null() {
            msg1_ptr(quit_item, sel(b"setTarget:\0"), shared_app);
        }

        // [menu addItem:quitItem];
        msg1_ptr(menu, sel(b"addItem:\0"), quit_item);

        // [item setMenu:menu];
        msg1_ptr(item, sel(b"setMenu:\0"), menu);

        eprintln!("[CLX] tray: status bar icon installed");
    }
}

/// Update the tray icon to reflect whether CapsLockX is active.
///
/// Safe to call from any thread — the actual NSImage swap is dispatched
/// to the main queue since AppKit is not thread-safe.
pub fn update_tray_icon(active: bool) {
    let item = STATUS_ITEM.load(Ordering::Acquire);
    if item.is_null() {
        return;
    }

    unsafe {
        extern "C" {
            fn dispatch_async_f(
                queue: *mut c_void,
                context: *mut c_void,
                work: extern "C" fn(*mut c_void),
            );
            fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
        }
        const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

        extern "C" fn set_active_icon(_ctx: *mut c_void) {
            unsafe {
                let item = STATUS_ITEM.load(Ordering::Acquire);
                if item.is_null() {
                    return;
                }
                let img = nsimage_from_bytes_ex(ICON_BLUE, false); // not template — show blue color
                if img.is_null() {
                    return;
                }
                let button = msg0(item, sel(b"button\0"));
                if !button.is_null() {
                    msg1_ptr(button, sel(b"setImage:\0"), img);
                }
            }
        }

        extern "C" fn set_inactive_icon(_ctx: *mut c_void) {
            unsafe {
                let item = STATUS_ITEM.load(Ordering::Acquire);
                if item.is_null() {
                    return;
                }
                let img = nsimage_from_bytes(ICON_WHITE);
                if img.is_null() {
                    return;
                }
                let button = msg0(item, sel(b"button\0"));
                if !button.is_null() {
                    msg1_ptr(button, sel(b"setImage:\0"), img);
                }
            }
        }

        // dispatch_get_main_queue() is a C macro that returns &_dispatch_main_q.
        // _dispatch_main_q is the dispatch_queue_t itself (an object).
        // dlsym returns the address OF the global, so we need to read through it.
        let sym = dlsym(RTLD_DEFAULT, b"_dispatch_main_q\0".as_ptr() as *const _);
        if sym.is_null() {
            return;
        }
        // The symbol IS the dispatch_queue_t object (dispatch_queue_t is a pointer type).
        let queue = sym;
        let work = if active {
            set_active_icon
        } else {
            set_inactive_icon
        };
        dispatch_async_f(queue, std::ptr::null_mut(), work);
    }
}

// ── PTT (push-to-talk) state indicator ────────────────────────────────────
// Reuses the NSStatusItem's `title` alongside the image so we don't fight
// the existing active/inactive icon switching. Single glyph per state.

static PTT_STATE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Set the PTT tray glyph. 0=Idle, 1=Recording, 2=Processing, 3=NoteMode.
pub fn set_ptt_tray_glyph(state: u8) {
    use std::sync::atomic::Ordering;
    if PTT_STATE.swap(state, Ordering::AcqRel) == state {
        return; // no change — skip dispatch
    }
    unsafe {
        extern "C" {
            fn dispatch_async_f(
                queue: *mut c_void,
                context: *mut c_void,
                work: extern "C" fn(*mut c_void),
            );
            fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
        }
        const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

        extern "C" fn apply(_ctx: *mut c_void) {
            unsafe {
                let item = STATUS_ITEM.load(Ordering::Acquire);
                if item.is_null() {
                    return;
                }
                let button = msg0(item, sel(b"button\0"));
                if button.is_null() {
                    return;
                }
                let glyph = match PTT_STATE.load(Ordering::Acquire) {
                    1 => "🎤",
                    2 => "⋯",
                    3 => "📝",
                    _ => "",
                };
                msg1_ptr(button, sel(b"setTitle:\0"), nsstring(glyph));
            }
        }
        let sym = dlsym(RTLD_DEFAULT, b"_dispatch_main_q\0".as_ptr() as *const _);
        if sym.is_null() {
            return;
        }
        dispatch_async_f(sym, std::ptr::null_mut(), apply);
    }
}
