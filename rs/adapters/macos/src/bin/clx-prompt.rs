//! Standalone prompt dialog for CapsLockX brainstorm.
//!
//! Runs as a separate process so the CGEventTap hook in the main process
//! doesn't intercept keyboard events meant for the text field.
//!
//! Usage: clx-prompt <title> <message> <prefill>
//! Output: prints "[KEEP]\n<text>" or "<text>" to stdout, exits 0.
//!         Exits with code 1 if cancelled.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
}

extern "C" {
    fn dispatch_async_f(queue: *mut c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
    fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
}

unsafe fn main_queue() -> *mut c_void {
    dlsym(-2isize as *mut c_void, b"_dispatch_main_q\0".as_ptr() as *const _)
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect { x: f64, y: f64, w: f64, h: f64 }

unsafe fn sel(name: &[u8]) -> *mut c_void { sel_registerName(name.as_ptr() as *const _) }
unsafe fn cls(name: &[u8]) -> *mut c_void { objc_getClass(name.as_ptr() as *const _) }
unsafe fn msg0(obj: *mut c_void, s: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s)
}
unsafe fn msg1(obj: *mut c_void, s: *mut c_void, a: *mut c_void) -> *mut c_void {
    let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s, a)
}
unsafe fn nsstring(s: &str) -> *mut c_void {
    let cstr = std::ffi::CString::new(s).unwrap();
    let f: extern "C" fn(*mut c_void, *mut c_void, *const std::ffi::c_char) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
    f(cls(b"NSString\0"), sel(b"stringWithUTF8String:\0"), cstr.as_ptr())
}
unsafe fn set_bool(obj: *mut c_void, s: *mut c_void, v: bool) {
    let f: extern "C" fn(*mut c_void, *mut c_void, bool) = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s, v);
}
unsafe fn set_f64(obj: *mut c_void, s: *mut c_void, v: f64) {
    let f: extern "C" fn(*mut c_void, *mut c_void, f64) = std::mem::transmute(objc_msgSend as *const ());
    f(obj, s, v);
}

// ── Global state ─────────────────────────────────────────────────────────────

static DONE: AtomicBool = AtomicBool::new(false);
static CANCELLED: AtomicBool = AtomicBool::new(false);

static mut WINDOW_PTR: *mut c_void = std::ptr::null_mut();
static mut TV_PTR: *mut c_void = std::ptr::null_mut();
static mut CHECKBOX_PTR: *mut c_void = std::ptr::null_mut();

// ── Args stored for main-thread callback ─────────────────────────────────────

static mut ARGS_TITLE: Option<String> = None;
static mut ARGS_MESSAGE: Option<String> = None;
static mut ARGS_PREFILL: Option<String> = None;
static mut ARGS_CURSOR_POS: usize = 0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let title = args.get(1).cloned().unwrap_or_else(|| "CapsLockX Brainstorm".into());
    let message = args.get(2).cloned().unwrap_or_default();
    let prefill = args.get(3).cloned().unwrap_or_default();

    // Format: clipboard text at top, then "\n---\n\n===" with cursor between --- and ===.
    let formatted = if prefill.is_empty() {
        "---\n\n===".to_string()
    } else {
        format!("{}\n---\n\n===", prefill)
    };
    let cursor_pos = formatted.rfind("\n\n===").map(|p| p + 1).unwrap_or(formatted.len());

    unsafe {
        ARGS_TITLE = Some(title);
        ARGS_MESSAGE = Some(message);
        ARGS_PREFILL = Some(formatted);
        ARGS_CURSOR_POS = cursor_pos;
    }

    // Kick off NSApplication on main thread.
    unsafe {
        let nsapp_cls = cls(b"NSApplication\0");
        let app = msg0(nsapp_cls, sel(b"sharedApplication\0"));

        // Set activation policy to regular (shows in dock briefly).
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool = std::mem::transmute(objc_msgSend as *const ());
            f(app, sel(b"setActivationPolicy:\0"), 0); // NSApplicationActivationPolicyRegular
        }

        dispatch_async_f(main_queue(), std::ptr::null_mut(), create_prompt);

        // Run the app event loop.
        let f: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
        f(app, sel(b"run\0"));
    }

    // After app.run() returns (it won't normally — we call terminate: instead).
    if CANCELLED.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
}

// ── keyDown handler ──────────────────────────────────────────────────────────

