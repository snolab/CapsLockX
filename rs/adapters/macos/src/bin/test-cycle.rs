/// Direct-call window cycle test — no key injection.
///
/// Calls CGWindowList + AX APIs directly to list windows, then activates them
/// one by one, verifying frontmost after each activation.
///
/// Does NOT require CapsLockX to be running.

use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::CFString;
use std::thread;
use std::time::Duration;

type CFArrayRef = *const std::ffi::c_void;
type AXUIElementRef = *mut std::ffi::c_void;
type CFStringRefRaw = *const std::ffi::c_void;

extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFArrayRef;
    fn CFArrayGetCount(arr: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: isize) -> *const std::ffi::c_void;
    fn CFDictionaryGetValue(d: *const std::ffi::c_void, k: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn CFNumberGetValue(n: *const std::ffi::c_void, t: i32, p: *mut std::ffi::c_void) -> bool;
    fn CFBooleanGetValue(b: *const std::ffi::c_void) -> bool;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(e: AXUIElementRef, attr: CFStringRefRaw, val: *mut *mut std::ffi::c_void) -> i32;
    fn AXUIElementPerformAction(e: AXUIElementRef, action: CFStringRefRaw) -> i32;
    fn AXUIElementSetAttributeValue(e: AXUIElementRef, attr: CFStringRefRaw, val: *const std::ffi::c_void) -> i32;
    fn _AXUIElementGetWindow(e: AXUIElementRef, wid: *mut u32) -> i32;

    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn objc_msgSend(receiver: *mut std::ffi::c_void, sel: *mut std::ffi::c_void, ...) -> *mut std::ffi::c_void;
}

#[derive(Clone, Debug)]
struct Win {
    pid: i64,
    wid: u32,
    title: String,
}

fn list_onscreen_windows() -> Vec<Win> {
    unsafe {
        let mut result = Vec::new();
        let opts: u32 = (1 << 0) | (1 << 4);
        let list = CGWindowListCopyWindowInfo(opts, 0);
        if list.is_null() { return result; }

        let count = CFArrayGetCount(list);
        let key_pid = CFString::new("kCGWindowOwnerPID");
        let key_layer = CFString::new("kCGWindowLayer");
        let key_wid = CFString::new("kCGWindowNumber");
        let key_onscreen = CFString::new("kCGWindowIsOnscreen");

        // Collect on-screen pids + wids
        let mut onscreen_wids = std::collections::HashSet::new();
        let mut pids = Vec::new();
        let mut pid_set = std::collections::HashSet::new();

        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list, i);
            if dict.is_null() { continue; }
            let lr = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
            if lr.is_null() { continue; }
            let mut layer: i64 = 0;
            CFNumberGetValue(lr, 4, &mut layer as *mut _ as *mut _);
            if layer != 0 { continue; }

            let osr = CFDictionaryGetValue(dict, key_onscreen.as_concrete_TypeRef() as *const _);
            let is_onscreen = if !osr.is_null() { CFBooleanGetValue(osr) } else { true };
            if !is_onscreen { continue; }

            let mut pid: i64 = 0;
            let pr = CFDictionaryGetValue(dict, key_pid.as_concrete_TypeRef() as *const _);
            if !pr.is_null() { CFNumberGetValue(pr, 4, &mut pid as *mut _ as *mut _); }

            let mut wid: i64 = 0;
            let wr = CFDictionaryGetValue(dict, key_wid.as_concrete_TypeRef() as *const _);
            if !wr.is_null() { CFNumberGetValue(wr, 4, &mut wid as *mut _ as *mut _); }

            if wid != 0 { onscreen_wids.insert(wid as u32); }
            if pid_set.insert(pid) { pids.push(pid); }
        }
        CFRelease(list);

        // For each pid, get AX windows
        let attr_windows = CFString::new("AXWindows");
        let attr_title = CFString::new("AXTitle");
        let sel_count = sel_registerName(b"count\0".as_ptr() as *const _);
        let sel_utf8 = sel_registerName(b"UTF8String\0".as_ptr() as *const _);

        for &pid in &pids {
            let app = AXUIElementCreateApplication(pid as i32);
            if app.is_null() { continue; }
            let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
            let err = AXUIElementCopyAttributeValue(
                app, attr_windows.as_concrete_TypeRef() as CFStringRefRaw, &mut value);
            if err != 0 || value.is_null() { CFRelease(app as *const _); continue; }

            let ax_count = {
                let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> usize
                    = std::mem::transmute(objc_msgSend as *const ());
                f(value, sel_count)
            };

            for wi in 0..ax_count {
                let win = CFArrayGetValueAtIndex(value as CFArrayRef, wi as isize);
                if win.is_null() { continue; }

                // Get title
                let mut tval: *mut std::ffi::c_void = std::ptr::null_mut();
                let terr = AXUIElementCopyAttributeValue(
                    win as AXUIElementRef, attr_title.as_concrete_TypeRef() as CFStringRefRaw, &mut tval);
                let title = if terr == 0 && !tval.is_null() {
                    let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void) -> *const std::ffi::c_char
                        = std::mem::transmute(objc_msgSend as *const ());
                    let cstr = f(tval, sel_utf8);
                    let t = if !cstr.is_null() {
                        std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
                    } else { String::new() };
                    CFRelease(tval as *const _);
                    t
                } else { String::new() };

                if title.is_empty() { continue; }

                // Get CGWindowID
                let mut wid: u32 = 0;
                _AXUIElementGetWindow(win as AXUIElementRef, &mut wid);

                // Filter: only include if on-screen
                if wid != 0 && !onscreen_wids.contains(&wid) { continue; }

                result.push(Win { pid, wid, title });
            }

            CFRelease(value as *const _);
            CFRelease(app as *const _);
        }
        result
    }
}

