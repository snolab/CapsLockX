//! Floating overlay for brainstorm AI responses.
//!
//! Shows a semi-transparent dark panel with scrollable text,
//! positioned at top-right of screen. Non-focusable (doesn't steal keyboard).

use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;

static WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static TEXT_VIEW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static OVERLAY_TEXT: Mutex<String> = Mutex::new(String::new());

// ── Position persistence ────────────────────────────────────────────────────

fn pos_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("CapsLockX")
        .join("brainstorm_overlay_pos.json")
}

fn load_pos() -> Option<(f64, f64)> {
    let data = std::fs::read_to_string(pos_path()).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;
    let x = v.get("x")?.as_f64()?;
    let y = v.get("y")?.as_f64()?;
    Some((x, y))
}

fn save_pos(x: f64, y: f64) {
    let path = pos_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = format!("{{\"x\":{},\"y\":{}}}", x, y);
    let _ = std::fs::write(path, json);
}

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
    // RTLD_DEFAULT is ((void *)(-2)) on macOS
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

extern "C" fn do_show(_: *mut c_void) { unsafe { ensure_window(); update_text_view(); show_window(); } }
extern "C" fn do_hide(_: *mut c_void) { unsafe { hide_window(); } }
extern "C" fn do_update(_: *mut c_void) { unsafe { ensure_window(); update_text_view(); show_window(); } }

pub fn show_overlay(text: &str) {
    *OVERLAY_TEXT.lock().unwrap() = text.to_string();
    unsafe { dispatch_async_f(main_queue(), std::ptr::null_mut(), do_update); }
}

pub fn hide_overlay() {
    unsafe { dispatch_async_f(main_queue(), std::ptr::null_mut(), do_hide); }
}