unsafe extern "C" fn prompt_tv_key_down(
    this: *mut c_void, _cmd: *mut c_void, event: *mut c_void,
) {
    let f_u16: extern "C" fn(*mut c_void, *mut c_void) -> u16 = std::mem::transmute(objc_msgSend as *const ());
    let key_code = f_u16(event, sel(b"keyCode\0"));
    let f_u64: extern "C" fn(*mut c_void, *mut c_void) -> u64 = std::mem::transmute(objc_msgSend as *const ());
    let flags = f_u64(event, sel(b"modifierFlags\0"));
    let shift = flags & (1 << 17) != 0;

    if key_code == 53 {
        // ESC → Cancel
        do_cancel();
    } else if key_code == 36 && !shift {
        // Enter → Send
        do_send();
    } else {
        // Forward to super (NSTextView).
        extern "C" { fn objc_msgSendSuper(sup: *mut c_void, sel: *mut c_void, ...) -> *mut c_void; }
        #[repr(C)] struct ObjcSuper { receiver: *mut c_void, super_class: *mut c_void }
        let mut sup = ObjcSuper { receiver: this, super_class: cls(b"NSTextView\0") };
        let f: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void
            = std::mem::transmute(objc_msgSendSuper as *const ());
        f(&mut sup as *mut _ as *mut _, sel(b"keyDown:\0"), event);
    }
}

// ── Button actions ───────────────────────────────────────────────────────────

unsafe extern "C" fn send_action(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    do_send();
}
unsafe extern "C" fn cancel_action(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    do_cancel();
}

fn do_send() {
    if DONE.swap(true, Ordering::SeqCst) { return; }
    unsafe {
        let tv = TV_PTR;
        if tv.is_null() { std::process::exit(1); }

        let ns_str = msg0(tv, sel(b"string\0"));
        let cstr: *const std::ffi::c_char = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char = std::mem::transmute(objc_msgSend as *const ());
            f(ns_str, sel(b"UTF8String\0"))
        };

        let keep_history = {
            let cb = CHECKBOX_PTR;
            if !cb.is_null() {
                let f: extern "C" fn(*mut c_void, *mut c_void) -> i64 = std::mem::transmute(objc_msgSend as *const ());
                f(cb, sel(b"state\0")) == 1
            } else { false }
        };

        if !cstr.is_null() {
            let text = std::ffi::CStr::from_ptr(cstr).to_string_lossy();
            if keep_history {
                print!("[KEEP]\n{}", text);
            } else {
                print!("{}", text);
            }
        }

        let panel = WINDOW_PTR;
        if !panel.is_null() {
            msg1(panel, sel(b"orderOut:\0"), std::ptr::null_mut());
        }

        // Terminate the app.
        let app = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        msg1(app, sel(b"terminate:\0"), std::ptr::null_mut());
    }
}

fn do_cancel() {
    if DONE.swap(true, Ordering::SeqCst) { return; }
    CANCELLED.store(true, Ordering::Relaxed);
    unsafe {
        let panel = WINDOW_PTR;
        if !panel.is_null() {
            msg1(panel, sel(b"orderOut:\0"), std::ptr::null_mut());
        }
        let app = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        msg1(app, sel(b"terminate:\0"), std::ptr::null_mut());
    }
}

// ── Window creation (runs on main thread) ────────────────────────────────────

