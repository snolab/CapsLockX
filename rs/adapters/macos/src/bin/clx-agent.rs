// clx-agent — standalone LLM-driven computer control agent.
//
// Reads the accessibility tree + voice input, sends to LLM, streams
// CLX commands, parses and executes them, echoes results back.
//
// Usage: clx-agent [--prompt "do something"] [--model gemini-2.0-flash]
//        clx agent [--prompt "do something"]  (as clx subcommand)
//
// Or launched by CapsLockX main process via CLX+M hotkey.

use std::ffi::c_void;
use std::io::{self, Write, BufRead};

// ── Timestamp + File Logging ─────────────────────────────────────────────────

static mut SESSION_START: Option<std::time::Instant> = None;
static mut LOG_FILE: Option<std::sync::Mutex<std::fs::File>> = None;

fn init_logging() {
    unsafe {
        SESSION_START = Some(std::time::Instant::now());
        let log_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("CapsLockX")
            .join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("agent.log");
        if let Ok(f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            LOG_FILE = Some(std::sync::Mutex::new(f));
        }
    }
}

fn elapsed_ms() -> u64 {
    unsafe {
        SESSION_START.as_ref()
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Log with timestamp to both stderr and log file.
fn tlog(msg: &str) {
    let ts = elapsed_ms();
    let line = format!("[{:>8}ms] {}", ts, msg);
    eprintln!("{}", line);
    unsafe {
        if let Some(ref f) = LOG_FILE {
            if let Ok(mut f) = f.lock() {
                let _ = writeln!(f, "{}", line);
            }
        }
    }
}

// ── macOS FFI ────────────────────────────────────────────────────────────────

type AXUIElementRef = *mut c_void;
type CFStringRef = *mut c_void;
type CFArrayRef = *mut c_void;
type CFTypeRef = *mut c_void;

// ApplicationServices linked lazily — only used when AX permission is confirmed.
// The #[link] attribute causes dyld to resolve symbols at load time,
// which can trigger AX subsystem init and hang without permission.
// So we use dlsym for lazy loading instead.
extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
}

type AXUIElementCreateApplicationFn = unsafe extern "C" fn(pid: i32) -> AXUIElementRef;
type AXUIElementCopyAttributeValueFn = unsafe extern "C" fn(
    element: AXUIElementRef, attribute: CFStringRef, value: *mut CFTypeRef,
) -> i32;

// RTLD_DEFAULT
const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

unsafe fn ax_create_app(pid: i32) -> AXUIElementRef {
    let sym = dlsym(RTLD_DEFAULT, b"AXUIElementCreateApplication\0".as_ptr() as *const _);
    if sym.is_null() { return std::ptr::null_mut(); }
    let f: AXUIElementCreateApplicationFn = std::mem::transmute(sym);
    f(pid)
}

unsafe fn ax_copy_attr(element: AXUIElementRef, attribute: CFStringRef, value: *mut CFTypeRef) -> i32 {
    let sym = dlsym(RTLD_DEFAULT, b"AXUIElementCopyAttributeValue\0".as_ptr() as *const _);
    if sym.is_null() { return -1; }
    let f: AXUIElementCopyAttributeValueFn = std::mem::transmute(sym);
    f(element, attribute, value)
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(arr: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> CFTypeRef;
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFBooleanGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFStringGetCString(s: CFStringRef, buf: *mut u8, buf_size: isize, encoding: u32) -> bool;
    fn CFStringGetLength(s: CFStringRef) -> isize;
    fn CFStringCreateWithCString(alloc: *mut c_void, s: *const u8, encoding: u32) -> CFStringRef;
    fn CFNumberGetValue(number: CFTypeRef, the_type: i32, value_ptr: *mut c_void) -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateMouseEvent(
        source: *mut c_void, mouse_type: u32, point: CGPoint, button: u32,
    ) -> *mut c_void;
    fn CGEventPost(tap: u32, event: *mut c_void);
    fn CGEventCreateKeyboardEvent(
        source: *mut c_void, keycode: u16, key_down: bool,
    ) -> *mut c_void;
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventSetIntegerValueField(event: *mut c_void, field: u32, value: i64);
    fn CGEventKeyboardSetUnicodeString(event: *mut c_void, len: u32, chars: *const u16);
}

// ── Target process ───────────────────────────────────────────────────────────

static mut TARGET_PID: i32 = 0; // 0 = global (all apps), >0 = specific process

/// Post a CGEvent, targeting a specific process if --target was set.
unsafe fn post_event(event: *mut c_void) {
    if event.is_null() { return; }
    let pid = TARGET_PID;
    if pid > 0 {
        // Field 40 = kCGEventTargetUnixProcessID — routes event to specific app.
        // Zero overhead vs CGEventPost, no focus switch needed.
        CGEventSetIntegerValueField(event, 40, pid as i64);
    }
    CGEventPost(0, event);
}

/// Find PID of a running app by name (case-insensitive substring match).
fn find_pid_by_name(name: &str) -> Option<i32> {
    unsafe {
        extern "C" {
            fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
            fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
            fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
        }

        let ws_cls = objc_getClass(b"NSWorkspace\0".as_ptr() as *const _);
        let f0: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let ws = f0(ws_cls, sel_registerName(b"sharedWorkspace\0".as_ptr() as *const _));
        let apps = f0(ws, sel_registerName(b"runningApplications\0".as_ptr() as *const _));

        let count = CFArrayGetCount(apps);
        let name_lower = name.to_lowercase();

        for i in 0..count {
            let app = CFArrayGetValueAtIndex(apps, i);
            if app.is_null() { continue; }

            let ns_name = f0(app, sel_registerName(b"localizedName\0".as_ptr() as *const _));
            if ns_name.is_null() { continue; }

            if let Some(app_name) = cfstring_to_string(ns_name) {
                if app_name.to_lowercase().contains(&name_lower) {
                    let fi: extern "C" fn(*mut c_void, *mut c_void) -> i32 =
                        std::mem::transmute(objc_msgSend as *const ());
                    let pid = fi(app, sel_registerName(b"processIdentifier\0".as_ptr() as *const _));
                    return Some(pid);
                }
            }
        }
        None
    }
}

/// Activate (bring to front) the target app by PID.
fn activate_pid(pid: i32) {
    unsafe {
        extern "C" {
            fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
            fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
            fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
        }
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr() as *const _);
        let fi: extern "C" fn(*mut c_void, *mut c_void, i32) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let app = fi(cls, sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr() as *const _), pid);
        if !app.is_null() {
            let fa: extern "C" fn(*mut c_void, *mut c_void, u64) -> bool =
                std::mem::transmute(objc_msgSend as *const ());
            fa(app, sel_registerName(b"activateWithOptions:\0".as_ptr() as *const _), 3); // NSApplicationActivateIgnoringOtherApps
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGPoint { x: f64, y: f64 }

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGSize { w: f64, h: f64 }

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

// ── CF helpers ───────────────────────────────────────────────────────────────

unsafe fn cfstr(s: &str) -> CFStringRef {
    let cstr = std::ffi::CString::new(s).unwrap();
    CFStringCreateWithCString(std::ptr::null_mut(), cstr.as_ptr() as *const u8, K_CF_STRING_ENCODING_UTF8)
}

unsafe fn cfstring_to_string(cf: CFStringRef) -> Option<String> {
    if cf.is_null() { return None; }
    let len = CFStringGetLength(cf);
    if len <= 0 { return Some(String::new()); }
    let buf_size = len * 4 + 1;
    let mut buf = vec![0u8; buf_size as usize];
    if CFStringGetCString(cf, buf.as_mut_ptr(), buf_size, K_CF_STRING_ENCODING_UTF8) {
        let s = std::ffi::CStr::from_ptr(buf.as_ptr() as *const _);
        Some(s.to_string_lossy().into_owned())
    } else {
        None
    }
}

unsafe fn ax_attr_string(elem: AXUIElementRef, attr: &str) -> Option<String> {
    let attr_cf = cfstr(attr);
    let mut val: CFTypeRef = std::ptr::null_mut();
    let err = ax_copy_attr(elem, attr_cf, &mut val);
    CFRelease(attr_cf);
    if err != 0 || val.is_null() { return None; }
    let type_id = CFGetTypeID(val);
    let result = if type_id == CFStringGetTypeID() {
        cfstring_to_string(val)
    } else {
        None
    };
    CFRelease(val);
    result
}

unsafe fn ax_attr_ref(elem: AXUIElementRef, attr: &str) -> Option<CFTypeRef> {
    let attr_cf = cfstr(attr);
    let mut val: CFTypeRef = std::ptr::null_mut();
    let err = ax_copy_attr(elem, attr_cf, &mut val);
    CFRelease(attr_cf);
    if err != 0 || val.is_null() { None } else { Some(val) }
}

unsafe fn ax_attr_array(elem: AXUIElementRef, attr: &str) -> Option<(CFArrayRef, isize)> {
    let val = ax_attr_ref(elem, attr)?;
    if CFGetTypeID(val) != CFArrayGetTypeID() {
        CFRelease(val);
        return None;
    }
    let count = CFArrayGetCount(val);
    Some((val, count))
}

unsafe fn ax_value_get(value: CFTypeRef, value_type: i32, value_ptr: *mut c_void) -> bool {
    let sym = dlsym(RTLD_DEFAULT, b"AXValueGetValue\0".as_ptr() as *const _);
    if sym.is_null() { return false; }
    let f: unsafe extern "C" fn(CFTypeRef, i32, *mut c_void) -> bool = std::mem::transmute(sym);
    f(value, value_type, value_ptr)
}

unsafe fn ax_attr_point(elem: AXUIElementRef, attr: &str) -> Option<CGPoint> {
    let val = ax_attr_ref(elem, attr)?;
    let mut point = CGPoint { x: 0.0, y: 0.0 };
    let ok = ax_value_get(val, 1, &mut point as *mut _ as *mut c_void);
    CFRelease(val);
    if ok { Some(point) } else { None }
}

unsafe fn ax_attr_size(elem: AXUIElementRef, attr: &str) -> Option<CGSize> {
    let val = ax_attr_ref(elem, attr)?;
    let mut size = CGSize { w: 0.0, h: 0.0 };
    let ok = ax_value_get(val, 2, &mut size as *mut _ as *mut c_void);
    CFRelease(val);
    if ok { Some(size) } else { None }
}

// ── Accessibility Tree ───────────────────────────────────────────────────────

unsafe fn read_ax_tree(elem: AXUIElementRef, depth: usize, out: &mut String, max_depth: usize) {
    if depth > max_depth { return; }

    let indent = "  ".repeat(depth);
    let role = ax_attr_string(elem, "AXRole").unwrap_or_default();
    let title = ax_attr_string(elem, "AXTitle").unwrap_or_default();
    let value = ax_attr_string(elem, "AXValue").unwrap_or_default();
    let desc = ax_attr_string(elem, "AXDescription").unwrap_or_default();
    let role_desc = ax_attr_string(elem, "AXRoleDescription").unwrap_or_default();

    // Get position and size for clickable elements
    let pos = ax_attr_point(elem, "AXPosition");
    let size = ax_attr_size(elem, "AXSize");

    // Compute center position
    let center = match (pos, size) {
        (Some(p), Some(s)) => Some(CGPoint {
            x: p.x + s.w / 2.0,
            y: p.y + s.h / 2.0,
        }),
        _ => None,
    };

    // Build label: prefer title, then description, then value (truncated)
    let label = if !title.is_empty() {
        title.clone()
    } else if !desc.is_empty() {
        desc.clone()
    } else if !value.is_empty() && value.len() < 80 {
        format!("\"{}\"", value.chars().take(60).collect::<String>())
    } else {
        String::new()
    };

    // Skip empty containers to keep tree compact
    let short_role = match role.as_str() {
        "AXWindow" => "window",
        "AXButton" => "btn",
        "AXStaticText" => "txt",
        "AXTextField" => "field",
        "AXTextArea" => "textarea",
        "AXLink" => "link",
        "AXImage" => "img",
        "AXGroup" => "group",
        "AXToolbar" => "toolbar",
        "AXMenuBar" => "menubar",
        "AXMenuItem" => "menuitem",
        "AXMenu" => "menu",
        "AXScrollArea" => "scroll",
        "AXWebArea" => "web",
        "AXList" => "list",
        "AXRow" => "row",
        "AXCell" => "cell",
        "AXTable" => "table",
        "AXTabGroup" => "tabs",
        "AXTab" => "tab",
        "AXCheckBox" => "checkbox",
        "AXRadioButton" => "radio",
        "AXPopUpButton" => "popup",
        "AXComboBox" => "combo",
        "AXSlider" => "slider",
        "AXHeading" => "heading",
        "AXApplication" => "app",
        _ => {
            // Skip unknown roles with no label
            if label.is_empty() && role != "AXGroup" {
                // Still recurse into children
                if let Some((arr, count)) = ax_attr_array(elem, "AXChildren") {
                    for i in 0..count.min(50) {
                        let child = CFArrayGetValueAtIndex(arr, i);
                        if !child.is_null() {
                            read_ax_tree(child, depth, out, max_depth);
                        }
                    }
                    CFRelease(arr);
                }
                return;
            }
            &role_desc
        }
    };

    // Format line
    if !label.is_empty() || matches!(short_role, "window" | "toolbar" | "web" | "menubar" | "tabs") {
        out.push_str(&indent);
        out.push_str(short_role);
        if !label.is_empty() {
            out.push(' ');
            out.push('"');
            out.push_str(&label);
            out.push('"');
        }
        if let Some(c) = center {
            out.push_str(&format!(" @{},{}", c.x as i32, c.y as i32));
        }
        out.push('\n');
    }

    // Recurse into children
    if let Some((arr, count)) = ax_attr_array(elem, "AXChildren") {
        for i in 0..count.min(100) {
            let child = CFArrayGetValueAtIndex(arr, i);
            if !child.is_null() {
                read_ax_tree(child, depth + 1, out, max_depth);
            }
        }
        CFRelease(arr);
    }
}

fn get_frontmost_ax_tree() -> String {
    // Try native AX API first (deeper tree, more detail).
    // Falls back to osascript if native fails (e.g. permission issue).
    let native = get_frontmost_ax_tree_inner();
    if !native.is_empty() && !native.starts_with("[AX] ERROR") {
        return native;
    }
    eprintln!("[clx-agent] native AX failed, falling back to osascript");
    get_frontmost_ax_tree_osascript()
}

fn get_frontmost_ax_tree_osascript() -> String {
    // Use osascript (System Events) to read UI — never calls AX APIs directly.
    // Direct AX calls hang in kernel (UE) when Accessibility permission is
    // missing or revoked after binary rebuild. osascript is safe because
    // System Events has its own permission.
    let script = r#"
tell application "System Events"
    set fp to first application process whose frontmost is true
    set appName to name of fp
    set res to "[AX] app=" & quoted form of appName & linefeed
    try
        repeat with w in (windows of fp)
            set wName to name of w
            set wPos to position of w
            set wSz to size of w
            set cx to (item 1 of wPos) + ((item 1 of wSz) / 2) as integer
            set cy to (item 2 of wPos) + ((item 2 of wSz) / 2) as integer
            set res to res & "  window \"" & wName & "\" @" & cx & "," & cy & linefeed
            try
                repeat with e in (UI elements of w)
                    try
                        set eRole to role of e
                        set eTitle to ""
                        try
                            set eTitle to title of e
                        end try
                        if eTitle is "" then try
                            set eTitle to description of e
                        end try
                        if eTitle is "" then try
                            set v to value of e
                            if (length of v) < 80 then set eTitle to v
                        end try
                        set ePos to position of e
                        set eSz to size of e
                        set ecx to (item 1 of ePos) + ((item 1 of eSz) / 2) as integer
                        set ecy to (item 2 of ePos) + ((item 2 of eSz) / 2) as integer
                        set sr to eRole
                        if eRole is "AXButton" then set sr to "btn"
                        if eRole is "AXStaticText" then set sr to "txt"
                        if eRole is "AXTextField" then set sr to "field"
                        if eRole is "AXLink" then set sr to "link"
                        if eRole is "AXImage" then set sr to "img"
                        if eRole is "AXGroup" then set sr to "group"
                        if eRole is "AXToolbar" then set sr to "toolbar"
                        if eRole is "AXScrollArea" then set sr to "scroll"
                        if eRole is "AXWebArea" then set sr to "web"
                        if eRole is "AXTabGroup" then set sr to "tabs"
                        if eRole is "AXCheckBox" then set sr to "checkbox"
                        if eRole is "AXPopUpButton" then set sr to "popup"
                        if eRole is "AXTextArea" then set sr to "textarea"
                        if eTitle is not "" then
                            if (length of eTitle) > 60 then set eTitle to text 1 thru 60 of eTitle
                            set res to res & "    " & sr & " \"" & eTitle & "\" @" & ecx & "," & ecy & linefeed
                        end if
                    end try
                end repeat
            end try
        end repeat
    end try
    return res
end tell
"#;

    match std::process::Command::new("osascript").arg("-e").arg(script).output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            format!("[AX] WARN: osascript: {}\n[AX] app=Unknown\n", err.trim())
        }
        Err(e) => format!("[AX] ERROR: {}\n", e),
    }
}

fn get_frontmost_ax_tree_inner() -> String {
    unsafe {
        // Get frontmost app PID via NSWorkspace
        extern "C" {
            fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
            fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
            fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
        }

        let ws_cls = objc_getClass(b"NSWorkspace\0".as_ptr() as *const _);
        let f0: extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let ws = f0(ws_cls, sel_registerName(b"sharedWorkspace\0".as_ptr() as *const _));
        let front_app = f0(ws, sel_registerName(b"frontmostApplication\0".as_ptr() as *const _));

        let fi: extern "C" fn(*mut c_void, *mut c_void) -> i32 =
            std::mem::transmute(objc_msgSend as *const ());
        let pid = fi(front_app, sel_registerName(b"processIdentifier\0".as_ptr() as *const _));

        let app_name = {
            let ns = f0(front_app, sel_registerName(b"localizedName\0".as_ptr() as *const _));
            if ns.is_null() { "Unknown".to_string() } else {
                let f: extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char =
                    std::mem::transmute(objc_msgSend as *const ());
                let cstr = f(ns, sel_registerName(b"UTF8String\0".as_ptr() as *const _));
                if cstr.is_null() { "Unknown".to_string() }
                else { std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned() }
            }
        };

        let app_elem = ax_create_app(pid);
        let mut tree = format!("[AX] app=\"{}\" pid={}\n", app_name, pid);
        read_ax_tree(app_elem, 0, &mut tree, 6);
        CFRelease(app_elem);
        tree
    }
}

// ── Input Injection ──────────────────────────────────────────────────────────

fn mouse_move_abs(x: f64, y: f64) {
    unsafe {
        let point = CGPoint { x, y };
        // kCGEventMouseMoved = 5, kCGMouseButtonLeft = 0
        let event = CGEventCreateMouseEvent(std::ptr::null_mut(), 5, point, 0);
        if !event.is_null() {
            post_event(event);
            CFRelease(event);
        }
    }
}

fn mouse_click(x: f64, y: f64) {
    unsafe {
        let point = CGPoint { x, y };
        // kCGEventLeftMouseDown = 1, kCGEventLeftMouseUp = 2
        let down = CGEventCreateMouseEvent(std::ptr::null_mut(), 1, point, 0);
        let up = CGEventCreateMouseEvent(std::ptr::null_mut(), 2, point, 0);
        if !down.is_null() { post_event(down); CFRelease(down); }
        std::thread::sleep(std::time::Duration::from_millis(30));
        if !up.is_null() { post_event(up); CFRelease(up); }
    }
}

fn key_tap(keycode: u16) {
    unsafe {
        let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, false);
        if !down.is_null() { post_event(down); CFRelease(down); }
        if !up.is_null() { post_event(up); CFRelease(up); }
    }
}