unsafe fn ensure_window() {
    if !WINDOW_PTR.load(Ordering::Relaxed).is_null() { return; }

    // Get screen size for positioning
    let screen = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
    let frame: NSRect = {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect = std::mem::transmute(objc_msgSend as *const ());
        f(screen, sel(b"frame\0"))
    };

    // Panel: 420px wide, 300px tall, top-right corner
    let panel_w = 420.0;
    let panel_h = 300.0;
    let panel_x = frame.w - panel_w - 20.0;
    let panel_y = frame.h - panel_h - 60.0; // below menu bar
    let rect = NSRect { x: panel_x, y: panel_y, w: panel_w, h: panel_h };

    // NSPanel (non-activating, floating)
    // styleMask: titled(1) + closable(2) + resizable(8) + nonActivatingPanel(1<<7) + utilityWindow(1<<4)
    let style: u64 = 1 | 2 | 8 | (1 << 4) | (1 << 7);
    let panel_cls = cls(b"NSPanel\0");
    let panel = msg0(panel_cls, sel(b"alloc\0"));
    let init_sel = sel(b"initWithContentRect:styleMask:backing:defer:\0");
    let panel: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(panel, init_sel, rect, style, 2/*buffered*/, false)
    };

    // Configure panel — non-focusable, click-through (like voice overlay).
    msg1(panel, sel(b"setTitle:\0"), nsstring("CapsLockX Brainstorm"));
    set_f64(panel, sel(b"setAlphaValue:\0"), 0.92);
    set_bool(panel, sel(b"setOpaque:\0"), false);
    set_bool(panel, sel(b"setIgnoresMouseEvents:\0"), true);
    set_bool(panel, sel(b"setHidesOnDeactivate:\0"), false);

    // Dark background
    let bg_color = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.12, 0.12, 0.18, 0.95)
    };
    msg1(panel, sel(b"setBackgroundColor:\0"), bg_color);

    // Floating level (above normal windows)
    {
        let f: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
        f(panel, sel(b"setLevel:\0"), 3); // NSFloatingWindowLevel
    }
    set_bool(panel, sel(b"setHidesOnDeactivate:\0"), false);
    set_bool(panel, sel(b"setReleasedWhenClosed:\0"), false);
    // Hide from screen sharing / screenshots (NSWindowSharingNone = 0)
    {
        let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
        f(panel, sel(b"setSharingType:\0"), 0);
    }

    // ScrollView + TextView
    let content_rect = NSRect { x: 0.0, y: 0.0, w: panel_w, h: panel_h - 28.0 }; // minus title bar
    let scroll = msg0(cls(b"NSScrollView\0"), sel(b"alloc\0"));
    let scroll: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(scroll, sel(b"initWithFrame:\0"), content_rect)
    };
    set_bool(scroll, sel(b"setHasVerticalScroller:\0"), true);

    let text_view = msg0(cls(b"NSTextView\0"), sel(b"alloc\0"));
    let text_view: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(text_view, sel(b"initWithFrame:\0"), content_rect)
    };
    set_bool(text_view, sel(b"setEditable:\0"), false);
    set_bool(text_view, sel(b"setSelectable:\0"), true);
    set_bool(text_view, sel(b"setRichText:\0"), false);

    // Font: system 14pt
    let font = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSFont\0"), sel(b"monospacedSystemFontOfSize:weight:\0"), 13.0)
    };
    msg1(text_view, sel(b"setFont:\0"), font);

    // Text color: light gray
    let text_color = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.80, 0.84, 0.96, 1.0)
    };
    msg1(text_view, sel(b"setTextColor:\0"), text_color);

    // Dark background for text view
    let tv_bg = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.09, 0.09, 0.14, 1.0)
    };
    msg1(text_view, sel(b"setBackgroundColor:\0"), tv_bg);
    set_bool(text_view, sel(b"setDrawsBackground:\0"), true);

    msg1(scroll, sel(b"setDocumentView:\0"), text_view);
    msg1(panel, sel(b"setContentView:\0"), scroll);

    msg0(panel, sel(b"retain\0"));
    WINDOW_PTR.store(panel, Ordering::Release);
    TEXT_VIEW_PTR.store(text_view, Ordering::Release);

    // Restore saved position if available.
    if let Some((sx, sy)) = load_pos() {
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct NSPoint { x: f64, y: f64 }
        let set_origin: extern "C" fn(*mut c_void, *mut c_void, NSPoint) =
            std::mem::transmute(objc_msgSend as *const ());
        set_origin(panel, sel(b"setFrameOrigin:\0"), NSPoint { x: sx, y: sy });
    }
}

unsafe fn update_text_view() {
    let tv = TEXT_VIEW_PTR.load(Ordering::Acquire);
    if tv.is_null() { return; }

    let text = OVERLAY_TEXT.lock().unwrap().clone();
    let ns_str = nsstring(&text);
    msg1(tv, sel(b"setString:\0"), ns_str);

    // Scroll to bottom
    let len: usize = {
        let storage = msg0(tv, sel(b"textStorage\0"));
        let f: extern "C" fn(*mut c_void, *mut c_void) -> usize = std::mem::transmute(objc_msgSend as *const ());
        f(storage, sel(b"length\0"))
    };
    if len > 0 {
        #[repr(C)]
        struct NSRange { location: usize, length: usize }
        let range = NSRange { location: len, length: 0 };
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRange) = std::mem::transmute(objc_msgSend as *const ());
        f(tv, sel(b"scrollRangeToVisible:\0"), range);
    }
}

unsafe fn show_window() {
    let w = WINDOW_PTR.load(Ordering::Acquire);
    if w.is_null() { return; }
    msg1(w, sel(b"makeKeyAndOrderFront:\0"), std::ptr::null_mut());
}

unsafe fn hide_window() {
    let w = WINDOW_PTR.load(Ordering::Acquire);
    if w.is_null() { return; }

    // Save current position before hiding.
    let frame: NSRect = {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
            std::mem::transmute(objc_msgSend as *const ());
        f(w, sel(b"frame\0"))
    };
    save_pos(frame.x, frame.y);

    msg1(w, sel(b"orderOut:\0"), std::ptr::null_mut());
}

