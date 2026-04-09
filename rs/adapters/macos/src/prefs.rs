//! Preferences window using WKWebView via raw Objective-C FFI.
//!
//! Creates an NSWindow containing a WKWebView that loads an embedded HTML
//! preferences UI. Communication between JS and Rust uses
//! WKScriptMessageHandler (webkit.messageHandlers.clx.postMessage).

use std::ffi::{c_void, CString};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::config_store;
use crate::hook::ENGINE;

// ── Embedded HTML ────────────────────────────────────────────────────────────

static PREFS_HTML: &str = include_str!("prefs_html.html");

// ── Global window reference (to prevent deallocation and reuse) ──────────────

static PREFS_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static WEBVIEW_REF: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

// ── Objective-C runtime FFI ──────────────────────────────────────────────────

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_getProtocol(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const std::ffi::c_char,
        extra_bytes: usize,
    ) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(
        cls: *mut c_void,
        sel: *mut c_void,
        imp: *const c_void,
        types: *const std::ffi::c_char,
    ) -> bool;
    fn class_addProtocol(cls: *mut c_void, protocol: *mut c_void) -> bool;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
}

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
unsafe fn msg1(obj: *mut c_void, sel: *mut c_void, arg: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(obj, sel, arg)
}

/// Helper: send a two-arg (id, id) message returning id.
#[allow(dead_code)]
unsafe fn msg2(
    obj: *mut c_void,
    sel: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(obj, sel, a, b)
}

/// Create an NSString from a Rust string slice.
unsafe fn nsstring(s: &str) -> *mut c_void {
    let cls_str = cls(b"NSString\0");
    let sel_utf8 = sel(b"stringWithUTF8String:\0");
    let cstr = CString::new(s).unwrap();
    let f: extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls_str, sel_utf8, cstr.as_ptr())
}

/// Extract UTF-8 string from an NSString.
unsafe fn nsstring_to_string(ns: *mut c_void) -> Option<String> {
    if ns.is_null() {
        return None;
    }
    let sel_utf8 = sel(b"UTF8String\0");
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char =
        std::mem::transmute(objc_msgSend as *const ());
    let cstr = f(ns, sel_utf8);
    if cstr.is_null() {
        return None;
    }
    Some(
        std::ffi::CStr::from_ptr(cstr)
            .to_string_lossy()
            .into_owned(),
    )
}

// ── NSRect for FFI ───────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

// ── WKScriptMessageHandler callback ──────────────────────────────────────────

/// The IMP for `userContentController:didReceiveScriptMessage:`.
/// Signature: void callback(id self, SEL _cmd, id userContentController, id message)
unsafe extern "C" fn handle_script_message(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _controller: *mut c_void,
    message: *mut c_void,
) {
    eprintln!("[CLX] prefs: received script message from JS");

    // message.body is an NSString (we send JSON strings from JS)
    let body = msg0(message, sel(b"body\0"));
    let body_str = match nsstring_to_string(body) {
        Some(s) => s,
        None => {
            eprintln!("[CLX] prefs: failed to extract message body");
            return;
        }
    };

    eprintln!("[CLX] prefs: message body = {}", body_str);

    // Parse JSON
    let parsed: serde_json::Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[CLX] prefs: JSON parse error: {}", e);
            return;
        }
    };

    let cmd = parsed.get("cmd").and_then(|v| v.as_str()).unwrap_or("");

    match cmd {
        "get_config" => {
            eprintln!("[CLX] prefs: get_config called");
            let cfg = ENGINE.get_config();
            let full = config_store::FullConfig::from_clx_config(&cfg);
            let json = serde_json::to_string(&full).unwrap_or_default();

            // Escape single quotes and backslashes for JS string literal
            let escaped = json.replace('\\', "\\\\").replace('\'', "\\'");
            let js = format!("window.handleGetConfig('{}')", escaped);

            eval_js(&js);
        }
        "set_config" => {
            eprintln!("[CLX] prefs: set_config called");
            if let Some(cfg_val) = parsed.get("cfg") {
                match serde_json::from_value::<config_store::FullConfig>(cfg_val.clone()) {
                    Ok(full) => {
                        config_store::save(&full);
                        let clx_cfg = full.into_clx_config();
                        ENGINE.update_config(clx_cfg);
                        eprintln!("[CLX] prefs: config saved and applied");
                        eval_js("window.handleSetConfig('{}')");
                    }
                    Err(e) => {
                        eprintln!("[CLX] prefs: failed to deserialize config: {}", e);
                    }
                }
            }
        }
        _ => {
            eprintln!("[CLX] prefs: unknown command: {}", cmd);
        }
    }
}

