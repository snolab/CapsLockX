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

// ── Daemon-based prompt input (zero cold-start) ──────────────────────────────

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::OnceLock;

struct PromptDaemon {
    _child: Child,
    stdin:  ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Drop for PromptDaemon {
    fn drop(&mut self) {
        // Kill the child explicitly so it doesn't outlive us as an orphan.
        // Without this, repeated clx restarts (watchdog) leak clx-prompt
        // processes (each ~74 MB) until the user reboots.
        let _ = self._child.kill();
        let _ = self._child.wait();
    }
}

static DAEMON: OnceLock<Mutex<PromptDaemon>> = OnceLock::new();

fn prompt_bin_path() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = exe.parent().unwrap_or(std::path::Path::new("."));
    // Check same dir as binary, then bin/ subdir (where build.sh deploys it).
    for candidate in [dir.join("clx-prompt"), dir.join("bin").join("clx-prompt")] {
        if candidate.exists() {
            // Verify it's the Tauri build, not an old AppKit binary.
            if let Ok(bytes) = std::fs::read(&candidate) {
                if bytes.windows(5).any(|w| w == b"tauri") {
                    return candidate;
                }
                eprintln!("[CLX] skipping non-Tauri clx-prompt at {:?}", candidate);
            }
        }
    }
    std::path::PathBuf::from("clx-prompt")
}

/// Pre-spawn the clx-prompt daemon so the window is warm before first use.
/// Call this at CLX startup. Safe to call multiple times.
pub fn spawn_prompt_daemon() {
    DAEMON.get_or_init(|| {
        let bin = prompt_bin_path();
        eprintln!("[CLX] spawning clx-prompt daemon: {:?}", bin);
        let mut child = std::process::Command::new(&bin)
            .arg("--daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn clx-prompt --daemon");

        let stdin  = child.stdin.take().expect("no stdin");
        let stdout = BufReader::new(child.stdout.take().expect("no stdout"));

        let mut daemon = PromptDaemon { _child: child, stdin, stdout };

        // Wait for {"type":"ready"} line before returning.
        let mut line = String::new();
        let _ = daemon.stdout.read_line(&mut line);
        eprintln!("[CLX] clx-prompt daemon ready: {}", line.trim());

        Mutex::new(daemon)
    });
}

/// Show the prompt dialog. Blocks until user submits or cancels.
/// Returns the prompt text (with optional "[KEEP]\n" prefix), or None if cancelled.
pub fn show_prompt_panel(title: &str, context: &str, last_prompt: &str) -> Option<String> {
    // Ensure daemon is running (lazy fallback if spawn_prompt_daemon wasn't called at startup).
    spawn_prompt_daemon();

    let daemon_lock = DAEMON.get()?;
    let mut d = daemon_lock.lock().ok()?;

    // Send show command.
    let cmd = serde_json::json!({
        "cmd": "show",
        "title": title,
        "context": context,
        "last_prompt": last_prompt,
    });
    writeln!(d.stdin, "{cmd}").ok()?;
    d.stdin.flush().ok()?;

    // Block until we get a result line.
    let mut line = String::new();
    d.stdout.read_line(&mut line).ok()?;
    let msg: serde_json::Value = serde_json::from_str(line.trim()).ok()?;

    match msg["type"].as_str()? {
        "submit" => {
            let text = msg["text"].as_str()?.to_string();
            let keep = msg["keep"].as_bool().unwrap_or(false);
            Some(if keep { format!("[KEEP]\n{text}") } else { text })
        }
        _ => {
            eprintln!("[CLX] prompt cancelled");
            None
        }
    }
}