fn frontmost_wid() -> u32 {
    unsafe {
        let opts: u32 = (1 << 0) | (1 << 4);
        let list = CGWindowListCopyWindowInfo(opts, 0);
        if list.is_null() { return 0; }
        let count = CFArrayGetCount(list);
        let key_layer = CFString::new("kCGWindowLayer");
        let key_wid = CFString::new("kCGWindowNumber");
        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list, i);
            if dict.is_null() { continue; }
            let lr = CFDictionaryGetValue(dict, key_layer.as_concrete_TypeRef() as *const _);
            if lr.is_null() { continue; }
            let mut layer: i64 = 0;
            CFNumberGetValue(lr, 4, &mut layer as *mut _ as *mut _);
            if layer != 0 { continue; }
            let wr = CFDictionaryGetValue(dict, key_wid.as_concrete_TypeRef() as *const _);
            if wr.is_null() { continue; }
            let mut wid: i64 = 0;
            CFNumberGetValue(wr, 4, &mut wid as *mut _ as *mut _);
            CFRelease(list);
            return wid as u32;
        }
        CFRelease(list);
        0
    }
}

fn activate(entry: &Win) {
    unsafe {
        // Activate app
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr() as *const _);
        if cls.is_null() { return; }
        let sel_running = sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr() as *const _);
        let app: *mut std::ffi::c_void = {
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, i32) -> *mut std::ffi::c_void
                = std::mem::transmute(objc_msgSend as *const ());
            f(cls, sel_running, entry.pid as i32)
        };
        if !app.is_null() {
            let sel_activate = sel_registerName(b"activateWithOptions:\0".as_ptr() as *const _);
            let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, u64) -> bool
                = std::mem::transmute(objc_msgSend as *const ());
            f(app, sel_activate, 3);
        }

        // Find AX window by wid and raise it
        let ax_app = AXUIElementCreateApplication(entry.pid as i32);
        if ax_app.is_null() { return; }
        let attr_windows = CFString::new("AXWindows");
        let mut value: *mut std::ffi::c_void = std::ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(
            ax_app, attr_windows.as_concrete_TypeRef() as CFStringRefRaw, &mut value);
        if err != 0 || value.is_null() { CFRelease(ax_app as *const _); return; }

        let ax_count = CFArrayGetCount(value as CFArrayRef);
        for wi in 0..ax_count {
            let win = CFArrayGetValueAtIndex(value as CFArrayRef, wi);
            if win.is_null() { continue; }
            let mut wid: u32 = 0;
            _AXUIElementGetWindow(win as AXUIElementRef, &mut wid);
            if wid == entry.wid {
                // AXMain
                let attr_main = CFString::new("AXMain");
                extern "C" { #[allow(non_upper_case_globals)] static kCFBooleanTrue: *const std::ffi::c_void; }
                AXUIElementSetAttributeValue(win as AXUIElementRef, attr_main.as_concrete_TypeRef() as CFStringRefRaw, kCFBooleanTrue);
                // AXRaise
                let action = CFString::new("AXRaise");
                AXUIElementPerformAction(win as AXUIElementRef, action.as_concrete_TypeRef() as CFStringRefRaw);
                // AXFocused
                let attr_focused = CFString::new("AXFocused");
                AXUIElementSetAttributeValue(win as AXUIElementRef, attr_focused.as_concrete_TypeRef() as CFStringRefRaw, kCFBooleanTrue);
                // Re-activate app
                if !app.is_null() {
                    let sel_act2 = sel_registerName(b"activateWithOptions:\0".as_ptr() as *const _);
                    let f: extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, u64) -> bool
                        = std::mem::transmute(objc_msgSend as *const ());
                    f(app, sel_act2, 3);
                }
                break;
            }
        }
        CFRelease(value as *const _);
        CFRelease(ax_app as *const _);
    }
}

fn main() {
    println!("=== Direct Window Cycle Test ===\n");

    let windows = list_onscreen_windows();
    println!("Found {} windows:", windows.len());
    for (i, w) in windows.iter().enumerate() {
        println!("  [{:>2}] wid={:<6} pid={:<6} {}", i, w.wid, w.pid, &w.title.chars().take(60).collect::<String>());
    }

    println!("\n--- Activating each window ---\n");
    println!("{:<4} {:<8} {:<8} {:<8} {}", "#", "wid", "front", "status", "title");
    println!("{}", "-".repeat(90));

    for (i, w) in windows.iter().enumerate() {
        activate(w);
        thread::sleep(Duration::from_millis(200));
        let front = frontmost_wid();
        let status = if front == w.wid { "OK" } else { "FAIL" };
        println!("{:<4} {:<8} {:<8} {:<8} {}", i, w.wid, front, status, &w.title.chars().take(50).collect::<String>());
    }

    println!("\n--- Cycling forward (simulating clx+z) ---\n");
    // Sort by wid (same as CapsLockX does)
    let mut sorted = windows.clone();
    sorted.sort_by_key(|w| w.wid);

    let mut idx = 0;
    for round in 0..sorted.len().min(10) {
        idx = (idx + 1) % sorted.len();
        let w = &sorted[idx];
        activate(w);
        thread::sleep(Duration::from_millis(200));
        let front = frontmost_wid();
        let status = if front == w.wid { "OK" } else { "FAIL" };
        println!("  z#{:<2}: idx={} wid={:<6} front={:<6} {} {:?}",
            round + 1, idx, w.wid, front, status, &w.title.chars().take(40).collect::<String>());
    }

    println!("\n=== Done ===");
}
