//! Floating HUD overlay showing CapsLockX keyboard layout.
//!
//! Displays a visual keyboard map when CLX+/ is pressed. Toggle behavior:
//! press once to show, press again (or ESC) to hide.
//! Uses NSPanel + NSTextView with monospace font, same pattern as brainstorm_overlay.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

static WINDOW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static TEXT_VIEW_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static VISIBLE: AtomicBool = AtomicBool::new(false);

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

fn layout_text() -> &'static str {
    concat!(
        "  CapsLockX 2.0 — Keyboard Layout HUD\n",
        "  Hold CapsLock or Space + press key\n",
        "  ─────────────────────────────────────────────\n",
        "\n",
        "  ┌───────── DESKTOP ──────────┐\n",
        "  │ 1   2   3   4   5   6   7   8   9   0  │\n",
        "  │Dsk1 Dsk2 Dsk3 Dsk4 Dsk5 Dsk6 Dsk7 Dsk8 Dsk9 Dsk0│\n",
        "  └──────────────────────────────┘\n",
        "\n",
        "  ┌── MOUSE ──┐ ┌ACT┐ ┌─ PAGE NAV ─┐ ┌ACT┐ ┌─MEDIA─┐\n",
        "  │ Q    W    E    R  │  T  │ Y    U    I    O  │  P  │ [  ]  \\ │\n",
        "  │RClk M↑  LClk Scr↑│ Del │Home PgDn PgUp End │S+Tab│Prv Nxt Play│\n",
        "  └───────────┘ └───┘ └────────────┘ └───┘ └──────┘\n",
        "\n",
        "  TRIGGER  ┌── MOUSE ──┐ ┌ACT┐ ┌── CURSOR ──┐\n",
        "  CapsLk   │ A    S    D    F  │  G  │ H    J    K    L  │\n",
        "           │ M←   M↓   M→  Scr↓│Enter│  ←   ↓    ↑   →  │\n",
        "           └───────────┘ └───┘ └────────────┘\n",
        "\n",
        "  Shift ┌─ WINDOW ─┐ ┌──── AI ────┐ ┌ACT┐ ┌AI ┐ ┌MISC─────┐\n",
        "        │ Z    X    C  │ V    B   │  N  │  M  │ ,    .    / │\n",
        "        │Cycle Cls Tile│Voice Brain│ Tab │Agent│Prefs Rstr HUD│\n",
        "        └──────────┘ └─────────┘ └───┘ └───┘ └─────────┘\n",
        "\n",
        "  Ctrl  Alt  Cmd  ┌────── TRIGGER (hold) ──────┐ Cmd  Alt  Ctrl\n",
        "                  │         Space               │\n",
        "                  └─────────────────────────────┘\n",
        "\n",
        "  Legend: CURSOR=blue  MOUSE=green  PAGE=cyan  ACTION=purple\n",
        "          WINDOW=orange  DESKTOP=yellow  AI=red  MEDIA=teal\n",
        "  ─────────────────────────────────────────────\n",
        "  Press CLX+/ again to dismiss\n",
    )
}

extern "C" fn do_toggle(_: *mut c_void) {
    unsafe {
        if VISIBLE.load(Ordering::Relaxed) {
            hide_window();
            VISIBLE.store(false, Ordering::Relaxed);
        } else {
            ensure_window();
            update_text_view();
            show_window();
            VISIBLE.store(true, Ordering::Relaxed);
        }
    }
}

extern "C" fn do_hide(_: *mut c_void) {
    unsafe {
        hide_window();
        VISIBLE.store(false, Ordering::Relaxed);
    }
}

pub fn toggle_overlay() {
    unsafe { dispatch_async_f(main_queue(), std::ptr::null_mut(), do_toggle); }
}

pub fn hide_overlay() {
    if VISIBLE.load(Ordering::Relaxed) {
        unsafe { dispatch_async_f(main_queue(), std::ptr::null_mut(), do_hide); }
    }
}

pub fn is_visible() -> bool {
    VISIBLE.load(Ordering::Relaxed)
}