extern "C" fn create_prompt(_: *mut c_void) {
    unsafe {
        // Register CLXPromptTextView and CLXPromptAction classes.
        extern "C" {
            fn objc_allocateClassPair(sup: *mut c_void, name: *const std::ffi::c_char, extra: usize) -> *mut c_void;
            fn objc_registerClassPair(cls_ptr: *mut c_void);
            fn class_addMethod(cls_ptr: *mut c_void, sel: *mut c_void, imp: *const c_void, types: *const std::ffi::c_char) -> bool;
        }

        let tv_super = cls(b"NSTextView\0");
        let tv_cls = objc_allocateClassPair(tv_super, b"CLXPromptTextView2\0".as_ptr() as *const _, 0);
        if !tv_cls.is_null() {
            class_addMethod(tv_cls, sel(b"keyDown:\0"), prompt_tv_key_down as *const c_void, b"v@:@\0".as_ptr() as *const _);
            objc_registerClassPair(tv_cls);
        }

        let obj_super = cls(b"NSObject\0");
        let act_cls = objc_allocateClassPair(obj_super, b"CLXPromptAction2\0".as_ptr() as *const _, 0);
        if !act_cls.is_null() {
            class_addMethod(act_cls, sel(b"cancelPrompt:\0"), cancel_action as *const c_void, b"v@:@\0".as_ptr() as *const _);
            class_addMethod(act_cls, sel(b"sendPrompt:\0"), send_action as *const c_void, b"v@:@\0".as_ptr() as *const _);
            objc_registerClassPair(act_cls);
        }

        let title = ARGS_TITLE.as_deref().unwrap_or("Prompt");
        let message = ARGS_MESSAGE.as_deref().unwrap_or("");
        let formatted = ARGS_PREFILL.as_deref().unwrap_or("");
        let cursor_pos = ARGS_CURSOR_POS;

        let screen = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
        let sf: NSRect = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect = std::mem::transmute(objc_msgSend as *const ());
            f(screen, sel(b"frame\0"))
        };
        let pw = 520.0_f64;
        let ph = 320.0_f64;
        let rect = NSRect { x: (sf.w - pw) / 2.0, y: (sf.h - ph) / 2.0 + 100.0, w: pw, h: ph };

        // titled(1) + closable(2) + resizable(8)
        let style: u64 = 1 | 2 | 8;
        let panel = msg0(cls(b"NSWindow\0"), sel(b"alloc\0"));
        let panel: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(panel, sel(b"initWithContentRect:styleMask:backing:defer:\0"), rect, style, 2, false)
        };

        msg1(panel, sel(b"setTitle:\0"), nsstring(title));
        set_f64(panel, sel(b"setAlphaValue:\0"), 0.95);
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
            f(panel, sel(b"setLevel:\0"), 8); // NSModalPanelWindowLevel
        }
        set_bool(panel, sel(b"setHidesOnDeactivate:\0"), false);
        set_bool(panel, sel(b"setReleasedWhenClosed:\0"), false);
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
            f(panel, sel(b"setSharingType:\0"), 0);
        }

        // Dark background
        let bg = {
            let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.12, 0.12, 0.18, 0.98)
        };
        msg1(panel, sel(b"setBackgroundColor:\0"), bg);

        // Container view
        let cv_rect = NSRect { x: 0.0, y: 0.0, w: pw, h: ph - 28.0 };
        let container = msg0(cls(b"NSView\0"), sel(b"alloc\0"));
        let container: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(container, sel(b"initWithFrame:\0"), cv_rect)
        };

        // Info label at top
        let info_rect = NSRect { x: 12.0, y: cv_rect.h - 50.0, w: pw - 24.0, h: 44.0 };
        let info_label = msg0(cls(b"NSTextField\0"), sel(b"alloc\0"));
        let info_label: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(info_label, sel(b"initWithFrame:\0"), info_rect)
        };
        msg1(info_label, sel(b"setStringValue:\0"), nsstring(&format!("{}\nEnter = Send | Shift+Enter = new line", message)));
        set_bool(info_label, sel(b"setBezeled:\0"), false);
        set_bool(info_label, sel(b"setDrawsBackground:\0"), false);
        set_bool(info_label, sel(b"setEditable:\0"), false);
        set_bool(info_label, sel(b"setSelectable:\0"), false);
        let text_color = {
            let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.7, 0.7, 0.8, 1.0)
        };
        msg1(info_label, sel(b"setTextColor:\0"), text_color);
        let small_font = {
            let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 11.0)
        };
        msg1(info_label, sel(b"setFont:\0"), small_font);
        msg1(container, sel(b"addSubview:\0"), info_label);

        // Buttons
        let btn_y = 8.0_f64;
        let btn_h = 28.0_f64;

        // Send button
        let send_rect = NSRect { x: pw - 100.0, y: btn_y, w: 80.0, h: btn_h };
        let send_btn = msg0(cls(b"NSButton\0"), sel(b"alloc\0"));
        let send_btn: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(send_btn, sel(b"initWithFrame:\0"), send_rect)
        };
        msg1(send_btn, sel(b"setTitle:\0"), nsstring("Send"));
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
            f(send_btn, sel(b"setBezelStyle:\0"), 1);
        }
        msg1(send_btn, sel(b"setKeyEquivalent:\0"), nsstring(""));

        let action_cls = cls(b"CLXPromptAction2\0");
        let action_target = if !action_cls.is_null() {
            let t = msg0(msg0(action_cls, sel(b"alloc\0")), sel(b"init\0"));
            msg0(t, sel(b"retain\0"));
            t
        } else { std::ptr::null_mut() };

        msg1(send_btn, sel(b"setTarget:\0"), action_target);
        msg1(send_btn, sel(b"setAction:\0"), sel(b"sendPrompt:\0"));

        // Cancel button
        let cancel_rect = NSRect { x: pw - 190.0, y: btn_y, w: 80.0, h: btn_h };
        let cancel_btn = msg0(cls(b"NSButton\0"), sel(b"alloc\0"));
        let cancel_btn: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(cancel_btn, sel(b"initWithFrame:\0"), cancel_rect)
        };
        msg1(cancel_btn, sel(b"setTitle:\0"), nsstring("Cancel"));
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
            f(cancel_btn, sel(b"setBezelStyle:\0"), 1);
        }
        msg1(cancel_btn, sel(b"setTarget:\0"), action_target);
        msg1(cancel_btn, sel(b"setAction:\0"), sel(b"cancelPrompt:\0"));

        msg1(container, sel(b"addSubview:\0"), send_btn);
        msg1(container, sel(b"addSubview:\0"), cancel_btn);

        // "Keep histories" checkbox
        let cb_rect = NSRect { x: 12.0, y: btn_y + 2.0, w: 140.0, h: btn_h };
        let checkbox = msg0(cls(b"NSButton\0"), sel(b"alloc\0"));
        let checkbox: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(checkbox, sel(b"initWithFrame:\0"), cb_rect)
        };
        let hist_count = {
            let path = dirs::config_dir().unwrap_or_default().join("CapsLockX").join("brainstorm_history.json");
            std::fs::read_to_string(&path).ok()
                .and_then(|d| serde_json::from_str::<Vec<serde_json::Value>>(&d).ok())
                .map(|v| v.len()).unwrap_or(0)
        };
        let keep_pref = dirs::config_dir().unwrap_or_default()
            .join("CapsLockX").join("brainstorm_keep_history").exists();

        msg1(checkbox, sel(b"setTitle:\0"), nsstring(&format!("Keep histories ({})", hist_count)));
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
            f(checkbox, sel(b"setButtonType:\0"), 3); // NSSwitchButton
        }
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
            f(checkbox, sel(b"setState:\0"), if keep_pref { 1 } else { 0 });
        }
        msg1(container, sel(b"addSubview:\0"), checkbox);
        CHECKBOX_PTR = checkbox;

        // ScrollView + TextView
        let tv_rect = NSRect { x: 12.0, y: btn_y + btn_h + 8.0, w: pw - 24.0, h: cv_rect.h - 50.0 - btn_h - 24.0 };
        let scroll = msg0(cls(b"NSScrollView\0"), sel(b"alloc\0"));
        let scroll: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(scroll, sel(b"initWithFrame:\0"), tv_rect)
        };
        set_bool(scroll, sel(b"setHasVerticalScroller:\0"), true);

        let tv_cls_obj = {
            let c = cls(b"CLXPromptTextView2\0");
            if c.is_null() { cls(b"NSTextView\0") } else { c }
        };
        let text_view = msg0(tv_cls_obj, sel(b"alloc\0"));
        let inner_rect = NSRect { x: 0.0, y: 0.0, w: tv_rect.w, h: tv_rect.h };
        let text_view: *mut c_void = {
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(text_view, sel(b"initWithFrame:\0"), inner_rect)
        };
        set_bool(text_view, sel(b"setEditable:\0"), true);
        set_bool(text_view, sel(b"setRichText:\0"), false);

        msg1(text_view, sel(b"setString:\0"), nsstring(formatted));
        {
            #[repr(C)] #[derive(Clone, Copy)] struct NSRange { location: usize, length: usize }
            let range = NSRange { location: cursor_pos, length: 0 };
            let f: extern "C" fn(*mut c_void, *mut c_void, NSRange) = std::mem::transmute(objc_msgSend as *const ());
            f(text_view, sel(b"setSelectedRange:\0"), range);
            f(text_view, sel(b"scrollRangeToVisible:\0"), range);
        }

        let font = {
            let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
            f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 13.0)
        };
        msg1(text_view, sel(b"setFont:\0"), font);

        msg1(scroll, sel(b"setDocumentView:\0"), text_view);
        msg1(container, sel(b"addSubview:\0"), scroll);
        msg1(panel, sel(b"setContentView:\0"), container);

        msg0(panel, sel(b"retain\0"));
        msg1(panel, sel(b"makeKeyAndOrderFront:\0"), std::ptr::null_mut());
        msg1(panel, sel(b"makeFirstResponder:\0"), text_view);

        // Activate this app
        let nsapp = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
        {
            let f: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool = std::mem::transmute(objc_msgSend as *const ());
            f(nsapp, sel(b"activateIgnoringOtherApps:\0"), 1);
        }

        WINDOW_PTR = panel;
        TV_PTR = text_view;
    }
}