/// Evaluate JavaScript on the webview (must be called from main thread or dispatched).
unsafe fn eval_js(js: &str) {
    let webview = WEBVIEW_REF.load(Ordering::Acquire);
    if webview.is_null() {
        eprintln!("[CLX] prefs: eval_js called but webview is null");
        return;
    }

    let js_str = nsstring(js);
    let sel_eval = sel(b"evaluateJavaScript:completionHandler:\0");
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(webview, sel_eval, js_str, ptr::null_mut());
}

// ── Runtime class registration ───────────────────────────────────────────────

/// Class name for the message handler — must be unique and only registered once.
static HANDLER_CLASS_REGISTERED: std::sync::Once = std::sync::Once::new();
static HANDLER_CLASS_NAME: &[u8] = b"CLXScriptMessageHandler\0";

/// Class name for the prefs action target.
static ACTION_CLASS_REGISTERED: std::sync::Once = std::sync::Once::new();
static ACTION_CLASS_NAME: &[u8] = b"CLXPrefsActionTarget\0";

/// Register the CLXScriptMessageHandler class at runtime (once).
unsafe fn ensure_handler_class() {
    HANDLER_CLASS_REGISTERED.call_once(|| {
        let superclass = cls(b"NSObject\0");
        let new_cls = objc_allocateClassPair(
            superclass,
            HANDLER_CLASS_NAME.as_ptr() as *const _,
            0,
        );
        if new_cls.is_null() {
            eprintln!("[CLX] prefs: failed to allocate CLXScriptMessageHandler class");
            return;
        }

        // Add WKScriptMessageHandler protocol
        let protocol = objc_getProtocol(b"WKScriptMessageHandler\0".as_ptr() as *const _);
        if !protocol.is_null() {
            class_addProtocol(new_cls, protocol);
        }

        // Add the method: userContentController:didReceiveScriptMessage:
        let sel_method = sel(b"userContentController:didReceiveScriptMessage:\0");
        let added = class_addMethod(
            new_cls,
            sel_method,
            handle_script_message as *const c_void,
            b"v@:@@\0".as_ptr() as *const _,
        );
        if !added {
            eprintln!("[CLX] prefs: failed to add method to CLXScriptMessageHandler");
        }

        objc_registerClassPair(new_cls);
        eprintln!("[CLX] prefs: registered CLXScriptMessageHandler class");
    });
}

/// Callback for the "Preferences..." menu action target.
unsafe extern "C" fn action_open_prefs(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
) {
    open_preferences();
}

/// Action handler for "Voice Recordings" menu item — open the recordings folder in Finder.
unsafe extern "C" fn action_open_voice_folder(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
) {
    eprintln!("[CLX] opening voice recordings folder");
    if let Some(dir) = dirs::home_dir().map(|h| h.join(".capslockx").join("voice")) {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::process::Command::new("open").arg(&dir).spawn();
    }
}

/// Action handler for "Restart" menu item — spawn new process and exit.
/// Using spawn+exit instead of execv so macOS properly cleans up the old
/// NSStatusItem (execv leaves a ghost/transparent icon in the menu bar).
unsafe extern "C" fn action_restart(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
) {
    eprintln!("[CLX] restart requested via tray menu");
    let exe = std::env::current_exe().unwrap_or_default();
    let args: Vec<String> = std::env::args().collect();
    match std::process::Command::new(&exe)
        .args(&args[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            eprintln!("[CLX] restart: spawned new instance pid={}", child.id());
            // Exit cleanly so the OS removes the old NSStatusItem.
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("[CLX] restart: spawn failed ({})", e);
        }
    }
}