// ── Non-modal prompt input panel ─────────────────────────────────────────────

static PROMPT_WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PROMPT_TV_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PROMPT_CHECKBOX_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static PROMPT_RESULT: Mutex<Option<std::sync::mpsc::Sender<Option<String>>>> = Mutex::new(None);

/// Show a non-modal prompt input panel. Returns user's text or None if cancelled.
/// Does NOT block the main run loop — voice overlay keeps updating.
pub fn show_prompt_panel(title: &str, message: &str, prefill: &str) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel::<Option<String>>();
    *PROMPT_RESULT.lock().unwrap() = Some(tx);

    // Format: clipboard text at top, then "\n---\n\n===" with cursor between --- and ===.
    let formatted = if prefill.is_empty() {
        "---\n\n===".to_string()
    } else {
        format!("{}\n---\n\n===", prefill)
    };
    // Cursor goes on the blank line between "---" and "==="
    let cursor_pos = formatted.rfind("\n\n===").map(|p| p + 1).unwrap_or(formatted.len());

    // Store data for the main-thread callback.
    static PROMPT_DATA: Mutex<Option<(String, String, String, usize)>> = Mutex::new(None);
    *PROMPT_DATA.lock().unwrap() = Some((title.to_string(), message.to_string(), formatted, cursor_pos));

    // Register CLXPromptTextView (Enter=Send, Shift+Enter=newline) and action target.
    static PROMPT_CLASSES_REGISTERED: std::sync::Once = std::sync::Once::new();
    #[allow(unused)]
    static PROMPT_ACTION_REGISTERED: std::sync::Once = std::sync::Once::new();

    unsafe extern "C" fn cancel_action(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
        prompt_cancel();
    }

    unsafe extern "C" fn send_action(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
        prompt_send();
    }

    // keyDown: handler for CLXPromptTextView — Enter=Send, Shift+Enter=newline.
    unsafe extern "C" fn prompt_tv_key_down(
        this: *mut c_void, _cmd: *mut c_void, event: *mut c_void,
    ) {
        let f_u16: extern "C" fn(*mut c_void, *mut c_void) -> u16
            = std::mem::transmute(objc_msgSend as *const ());
        let key_code = f_u16(event, sel(b"keyCode\0"));
        let f_u64: extern "C" fn(*mut c_void, *mut c_void) -> u64
            = std::mem::transmute(objc_msgSend as *const ());
        let flags = f_u64(event, sel(b"modifierFlags\0"));
        let shift = flags & (1 << 17) != 0;

        if key_code == 53 {
            // ESC → Cancel
            prompt_cancel();
        } else if key_code == 36 && !shift {
            // Enter without Shift → Send
            prompt_send();
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

    extern "C" fn create_prompt(_: *mut c_void) {
        unsafe {
            // Register both CLXPromptTextView and CLXPromptAction classes (once).
            PROMPT_CLASSES_REGISTERED.call_once(|| {
                extern "C" {
                    fn objc_allocateClassPair(sup: *mut c_void, name: *const std::ffi::c_char, extra: usize) -> *mut c_void;
                    fn objc_registerClassPair(cls_ptr: *mut c_void);
                    fn class_addMethod(cls_ptr: *mut c_void, sel: *mut c_void, imp: *const c_void, types: *const std::ffi::c_char) -> bool;
                }

                // CLXPromptTextView: NSTextView subclass with Enter=Send.
                let tv_super = cls(b"NSTextView\0");
                let tv_cls = objc_allocateClassPair(tv_super, b"CLXPromptTextView\0".as_ptr() as *const _, 0);
                if !tv_cls.is_null() {
                    class_addMethod(tv_cls, sel(b"keyDown:\0"), prompt_tv_key_down as *const c_void, b"v@:@\0".as_ptr() as *const _);
                    objc_registerClassPair(tv_cls);
                }

                // CLXPromptAction: button action target.
                let obj_super = cls(b"NSObject\0");
                let act_cls = objc_allocateClassPair(obj_super, b"CLXPromptAction\0".as_ptr() as *const _, 0);
                if !act_cls.is_null() {
                    class_addMethod(act_cls, sel(b"cancelPrompt:\0"), cancel_action as *const c_void, b"v@:@\0".as_ptr() as *const _);
                    class_addMethod(act_cls, sel(b"sendPrompt:\0"), send_action as *const c_void, b"v@:@\0".as_ptr() as *const _);
                    objc_registerClassPair(act_cls);
                }
            });
            let (title, message, formatted, cursor_pos) = {
                let guard = PROMPT_DATA.lock().unwrap();
                guard.as_ref().unwrap().clone()
            };

            // Close existing prompt if any.
            let existing = PROMPT_WINDOW_PTR.load(Ordering::Acquire);
            if !existing.is_null() {
                msg1(existing, sel(b"orderOut:\0"), std::ptr::null_mut());
            }

            let screen = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
            let sf: NSRect = {
                let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect = std::mem::transmute(objc_msgSend as *const ());
                f(screen, sel(b"frame\0"))
            };
            let pw = 520.0_f64;
            let ph = 320.0_f64;
            let rect = NSRect { x: (sf.w - pw) / 2.0, y: (sf.h - ph) / 2.0 + 100.0, w: pw, h: ph };

            // NSPanel: titled + closable + non-activating
            // Use NSWindow (not NSPanel) so Cmd+A/C/V/Z work properly.
            // titled(1) + closable(2) + resizable(8)
            let style: u64 = 1 | 2 | 8;
            let panel = msg0(cls(b"NSWindow\0"), sel(b"alloc\0"));
            let panel: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void
                    = std::mem::transmute(objc_msgSend as *const ());
                f(panel, sel(b"initWithContentRect:styleMask:backing:defer:\0"), rect, style, 2, false)
            };

            msg1(panel, sel(b"setTitle:\0"), nsstring(&title));
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

            // Buttons at bottom (40px)
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
                f(send_btn, sel(b"setBezelStyle:\0"), 1); // NSBezelStyleRounded
            }
            msg1(send_btn, sel(b"setKeyEquivalent:\0"), nsstring("")); // Enter handled by text view

            // Create action target for buttons.
            let action_cls = cls(b"CLXPromptAction\0");
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

            // "Clear history" checkbox — checked by default.
            let cb_rect = NSRect { x: 12.0, y: btn_y + 2.0, w: 140.0, h: btn_h };
            let checkbox = msg0(cls(b"NSButton\0"), sel(b"alloc\0"));
            let checkbox: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(checkbox, sel(b"initWithFrame:\0"), cb_rect)
            };
            // Label shows history count. State loaded from persistent pref.
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
            PROMPT_CHECKBOX_PTR.store(checkbox, Ordering::Release);

            // ScrollView + TextView (between info and buttons)
            let tv_rect = NSRect { x: 12.0, y: btn_y + btn_h + 8.0, w: pw - 24.0, h: cv_rect.h - 50.0 - btn_h - 24.0 };
            let scroll = msg0(cls(b"NSScrollView\0"), sel(b"alloc\0"));
            let scroll: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(scroll, sel(b"initWithFrame:\0"), tv_rect)
            };
            set_bool(scroll, sel(b"setHasVerticalScroller:\0"), true);

            // Use CLXPromptTextView if registered, else NSTextView
            let tv_cls = {
                let c = cls(b"CLXPromptTextView\0");
                if c.is_null() { cls(b"NSTextView\0") } else { c }
            };
            let text_view = msg0(tv_cls, sel(b"alloc\0"));
            let inner_rect = NSRect { x: 0.0, y: 0.0, w: tv_rect.w, h: tv_rect.h };
            let text_view: *mut c_void = {
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(text_view, sel(b"initWithFrame:\0"), inner_rect)
            };
            set_bool(text_view, sel(b"setEditable:\0"), true);
            set_bool(text_view, sel(b"setRichText:\0"), false);

            // Set text and cursor
            msg1(text_view, sel(b"setString:\0"), nsstring(&formatted));
            {
                #[repr(C)] #[derive(Clone, Copy)] struct NSRange { location: usize, length: usize }
                let range = NSRange { location: cursor_pos, length: 0 };
                let f: extern "C" fn(*mut c_void, *mut c_void, NSRange) = std::mem::transmute(objc_msgSend as *const ());
                f(text_view, sel(b"setSelectedRange:\0"), range);
                f(text_view, sel(b"scrollRangeToVisible:\0"), range);
            }

            // Font
            let font = {
                let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
                f(cls(b"NSFont\0"), sel(b"systemFontOfSize:\0"), 13.0)
            };
            msg1(text_view, sel(b"setFont:\0"), font);

            msg1(scroll, sel(b"setDocumentView:\0"), text_view);
            msg1(container, sel(b"addSubview:\0"), scroll);
            msg1(panel, sel(b"setContentView:\0"), container);

            // Show and focus
            msg0(panel, sel(b"retain\0"));
            msg1(panel, sel(b"makeKeyAndOrderFront:\0"), std::ptr::null_mut());
            msg1(panel, sel(b"makeFirstResponder:\0"), text_view);

            // Activate app
            let nsapp = msg0(cls(b"NSApplication\0"), sel(b"sharedApplication\0"));
            {
                let f: extern "C" fn(*mut c_void, *mut c_void, i64) -> bool = std::mem::transmute(objc_msgSend as *const ());
                f(nsapp, sel(b"activateIgnoringOtherApps:\0"), 1);
            }

            PROMPT_WINDOW_PTR.store(panel, Ordering::Release);
            PROMPT_TV_PTR.store(text_view, Ordering::Release);
        }
    }

    unsafe { dispatch_async_f(main_queue(), std::ptr::null_mut(), create_prompt); }

    // Block the calling thread (NOT main thread) until user clicks Send or Cancel.
    rx.recv().ok().flatten()
}