fn key_tap_with_flags(keycode: u16, flags: u64) {
    unsafe {
        let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, false);
        if !down.is_null() {
            CGEventSetFlags(down, flags);
            post_event(down);
            CFRelease(down);
        }
        if !up.is_null() {
            CGEventSetFlags(up, flags);
            post_event(up);
            CFRelease(up);
        }
    }
}

fn type_text(text: &str) {
    unsafe {
        for ch in text.chars() {
            let mut utf16_buf = [0u16; 2];
            let utf16 = ch.encode_utf16(&mut utf16_buf);
            let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0, true);
            let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0, false);
            if !down.is_null() {
                CGEventKeyboardSetUnicodeString(down, utf16.len() as u32, utf16.as_ptr());
                post_event(down);
                CFRelease(down);
            }
            if !up.is_null() {
                CGEventKeyboardSetUnicodeString(up, utf16.len() as u32, utf16.as_ptr());
                post_event(up);
                CFRelease(up);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}

// ── Key name → macOS keycode mapping ─────────────────────────────────────────

fn keyname_to_code(name: &str) -> Option<u16> {
    Some(match name.to_lowercase().as_str() {
        "a" => 0x00, "s" => 0x01, "d" => 0x02, "f" => 0x03,
        "h" => 0x04, "g" => 0x05, "z" => 0x06, "x" => 0x07,
        "c" => 0x08, "v" => 0x09, "b" => 0x0B, "q" => 0x0C,
        "w" => 0x0D, "e" => 0x0E, "r" => 0x0F, "y" => 0x10,
        "t" => 0x11, "1" => 0x12, "2" => 0x13, "3" => 0x14,
        "4" => 0x15, "6" => 0x16, "5" => 0x17, "9" => 0x19,
        "7" => 0x1A, "8" => 0x1C, "0" => 0x1D, "o" => 0x1F,
        "u" => 0x20, "i" => 0x22, "p" => 0x23, "l" => 0x25,
        "j" => 0x26, "k" => 0x28, "n" => 0x2D, "m" => 0x2E,
        "ret" | "enter" | "return" => 0x24,
        "tab" => 0x30,
        "space" => 0x31,
        "bksp" | "backspace" | "delete" => 0x33,
        "esc" | "escape" => 0x35,
        "del" => 0x75,
        "up" => 0x7E, "down" => 0x7D, "left" => 0x7B, "right" => 0x7C,
        "home" => 0x73, "end" => 0x77,
        "pgup" | "pageup" => 0x74, "pgdn" | "pagedown" => 0x79,
        "f1" => 0x7A, "f2" => 0x78, "f3" => 0x63, "f4" => 0x76,
        "f5" => 0x60, "f6" => 0x61, "f7" => 0x62, "f8" => 0x64,
        "f9" => 0x65, "f10" => 0x6D, "f11" => 0x67, "f12" => 0x6F,
        "-" | "minus" => 0x1B, "=" | "equal" => 0x18,
        "[" => 0x21, "]" => 0x1E, "\\" => 0x2A,
        ";" | "semicolon" => 0x29, "'" | "quote" => 0x27,
        "," | "comma" => 0x2B, "." | "period" => 0x2F,
        "/" | "slash" => 0x2C, "`" | "grave" => 0x32,
        _ => return None,
    })
}

// ── Simple Command Parser ────────────────────────────────────────────────────

#[derive(Debug)]
enum Cmd {
    KeyTap { keycode: u16, flags: u64 },
    TypeString(String),
    MouseMove { x: f64, y: f64 },
    MouseClick { x: f64, y: f64 },
    Wait(std::time::Duration),
    WaitFor { query: String, negate: bool, timeout_ms: u64 },
    Comment(String),
    Unknown(String),
}

fn parse_mods_and_key(s: &str) -> Option<(u16, u64)> {
    let mut flags: u64 = 0;
    let parts: Vec<&str> = s.split('-').collect();
    for (i, part) in parts.iter().enumerate() {
        if i < parts.len() - 1 {
            match *part {
                "c" => flags |= 1 << 18, // kCGEventFlagMaskControl
                "s" => flags |= 1 << 17, // kCGEventFlagMaskShift
                "a" => flags |= 1 << 19, // kCGEventFlagMaskAlternate
                "w" => flags |= 1 << 20, // kCGEventFlagMaskCommand
                _ => {}
            }
        } else {
            let keycode = keyname_to_code(part)?;
            return Some((keycode, flags));
        }
    }
    None
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => { out.push('\\'); out.push(other); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_line(line: &str) -> Cmd {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Cmd::Comment(line.to_string());
    }

    let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
    let rest = rest.trim();

    match verb {
        "k" => {
            if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                return Cmd::TypeString(unescape_string(&rest[1..rest.len()-1]));
            }
            if let Some((keycode, flags)) = parse_mods_and_key(rest) {
                Cmd::KeyTap { keycode, flags }
            } else {
                Cmd::Unknown(line.to_string())
            }
        }
        "m" => {
            let args: Vec<&str> = rest.split_whitespace().collect();
            if args.len() == 1 && args[0] == "c" {
                // Click at current position — need to get current pos
                // For now, just click at 0,0 — will fix with proper tracking
                Cmd::Unknown("m c (no position)".to_string())
            } else if args.len() >= 2 {
                let x: f64 = args[0].parse().unwrap_or(0.0);
                let y: f64 = args[1].parse().unwrap_or(0.0);
                if args.len() >= 3 && args[2] == "c" {
                    Cmd::MouseClick { x, y }
                } else {
                    Cmd::MouseMove { x, y }
                }
            } else {
                Cmd::Unknown(line.to_string())
            }
        }
        "w" => {
            let ms = if rest.ends_with("ms") {
                rest.trim_end_matches("ms").parse::<u64>().unwrap_or(0)
            } else if rest.ends_with('s') {
                rest.trim_end_matches('s').parse::<f64>().unwrap_or(0.0) as u64 * 1000
            } else {
                rest.parse::<u64>().unwrap_or(0)
            };
            Cmd::Wait(std::time::Duration::from_millis(ms))
        }
        "wf" => {
            // wf "Quick Open" 3s
            // wf !loading 10s
            // wf btn "Save" 5s
            let negate = rest.starts_with('!');
            let rest = if negate { &rest[1..] } else { rest };

            // Parse timeout from last arg (e.g. "3s", "5000ms")
            let parts: Vec<&str> = rest.rsplitn(2, ' ').collect();
            let (query_part, timeout_str) = if parts.len() == 2 {
                (parts[1].trim(), parts[0].trim())
            } else {
                (rest.trim(), "5s") // default 5s timeout
            };

            let timeout_ms = if timeout_str.ends_with("ms") {
                timeout_str.trim_end_matches("ms").parse::<u64>().unwrap_or(5000)
            } else if timeout_str.ends_with('s') {
                timeout_str.trim_end_matches('s').parse::<f64>().unwrap_or(5.0) as u64 * 1000
            } else {
                // Last arg isn't a duration — it's part of the query.
                5000
            };

            // If timeout parse consumed the last arg, use query_part; otherwise whole rest is query.
            let query = if timeout_ms != 5000 || timeout_str.ends_with('s') || timeout_str.ends_with("ms") {
                query_part.trim_matches('"').to_string()
            } else {
                rest.trim_matches('"').to_string()
            };

            Cmd::WaitFor { query, negate, timeout_ms }
        }
        _ => Cmd::Unknown(line.to_string()),
    }
}

/// Execute a command. Returns the original command text as echo (for LLM history).
fn execute_cmd(cmd: &Cmd, line: &str) -> String {
    match cmd {
        Cmd::KeyTap { keycode, flags } => {
            if *flags != 0 {
                key_tap_with_flags(*keycode, *flags);
            } else {
                key_tap(*keycode);
            }
            line.to_string() // echo as-is: "k a" or "k c-s"
        }
        Cmd::TypeString(text) => {
            type_text(text);
            line.to_string()
        }
        Cmd::MouseMove { x, y } => {
            mouse_move_abs(*x, *y);
            format!("m {} {}", *x as i32, *y as i32)
        }
        Cmd::MouseClick { x, y } => {
            mouse_move_abs(*x, *y);
            std::thread::sleep(std::time::Duration::from_millis(30));
            mouse_click(*x, *y);
            format!("m {} {} c", *x as i32, *y as i32)
        }
        Cmd::Wait(d) => {
            std::thread::sleep(*d);
            format!("w {}ms", d.as_millis())
        }
        Cmd::WaitFor { query, negate, timeout_ms } => {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_millis(*timeout_ms);
            let poll_interval = std::time::Duration::from_millis(200);

            loop {
                let tree = get_frontmost_ax_tree();
                let found = tree.to_lowercase().contains(&query.to_lowercase());
                let condition_met = if *negate { !found } else { found };

                if condition_met {
                    // Find the matching line to extract position
                    let matched_line = tree.lines()
                        .find(|l| l.to_lowercase().contains(&query.to_lowercase()))
                        .unwrap_or("")
                        .trim();
                    let elapsed = start.elapsed().as_millis();
                    break format!("[OK wf] matched after {}ms: {}", elapsed, matched_line);
                }

                if start.elapsed() > timeout {
                    let prefix = if *negate { "still present" } else { "not found" };
                    break format!("[TIMEOUT wf] \"{}\" {} after {}ms", query, prefix, timeout_ms);
                }

                std::thread::sleep(poll_interval);
            }
        }
        Cmd::Comment(_) => String::new(),
        Cmd::Unknown(s) => format!("# ERR: {}", s),
    }
}

// ── System Prompt ────────────────────────────────────────────────────────────

/// Load the system prompt from skills/clx-agent/SKILL.md at runtime.
/// Falls back to a minimal built-in prompt if the file is missing.
fn load_system_prompt() -> String {
    // Search relative to the binary, then CWD, then common locations.
    let search_paths = [
        // Next to the binary (e.g. /Users/snomiao/CapsLockX/skills/clx-agent/SKILL.md)
        std::env::current_exe().ok()
            .and_then(|e| e.parent().map(|p| p.join("../skills/clx-agent/SKILL.md"))),
        std::env::current_exe().ok()
            .and_then(|e| e.parent().map(|p| p.join("../../skills/clx-agent/SKILL.md"))),
        // CWD
        Some(std::path::PathBuf::from("skills/clx-agent/SKILL.md")),
        // Absolute fallback
        Some(std::path::PathBuf::from("/Users/snomiao/CapsLockX/skills/clx-agent/SKILL.md")),
    ];

    for path in search_paths.iter().flatten() {
        if let Ok(content) = std::fs::read_to_string(path) {
            tlog(&format!("loaded SKILL.md from {:?}", path));
            return content;
        }
    }

    tlog("WARN: SKILL.md not found, using built-in fallback prompt");
    "You are CLX Agent on macOS. Output CLX commands only.\n\
     k a = tap key, m 400 300 = mouse move, m 400 300 c = click, w 200ms = wait.\n\
     w- = Cmd modifier. Use w-p for Cmd+P, w-s for Cmd+S.\n\
     Output nothing when done.".to_string()
}

// ── LLM Loop ─────────────────────────────────────────────────────────────────

fn load_llm_config() -> Option<capslockx_core::llm_client::LlmConfig> {
    // Try config file first.
    let cfg_path = dirs::config_dir()?.join("CapsLockX").join("config.json");
    let data = std::fs::read_to_string(&cfg_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;

    // Extract best API key.
    let api_key = v.get("llm_api_key").and_then(|k| k.as_str()).unwrap_or("").to_string();
    let model = v.get("llm_model").and_then(|k| k.as_str()).unwrap_or("").to_string();

    // Also check env vars.
    let api_key = if api_key.is_empty() {
        std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .or_else(|_| std::env::var("GROQ_API_KEY"))
            .unwrap_or_default()
    } else {
        api_key
    };

    if api_key.is_empty() {
        return None;
    }

    Some(capslockx_core::llm_client::LlmConfig::from_key_and_model(&api_key, &model))
}

/// Compute a compact diff between two AX trees.
/// Returns empty string if no changes.
fn ax_tree_diff(old: &str, new: &str) -> String {
    if old.trim() == new.trim() { return String::new(); }

    let old_lines: std::collections::HashSet<&str> = old.lines().collect();
    let new_lines: std::collections::HashSet<&str> = new.lines().collect();

    let mut diff = String::new();
    // Removed lines
    for line in old.lines() {
        if !new_lines.contains(line) && !line.trim().is_empty() {
            diff.push_str("- ");
            diff.push_str(line.trim());
            diff.push('\n');
        }
    }
    // Added lines
    for line in new.lines() {
        if !old_lines.contains(line) && !line.trim().is_empty() {
            diff.push_str("+ ");
            diff.push_str(line.trim());
            diff.push('\n');
        }
    }
    diff
}

/// Capture a screenshot as JPEG base64 string.
/// Uses macOS `screencapture` CLI for simplicity and reliability.
fn capture_screenshot_base64() -> Option<String> {
    let tmp = "/tmp/clx-agent-screenshot.jpg";
    let status = std::process::Command::new("screencapture")
        .args(["-x", "-t", "jpg", "-o", tmp]) // -x = no sound, -o = no shadow
        .status()
        .ok()?;
    if !status.success() { return None; }

    // Resize to 512px wide for token efficiency (sips is built into macOS).
    let _ = std::process::Command::new("sips")
        .args(["--resampleWidth", "512", tmp, "--out", tmp])
        .output();

    let data = std::fs::read(tmp).ok()?;
    let _ = std::fs::remove_file(tmp);

    use std::io::Read;
    // Base64 encode
    let b64 = base64_encode(&data);
    Some(b64)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(n & 0x3F) as usize] as char); } else { out.push('='); }
    }
    out
}