/// Register the CLXPrefsActionTarget class at runtime (once).
unsafe fn ensure_action_class() {
    ACTION_CLASS_REGISTERED.call_once(|| {
        let superclass = cls(b"NSObject\0");
        let new_cls = objc_allocateClassPair(
            superclass,
            ACTION_CLASS_NAME.as_ptr() as *const _,
            0,
        );
        if new_cls.is_null() {
            eprintln!("[CLX] prefs: failed to allocate CLXPrefsActionTarget class");
            return;
        }

        let sel_method = sel(b"openPrefs:\0");
        let added = class_addMethod(
            new_cls,
            sel_method,
            action_open_prefs as *const c_void,
            b"v@:@\0".as_ptr() as *const _,
        );
        if !added {
            eprintln!("[CLX] prefs: failed to add openPrefs: method");
        }

        let sel_restart = sel(b"restartApp:\0");
        let added = class_addMethod(
            new_cls,
            sel_restart,
            action_restart as *const c_void,
            b"v@:@\0".as_ptr() as *const _,
        );
        if !added {
            eprintln!("[CLX] prefs: failed to add restartApp: method");
        }

        let sel_voice = sel(b"openVoiceFolder:\0");
        let added = class_addMethod(
            new_cls,
            sel_voice,
            action_open_voice_folder as *const c_void,
            b"v@:@\0".as_ptr() as *const _,
        );
        if !added {
            eprintln!("[CLX] prefs: failed to add openVoiceFolder: method");
        }

        objc_registerClassPair(new_cls);
        eprintln!("[CLX] prefs: registered CLXPrefsActionTarget class");
    });
}

// ── Global action target instance (must stay alive) ──────────────────────────

