//! System-wide light/dark appearance switching.
//!
//! Primary path is SkyLight's private `SLS*AppearanceThemeLegacy` pair, resolved
//! with `dlopen`/`dlsym` rather than a link-time dependency: the symbols are
//! private, so a future macOS may drop them, and a missing symbol must degrade
//! to the fallback instead of failing to launch. It flips the appearance
//! instantly and needs no TCC grant.
//!
//! Fallback is `osascript` driving System Events, which is stable across
//! releases but costs a few hundred ms and prompts once for Automation.

use std::ffi::{c_void, CString};
use std::sync::OnceLock;

const SKYLIGHT: &str =
    "/System/Library/PrivateFrameworks/SkyLight.framework/Versions/A/SkyLight\0";

type GetThemeFn = unsafe extern "C" fn() -> bool;
type SetThemeFn = unsafe extern "C" fn(bool);

struct SkyLightAppearance {
    get: GetThemeFn,
    set: SetThemeFn,
}

// The function pointers are plain code addresses in a never-unloaded framework.
unsafe impl Send for SkyLightAppearance {}
unsafe impl Sync for SkyLightAppearance {}

fn skylight() -> Option<&'static SkyLightAppearance> {
    static CELL: OnceLock<Option<SkyLightAppearance>> = OnceLock::new();
    CELL.get_or_init(|| unsafe {
        let handle = libc::dlopen(SKYLIGHT.as_ptr() as *const libc::c_char, libc::RTLD_LAZY);
        if handle.is_null() {
            return None;
        }
        let get = dlsym(handle, "SLSGetAppearanceThemeLegacy")?;
        let set = dlsym(handle, "SLSSetAppearanceThemeLegacy")?;
        Some(SkyLightAppearance {
            get: std::mem::transmute::<*mut c_void, GetThemeFn>(get),
            set: std::mem::transmute::<*mut c_void, SetThemeFn>(set),
        })
    })
    .as_ref()
}

unsafe fn dlsym(handle: *mut c_void, name: &str) -> Option<*mut c_void> {
    let sym = CString::new(name).ok()?;
    let ptr = libc::dlsym(handle, sym.as_ptr());
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

/// Flip the OS appearance between light and dark.
///
/// Runs on a detached thread: this is called from the CGEventTap callback, and
/// a slow callback gets the tap disabled by the system.
pub fn toggle() {
    std::thread::spawn(|| {
        if let Some(sl) = skylight() {
            unsafe {
                let dark = (sl.get)();
                (sl.set)(!dark);
            }
            eprintln!("[CLX] dark mode toggled (SkyLight)");
            return;
        }
        toggle_via_osascript();
    });
}

fn toggle_via_osascript() {
    let script = r#"tell application "System Events" to tell appearance preferences to set dark mode to not dark mode"#;
    match std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
    {
        Ok(out) if out.status.success() => eprintln!("[CLX] dark mode toggled (osascript)"),
        Ok(out) => eprintln!(
            "[CLX] dark mode toggle failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(e) => eprintln!("[CLX] dark mode toggle failed: {e}"),
    }
}