fn run_agent_loop(prompt: &str) {
    use capslockx_core::llm_client::{stream_chat, LlmConfig, Message};

    init_logging();
    tlog(&format!("=== new session: {} ===", prompt));

    let config = match load_llm_config() {
        Some(c) => c,
        None => {
            tlog("ERROR: No LLM API key found.");
            tlog("Set GEMINI_API_KEY, OPENAI_API_KEY, or configure in CapsLockX prefs.");
            return;
        }
    };

    tlog(&format!("LLM: {:?} model={}", config.provider, config.model));

    // Read initial AX tree.
    tlog("reading accessibility tree...");
    let mut last_ax_tree = get_frontmost_ax_tree();
    tlog(&format!("AX tree: {} lines, {} bytes", last_ax_tree.lines().count(), last_ax_tree.len()));

    // Capture initial screenshot.
    tlog("capturing screenshot...");
    let mut last_screenshot = capture_screenshot_base64();
    tlog(&format!("screenshot: {} bytes base64", last_screenshot.as_ref().map(|s| s.len()).unwrap_or(0)));

    let system_prompt = load_system_prompt();

    // We'll build Gemini requests manually to support inline images.
    // The generic stream_chat doesn't support images.
    let mut conversation: Vec<serde_json::Value> = Vec::new();

    // First user message: AX tree + screenshot + task.
    let mut first_parts = vec![
        serde_json::json!({"text": format!(
            "## Current Screen (Accessibility Tree)\n```\n{}\n```\n\n## Task\n{}",
            last_ax_tree.trim(), prompt
        )}),
    ];
    if let Some(ref img) = last_screenshot {
        first_parts.push(serde_json::json!({
            "inlineData": { "mimeType": "image/jpeg", "data": img }
        }));
    }
    conversation.push(serde_json::json!({"role": "user", "parts": first_parts}));

    // Multi-turn loop: LLM acts → screenshot → diff → LLM continues.
    const MAX_TURNS: usize = 10;
    for turn in 0..MAX_TURNS {
        tlog(&format!("turn {}/{} — streaming from LLM...", turn + 1, MAX_TURNS));

        let mut llm_output = String::new();
        let mut echo_lines: Vec<String> = Vec::new();
        let mut line_buf = String::new();
        let mut had_action = false;

        // Stream from Gemini with vision support (direct API call).
        let result = stream_gemini_vision(&config, &system_prompt, &conversation, &mut |token| {
            llm_output.push_str(token);
            line_buf.push_str(token);

            while let Some(nl_pos) = line_buf.find('\n') {
                let line = line_buf[..nl_pos].to_string();
                line_buf.drain(..=nl_pos);

                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }

                let cmd = parse_line(trimmed);
                let echo = execute_cmd(&cmd, trimmed);

                if !echo.is_empty() {
                    tlog(&format!("  > {}", echo));
                    echo_lines.push(echo);
                    if !matches!(cmd, Cmd::Comment(_) | Cmd::Wait(_)) {
                        had_action = true;
                    }
                }
            }
        });

        // Execute remaining partial line.
        let remaining = line_buf.trim().to_string();
        if !remaining.is_empty() {
            let cmd = parse_line(&remaining);
            let echo = execute_cmd(&cmd, &remaining);
            if !echo.is_empty() {
                tlog(&format!("  > {}", echo));
                echo_lines.push(echo);
                if !matches!(cmd, Cmd::Comment(_) | Cmd::Wait(_)) {
                    had_action = true;
                }
            }
        }

        if let Err(e) = result {
            tlog(&format!("LLM error: {}", e));
            break;
        }

        // Add LLM's response to conversation.
        if !llm_output.trim().is_empty() {
            conversation.push(serde_json::json!({
                "role": "model",
                "parts": [{"text": llm_output.clone()}]
            }));
        }

        if !had_action {
            tlog("no actions in this turn — done.");
            break;
        }

        // Wait for UI to settle, then capture new screenshot + AX tree.
        std::thread::sleep(std::time::Duration::from_millis(300));

        tlog("re-reading AX tree + screenshot...");
        let new_ax_tree = get_frontmost_ax_tree();
        let diff = ax_tree_diff(&last_ax_tree, &new_ax_tree);
        let new_screenshot = capture_screenshot_base64();

        // Build feedback message with screenshot.
        let feedback_text = if diff.is_empty() {
            tlog("AX tree unchanged");
            format!(
                "## Result\nCommands executed:\n{}\n\nScreen may have changed (see screenshot). Continue or output nothing if done.",
                echo_lines.join("\n")
            )
        } else {
            tlog(&format!("AX tree changed:\n{}", diff.trim()));
            format!(
                "## Result\nCommands executed:\n{}\n\n## Screen Changes\n```\n{}\n```\n\nContinue or output nothing if done.",
                echo_lines.join("\n"), diff.trim()
            )
        };

        let mut feedback_parts = vec![serde_json::json!({"text": feedback_text})];
        if let Some(ref img) = new_screenshot {
            feedback_parts.push(serde_json::json!({
                "inlineData": { "mimeType": "image/jpeg", "data": img }
            }));
        }
        conversation.push(serde_json::json!({"role": "user", "parts": feedback_parts}));

        last_ax_tree = new_ax_tree;
        last_screenshot = new_screenshot;
    }

    tlog("done.");
}