unsafe fn ensure_window() {
    if !WINDOW_PTR.load(Ordering::Relaxed).is_null() { return; }

    // Get screen size for centering
    let screen = msg0(cls(b"NSScreen\0"), sel(b"mainScreen\0"));
    let frame: NSRect = {
        let f: extern "C" fn(*mut c_void, *mut c_void) -> NSRect = std::mem::transmute(objc_msgSend as *const ());
        f(screen, sel(b"frame\0"))
    };

    // Panel: 620px wide, 460px tall, centered on screen
    let panel_w = 620.0;
    let panel_h = 460.0;
    let panel_x = (frame.w - panel_w) / 2.0;
    let panel_y = (frame.h - panel_h) / 2.0;
    let rect = NSRect { x: panel_x, y: panel_y, w: panel_w, h: panel_h };

    // NSPanel (non-activating, floating)
    // styleMask: titled(1) + nonActivatingPanel(1<<7) + utilityWindow(1<<4)
    let style: u64 = 1 | (1 << 4) | (1 << 7);
    let panel_cls = cls(b"NSPanel\0");
    let panel = msg0(panel_cls, sel(b"alloc\0"));
    let init_sel = sel(b"initWithContentRect:styleMask:backing:defer:\0");
    let panel: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect, u64, u64, bool) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(panel, init_sel, rect, style, 2/*buffered*/, false)
    };

    msg1(panel, sel(b"setTitle:\0"), nsstring("CapsLockX Layout"));
    set_f64(panel, sel(b"setAlphaValue:\0"), 0.95);
    set_bool(panel, sel(b"setOpaque:\0"), false);
    set_bool(panel, sel(b"setIgnoresMouseEvents:\0"), true);
    set_bool(panel, sel(b"setHidesOnDeactivate:\0"), false);

    // Dark background
    let bg_color = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.10, 0.10, 0.15, 0.96)
    };
    msg1(panel, sel(b"setBackgroundColor:\0"), bg_color);

    // Floating above all windows (NSStatusWindowLevel = 25 for HUD feel)
    {
        let f: extern "C" fn(*mut c_void, *mut c_void, i64) = std::mem::transmute(objc_msgSend as *const ());
        f(panel, sel(b"setLevel:\0"), 25);
    }
    set_bool(panel, sel(b"setReleasedWhenClosed:\0"), false);
    // Hide from screen sharing
    {
        let f: extern "C" fn(*mut c_void, *mut c_void, u64) = std::mem::transmute(objc_msgSend as *const ());
        f(panel, sel(b"setSharingType:\0"), 0);
    }

    // TextView (no scrollview needed, content is static)
    let content_rect = NSRect { x: 0.0, y: 0.0, w: panel_w, h: panel_h - 28.0 };
    let text_view = msg0(cls(b"NSTextView\0"), sel(b"alloc\0"));
    let text_view: *mut c_void = {
        let f: extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(text_view, sel(b"initWithFrame:\0"), content_rect)
    };
    set_bool(text_view, sel(b"setEditable:\0"), false);
    set_bool(text_view, sel(b"setSelectable:\0"), false);
    set_bool(text_view, sel(b"setRichText:\0"), false);

    // Monospace font, 12pt
    let font = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSFont\0"), sel(b"monospacedSystemFontOfSize:weight:\0"), 12.0)
    };
    msg1(text_view, sel(b"setFont:\0"), font);

    // Light green-ish text on dark background
    let text_color = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.75, 0.90, 0.75, 1.0)
    };
    msg1(text_view, sel(b"setTextColor:\0"), text_color);

    let tv_bg = {
        let f: extern "C" fn(*mut c_void, *mut c_void, f64, f64, f64, f64) -> *mut c_void
            = std::mem::transmute(objc_msgSend as *const ());
        f(cls(b"NSColor\0"), sel(b"colorWithRed:green:blue:alpha:\0"), 0.08, 0.08, 0.12, 1.0)
    };
    msg1(text_view, sel(b"setBackgroundColor:\0"), tv_bg);
    set_bool(text_view, sel(b"setDrawsBackground:\0"), true);

    msg1(panel, sel(b"setContentView:\0"), text_view);

    msg0(panel, sel(b"retain\0"));
    WINDOW_PTR.store(panel, Ordering::Release);
    TEXT_VIEW_PTR.store(text_view, Ordering::Release);
}

unsafe fn update_text_view() {
    let tv = TEXT_VIEW_PTR.load(Ordering::Acquire);
    if tv.is_null() { return; }
    let ns_str = nsstring(layout_text());
    msg1(tv, sel(b"setString:\0"), ns_str);
}

unsafe fn show_window() {
    let w = WINDOW_PTR.load(Ordering::Acquire);
    if w.is_null() { return; }
    msg1(w, sel(b"makeKeyAndOrderFront:\0"), std::ptr::null_mut());
}

unsafe fn hide_window() {
    let w = WINDOW_PTR.load(Ordering::Acquire);
    if w.is_null() { return; }
    msg1(w, sel(b"orderOut:\0"), std::ptr::null_mut());
}