/// Called from CLXPromptTextView.keyDown: when Enter is pressed (Send).
pub fn prompt_send() {
    unsafe {
        let tv = PROMPT_TV_PTR.load(Ordering::Acquire);
        if tv.is_null() { return; }

        // Get text from text view.
        let ns_str = msg0(tv, sel(b"string\0"));
        let cstr: *const std::ffi::c_char = {
            let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char = std::mem::transmute(objc_msgSend as *const ());
            f(ns_str, sel(b"UTF8String\0"))
        };

        // Check if "Keep histories" is checked.
        let keep_history = {
            let cb = PROMPT_CHECKBOX_PTR.load(Ordering::Acquire);
            if !cb.is_null() {
                let f: extern "C" fn(*mut c_void, *mut c_void) -> i64 = std::mem::transmute(objc_msgSend as *const ());
                f(cb, sel(b"state\0")) == 1 // NSControlStateValueOn
            } else {
                false // default: don't keep
            }
        };

        let text = if !cstr.is_null() {
            let mut t = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
            if keep_history {
                t = format!("[KEEP]\n{}", t);
            }
            Some(t)
        } else {
            None
        };

        // Hide panel.
        let panel = PROMPT_WINDOW_PTR.load(Ordering::Acquire);
        if !panel.is_null() {
            msg1(panel, sel(b"orderOut:\0"), std::ptr::null_mut());
        }

        // Send result.
        if let Some(tx) = PROMPT_RESULT.lock().unwrap().take() {
            let _ = tx.send(text);
        }
    }
}

/// Called when Cancel is pressed or window closed.
pub fn prompt_cancel() {
    unsafe {
        let panel = PROMPT_WINDOW_PTR.load(Ordering::Acquire);
        if !panel.is_null() {
            msg1(panel, sel(b"orderOut:\0"), std::ptr::null_mut());
        }
        if let Some(tx) = PROMPT_RESULT.lock().unwrap().take() {
            let _ = tx.send(None);
        }
    }
}