/// Stream from Gemini API with inline image support.
/// This bypasses the generic stream_chat which doesn't handle images.
fn stream_gemini_vision(
    config: &capslockx_core::llm_client::LlmConfig,
    system_prompt: &str,
    conversation: &[serde_json::Value],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    use std::io::{BufRead, BufReader};

    let mut body = serde_json::json!({ "contents": conversation });
    body["systemInstruction"] = serde_json::json!({"parts": [{"text": system_prompt}]});

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        config.model, config.api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Gemini request: {}", e))?;

    let reader = BufReader::new(resp.into_reader());
    let mut full = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read: {}", e))?;
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(text) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    full.push_str(text);
                    on_token(text);
                }
            }
        }
    }
    Ok(full)
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn agent_main(args: &[String]) {

    // --target "App Name": send events to a specific app (not global).
    if let Some(idx) = args.iter().position(|a| a == "--target") {
        if let Some(target_name) = args.get(idx + 1) {
            match find_pid_by_name(target_name) {
                Some(pid) => {
                    unsafe { TARGET_PID = pid; }
                    eprintln!("[clx-agent] targeting: {} (pid={})", target_name, pid);
                    // Activate the target app so it receives events.
                    activate_pid(pid);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                None => {
                    eprintln!("[clx-agent] ERROR: app '{}' not found. Running apps:", target_name);
                    // List running apps for debugging.
                    let script = r#"tell application "System Events" to get name of every application process whose background only is false"#;
                    if let Ok(out) = std::process::Command::new("osascript").arg("-e").arg(script).output() {
                        eprintln!("  {}", String::from_utf8_lossy(&out.stdout).trim());
                    }
                    return;
                }
            }
        }
    }

    // --tree: dump AX tree and exit.
    if args.iter().any(|a| a == "--tree") {
        let tree = get_frontmost_ax_tree();
        print!("{}", tree);
        return;
    }

    // --exec: read CLX commands from stdin and execute.
    if args.iter().any(|a| a == "--exec") {
        eprintln!("[clx-agent] exec mode — reading CLX commands from stdin");
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let cmd = parse_line(&line);
            let echo = execute_cmd(&cmd, &line);
            if !echo.is_empty() {
                eprintln!("{}", echo);
            }
        }
        return;
    }

    // --prompt "do something": run the LLM agent loop with a prompt.
    let prompt_idx = args.iter().position(|a| a == "--prompt");
    if let Some(idx) = prompt_idx {
        let prompt = args.get(idx + 1).cloned().unwrap_or_else(|| {
            eprintln!("Usage: clx-agent --prompt \"click on the Issues tab\"");
            std::process::exit(1);
        });
        run_agent_loop(&prompt);
        return;
    }

    // No flags: interactive mode — read prompt from stdin or args.
    let prompt = if args.len() > 1 && !args[1].starts_with('-') {
        // clx-agent "click on Issues"
        args[1..].join(" ")
    } else {
        // Read from stdin.
        eprint!("[clx-agent] Enter task: ");
        io::stderr().flush().ok();
        let mut p = String::new();
        io::stdin().read_line(&mut p).ok();
        p.trim().to_string()
    };

    if prompt.is_empty() {
        eprintln!("Usage: clx-agent --prompt \"task description\"");
        eprintln!("       clx-agent \"task description\"");
        eprintln!("       clx-agent --tree    (dump accessibility tree)");
        eprintln!("       clx-agent --exec    (execute CLX commands from stdin)");
        return;
    }

    run_agent_loop(&prompt);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    agent_main(&args);
}