static ACTION_TARGET: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Get or create the action target singleton.
pub unsafe fn get_action_target() -> *mut c_void {
    let existing = ACTION_TARGET.load(Ordering::Acquire);
    if !existing.is_null() {
        return existing;
    }

    ensure_action_class();

    let handler_cls = cls(ACTION_CLASS_NAME);
    if handler_cls.is_null() {
        eprintln!("[CLX] prefs: CLXPrefsActionTarget class not found");
        return ptr::null_mut();
    }
    let instance = msg0(msg0(handler_cls, sel(b"alloc\0")), sel(b"init\0"));
    msg0(instance, sel(b"retain\0"));
    ACTION_TARGET.store(instance, Ordering::Release);
    instance
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Create and show the preferences window, or bring it to front if already open.
pub fn open_preferences() {
    eprintln!("[CLX] prefs: open_preferences called");

    unsafe {
        // If window already exists, just bring it to front.
        let existing = PREFS_WINDOW.load(Ordering::Acquire);
        if !existing.is_null() {
            eprintln!("[CLX] prefs: reusing existing window");
            msg1(existing, sel(b"makeKeyAndOrderFront:\0"), ptr::null_mut());

            // Also activate our app so the window comes to front
            let nsapp = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
            let f: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool =
                std::mem::transmute(objc_msgSend as *const ());
            f(nsapp, sel(b"activateIgnoringOtherApps:\0"), 1);

            // Reload config into the webview
            let cfg = ENGINE.get_config();
            let full = config_store::FullConfig::from_clx_config(&cfg);
            let json = serde_json::to_string(&full).unwrap_or_default();
            let escaped = json.replace('\\', "\\\\").replace('\'', "\\'");
            let js = format!("window.handleGetConfig('{}')", escaped);
            eval_js(&js);
            return;
        }

        eprintln!("[CLX] prefs: creating new preferences window");

        // Ensure the message handler class is registered.
        ensure_handler_class();

        // 1. Create WKWebViewConfiguration
        let wk_config_cls = cls(b"WKWebViewConfiguration\0");
        if wk_config_cls.is_null() {
            eprintln!("[CLX] prefs: WKWebViewConfiguration class not found (WebKit not linked?)");
            return;
        }
        let wk_config = msg0(msg0(wk_config_cls, sel(b"alloc\0")), sel(b"init\0"));
        if wk_config.is_null() {
            eprintln!("[CLX] prefs: failed to create WKWebViewConfiguration");
            return;
        }

        // 2. Get userContentController
        let ucc = msg0(wk_config, sel(b"userContentController\0"));
        if ucc.is_null() {
            eprintln!("[CLX] prefs: failed to get userContentController");
            return;
        }

        // 3. Create message handler instance
        let handler_cls = cls(HANDLER_CLASS_NAME);
        if handler_cls.is_null() {
            eprintln!("[CLX] prefs: CLXScriptMessageHandler class not found after registration");
            return;
        }
        let handler = msg0(msg0(handler_cls, sel(b"alloc\0")), sel(b"init\0"));
        if handler.is_null() {
            eprintln!("[CLX] prefs: failed to create message handler instance");
            return;
        }

        // 4. [ucc addScriptMessageHandler:handler name:@"clx"]
        let name = nsstring("clx");
        let sel_add = sel(b"addScriptMessageHandler:name:\0");
        let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(ucc, sel_add, handler, name);

        // 5. Create WKWebView with frame and configuration
        let rect = NSRect {
            x: 0.0,
            y: 0.0,
            w: 560.0,
            h: 640.0,
        };
        let wk_view_cls = cls(b"WKWebView\0");
        if wk_view_cls.is_null() {
            eprintln!("[CLX] prefs: WKWebView class not found");
            return;
        }
        let wk_alloc = msg0(wk_view_cls, sel(b"alloc\0"));
        let sel_init_frame = sel(b"initWithFrame:configuration:\0");
        let webview: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                NSRect,
                *mut c_void,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(wk_alloc, sel_init_frame, rect, wk_config)
        };
        if webview.is_null() {
            eprintln!("[CLX] prefs: failed to create WKWebView");
            return;
        }

        WEBVIEW_REF.store(webview, Ordering::Release);
        eprintln!("[CLX] prefs: WKWebView created");

        // 6. Load HTML string
        let html_ns = nsstring(PREFS_HTML);
        let sel_load = sel(b"loadHTMLString:baseURL:\0");
        let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        f(webview, sel_load, html_ns, ptr::null_mut());

        // 7. Create NSWindow
        // NSWindowStyleMaskTitled = 1, Closable = 2, Miniaturizable = 4, Resizable = 8
        let style_mask: u64 = 1 | 2 | 4 | 8;
        let backing: u64 = 2; // NSBackingStoreBuffered
        let window_cls = cls(b"NSWindow\0");
        let window_alloc = msg0(window_cls, sel(b"alloc\0"));
        let sel_init_window = sel(b"initWithContentRect:styleMask:backing:defer:\0");
        let window: *mut c_void = {
            let f: extern "C" fn(
                *mut c_void,
                *mut c_void,
                NSRect,
                u64,
                u64,
                bool,
            ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(window_alloc, sel_init_window, rect, style_mask, backing, false)
        };
        if window.is_null() {
            eprintln!("[CLX] prefs: failed to create NSWindow");
            return;
        }

        // 8. Configure window
        let title = nsstring("CapsLockX Preferences");
        msg1(window, sel(b"setTitle:\0"), title);
        msg1(window, sel(b"setContentView:\0"), webview);
        msg0(window, sel(b"center\0"));

        // setReleasedWhenClosed:NO — prevent deallocation when user clicks close
        let f_set_bool: extern "C" fn(*mut c_void, *mut c_void, bool) =
            std::mem::transmute(objc_msgSend as *const ());
        f_set_bool(window, sel(b"setReleasedWhenClosed:\0"), false);

        // Retain the window
        msg0(window, sel(b"retain\0"));
        PREFS_WINDOW.store(window, Ordering::Release);

        // 9. Show window
        msg1(window, sel(b"makeKeyAndOrderFront:\0"), ptr::null_mut());

        // Activate our app so the window is visible
        let nsapp = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        let f_activate: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool =
            std::mem::transmute(objc_msgSend as *const ());
        f_activate(nsapp, sel(b"activateIgnoringOtherApps:\0"), 1);

        eprintln!("[CLX] prefs: preferences window opened successfully");
    }
}
