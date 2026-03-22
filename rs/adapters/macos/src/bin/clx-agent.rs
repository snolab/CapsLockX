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

// ── macOS FFI ────────────────────────────────────────────────────────────────

type AXUIElementRef = *mut c_void;
type CFStringRef = *mut c_void;
type CFArrayRef = *mut c_void;
type CFTypeRef = *mut c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef, attribute: CFStringRef, value: *mut CFTypeRef,
    ) -> i32;
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
    fn CGEventKeyboardSetUnicodeString(event: *mut c_void, len: u32, chars: *const u16);
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
    let err = AXUIElementCopyAttributeValue(elem, attr_cf, &mut val);
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
    let err = AXUIElementCopyAttributeValue(elem, attr_cf, &mut val);
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

extern "C" {
    fn AXValueGetValue(value: CFTypeRef, value_type: i32, value_ptr: *mut c_void) -> bool;
}

unsafe fn ax_attr_point(elem: AXUIElementRef, attr: &str) -> Option<CGPoint> {
    let val = ax_attr_ref(elem, attr)?;
    let mut point = CGPoint { x: 0.0, y: 0.0 };
    let ok = AXValueGetValue(val, 1, &mut point as *mut _ as *mut c_void);
    CFRelease(val);
    if ok { Some(point) } else { None }
}

unsafe fn ax_attr_size(elem: AXUIElementRef, attr: &str) -> Option<CGSize> {
    let val = ax_attr_ref(elem, attr)?;
    let mut size = CGSize { w: 0.0, h: 0.0 };
    let ok = AXValueGetValue(val, 2, &mut size as *mut _ as *mut c_void);
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

fn check_ax_permission() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

fn get_frontmost_ax_tree() -> String {
    if !check_ax_permission() {
        return "[AX] ERROR: Accessibility permission not granted.\n\
                Grant permission in: System Settings → Privacy & Security → Accessibility\n\
                Add this binary: clx-agent\n".to_string();
    }

    // Run AX tree reading with a timeout to prevent hanging.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let tree = get_frontmost_ax_tree_inner();
        let _ = tx.send(tree);
    });
    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(tree) => tree,
        Err(_) => "[AX] ERROR: Accessibility tree read timed out (5s).\n".to_string(),
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

        let app_elem = AXUIElementCreateApplication(pid);
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
            CGEventPost(0, event); // kCGHIDEventTap = 0
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
        if !down.is_null() { CGEventPost(0, down); CFRelease(down); }
        std::thread::sleep(std::time::Duration::from_millis(30));
        if !up.is_null() { CGEventPost(0, up); CFRelease(up); }
    }
}

fn key_tap(keycode: u16) {
    unsafe {
        let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, false);
        if !down.is_null() { CGEventPost(0, down); CFRelease(down); }
        if !up.is_null() { CGEventPost(0, up); CFRelease(up); }
    }
}

fn key_tap_with_flags(keycode: u16, flags: u64) {
    unsafe {
        let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, false);
        if !down.is_null() {
            CGEventSetFlags(down, flags);
            CGEventPost(0, down);
            CFRelease(down);
        }
        if !up.is_null() {
            CGEventSetFlags(up, flags);
            CGEventPost(0, up);
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
                CGEventPost(0, down);
                CFRelease(down);
            }
            if !up.is_null() {
                CGEventKeyboardSetUnicodeString(up, utf16.len() as u32, utf16.as_ptr());
                CGEventPost(0, up);
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
                return Cmd::TypeString(rest[1..rest.len()-1].to_string());
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
        Cmd::Comment(_) => String::new(),
        Cmd::Unknown(s) => format!("# ERR: {}", s),
    }
}

// ── System Prompt ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = r#"You are CLX Agent. You control the computer by outputting CLX commands.
Commands execute IMMEDIATELY as you stream them. Each line runs instantly.

## Commands
k a          tap key 'a'
k A          tap Shift+A (uppercase = shift)
k ret        tap Enter
k esc        tap Escape
k tab        tap Tab
k space      tap Space
k bksp       tap Backspace
k c-c        Ctrl+C (c=ctrl, s=shift, a=alt, w=cmd)
k c-a        Ctrl+A (select all)
k w-space    Cmd+Space
k "text"     type string
m 400 300    move mouse to (400,300)
m 400 300 c  move to (400,300) and click
w 200ms      wait 200 milliseconds
w 1s         wait 1 second

## Rules
1. Output ONLY CLX commands. No explanations, no prose, no markdown.
2. After clicking, add: w 200ms (wait for UI response).
3. Use @x,y positions from the accessibility tree to click elements.
4. Keep commands minimal — fewer lines = faster execution.
5. You will see your executed commands echoed back. Errors show as: # ERR: ...
6. After key actions, you may see [AX] updates showing new focus/state.
"#;

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

fn run_agent_loop(prompt: &str) {
    use capslockx_core::llm_client::{stream_chat, Message};

    let config = match load_llm_config() {
        Some(c) => c,
        None => {
            eprintln!("[clx-agent] ERROR: No LLM API key found.");
            eprintln!("[clx-agent] Set GEMINI_API_KEY, OPENAI_API_KEY, or configure in CapsLockX prefs.");
            return;
        }
    };

    eprintln!("[clx-agent] LLM: {:?} model={}", config.provider, config.model);

    // Read AX tree of the frontmost app.
    eprintln!("[clx-agent] reading accessibility tree...");
    let ax_tree = get_frontmost_ax_tree();
    eprintln!("{}", ax_tree);

    // Build messages.
    let mut messages = vec![
        Message { role: "system".into(), content: SYSTEM_PROMPT.to_string() },
        Message {
            role: "user".into(),
            content: format!(
                "## Current Screen (Accessibility Tree)\n```\n{}\n```\n\n## Task\n{}",
                ax_tree.trim(), prompt
            ),
        },
    ];

    // Accumulate the LLM's full response + our echo history for multi-turn.
    let mut llm_output = String::new();
    let mut echo_history = String::new();
    let mut line_buf = String::new();

    eprintln!("[clx-agent] streaming from LLM...\n");

    let result = stream_chat(&config, &messages, &mut |token| {
        // Accumulate tokens into lines, execute each complete line.
        llm_output.push_str(token);
        line_buf.push_str(token);

        // Process complete lines.
        while let Some(nl_pos) = line_buf.find('\n') {
            let line = line_buf[..nl_pos].to_string();
            line_buf.drain(..=nl_pos);

            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            // Parse and execute.
            let cmd = parse_line(trimmed);
            let echo = execute_cmd(&cmd, trimmed);

            if !echo.is_empty() {
                eprintln!("  > {}", echo);
                echo_history.push_str(&echo);
                echo_history.push('\n');
            }
        }
    });

    // Execute any remaining partial line.
    let remaining = line_buf.trim().to_string();
    if !remaining.is_empty() {
        let cmd = parse_line(&remaining);
        let echo = execute_cmd(&cmd, &remaining);
        if !echo.is_empty() {
            eprintln!("  > {}", echo);
            echo_history.push_str(&echo);
            echo_history.push('\n');
        }
    }

    match result {
        Ok(_) => eprintln!("\n[clx-agent] done."),
        Err(e) => eprintln!("\n[clx-agent] LLM error: {}", e),
    }

    // If there were errors, we could do a follow-up turn here.
    if echo_history.contains("# ERR:") {
        eprintln!("[clx-agent] some commands had errors — a follow-up turn could retry.");
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn agent_main(args: &[String]) {

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
