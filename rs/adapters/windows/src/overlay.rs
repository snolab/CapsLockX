//! Brainstorm streaming overlay — a separate `clx overlay-window` process that
//! renders the live LLM response.
//!
//! Like prefs/prompt, it's a separate process so its WebView2 window can never
//! interfere with the hook process's WH_KEYBOARD_LL. The hook process (writer)
//! publishes the current text into a named shared-memory region; the overlay
//! process (reader) polls it ~10×/s and updates the DOM. The window is given
//! WS_EX_NOACTIVATE so showing it never steals the user's focus.
//!
//! Shared region `CapsLockX_OverlayText` (64 KiB):
//!   0x00 u32 seq      — bumped on every update; reader re-renders when it changes
//!   0x04 u32 visible  — 1 = show window, 0 = hide
//!   0x08 u32 len      — UTF-8 byte length of the text
//!   0x0C ..  text     — UTF-8 bytes

use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};

const OVERLAY_SHM_SIZE: u32 = 65536;
const HEADER: usize = 12; // seq(4) + visible(4) + len(4)
const MAX_TEXT: usize = OVERLAY_SHM_SIZE as usize - HEADER;

// ── Writer side (hook process) ──────────────────────────────────────────────

struct Writer {
    _handle: HANDLE,
    ptr: *mut u8,
}
// Safety: the region is written only via volatile writes, always from the
// brainstorm background thread (serialized by the module's own state machine).
unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

static WRITER: OnceLock<Option<Writer>> = OnceLock::new();
static SPAWNED: AtomicBool = AtomicBool::new(false);

fn writer() -> Option<&'static Writer> {
    WRITER
        .get_or_init(|| unsafe {
            let handle = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                OVERLAY_SHM_SIZE,
                w!("CapsLockX_OverlayText"),
            )
            .ok()?;
            let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, OVERLAY_SHM_SIZE as usize);
            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return None;
            }
            let p = view.Value as *mut u8;
            ptr::write_volatile(p as *mut u32, 0); // seq
            ptr::write_volatile(p.add(4) as *mut u32, 0); // visible
            ptr::write_volatile(p.add(8) as *mut u32, 0); // len
            Some(Writer {
                _handle: handle,
                ptr: p,
            })
        })
        .as_ref()
}

#[inline]
fn bump_seq(w: &Writer) {
    unsafe {
        let seq = ptr::read_volatile(w.ptr as *const u32).wrapping_add(1);
        ptr::write_volatile(w.ptr as *mut u32, seq);
    }
}

/// Publish `text` and make the overlay visible. Spawns the overlay process on
/// first use. Called from the hook process via `WinPlatform::show_brainstorm_overlay`.
pub fn show(text: &str) {
    let Some(w) = writer() else { return };
    let bytes = text.as_bytes();
    let len = bytes.len().min(MAX_TEXT);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), w.ptr.add(HEADER), len);
        ptr::write_volatile(w.ptr.add(8) as *mut u32, len as u32);
        ptr::write_volatile(w.ptr.add(4) as *mut u32, 1); // visible
    }
    bump_seq(w);
    ensure_overlay_process();
}

/// Hide the overlay window (keeps the overlay process alive for next time).
pub fn hide() {
    let Some(w) = writer() else { return };
    unsafe {
        ptr::write_volatile(w.ptr.add(4) as *mut u32, 0); // visible = 0
    }
    bump_seq(w);
}

fn ensure_overlay_process() {
    if SPAWNED.swap(true, Ordering::SeqCst) {
        return; // already spawned once this session
    }
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe)
            .arg("overlay-window")
            .spawn();
    }
}

/// Dev smoke test (`clx overlay-selftest`): act as the writer, stream a few
/// updates into the overlay, then hide — verifies the full spawn → render →
/// hide → self-terminate pipeline without needing an LLM. This process stays
/// the writer (keeps the region mapped) for its whole run.
pub fn selftest() {
    show("Thinking…");
    std::thread::sleep(std::time::Duration::from_millis(900));
    let mut acc = String::new();
    for word in [
        "Local ",
        "LLM ",
        "tokens ",
        "streaming ",
        "into ",
        "the ",
        "overlay ",
        "panel.",
    ] {
        acc.push_str(word);
        show(&acc);
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    hide();
    std::thread::sleep(std::time::Duration::from_millis(600));
}

// ── Reader side (overlay process) ────────────────────────────────────────────

/// Read the current overlay state. Returns `None` if the region is gone (the
/// main clx process exited) — the overlay uses this to self-terminate.
pub fn read_snapshot() -> Option<(u32, bool, String)> {
    unsafe {
        let handle = OpenFileMappingW(FILE_MAP_READ.0, false, w!("CapsLockX_OverlayText")).ok()?;
        let view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, OVERLAY_SHM_SIZE as usize);
        if view.Value.is_null() {
            let _ = CloseHandle(handle);
            return None;
        }
        let p = view.Value as *const u8;
        let seq = ptr::read_volatile(p as *const u32);
        let visible = ptr::read_volatile(p.add(4) as *const u32) != 0;
        let len = (ptr::read_volatile(p.add(8) as *const u32) as usize).min(MAX_TEXT);
        let mut buf = vec![0u8; len];
        ptr::copy_nonoverlapping(p.add(HEADER), buf.as_mut_ptr(), len);
        let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
            Value: view.Value as *mut _,
        });
        let _ = CloseHandle(handle);
        Some((seq, visible, String::from_utf8_lossy(&buf).into_owned()))
    }
}

// ── Overlay subprocess (`clx overlay-window`) ────────────────────────────────

#[derive(serde::Serialize)]
struct Snapshot {
    /// False once the main process exits → the overlay should quit.
    alive: bool,
    visible: bool,
    seq: u32,
    text: String,
}

#[tauri::command]
fn overlay_poll() -> Snapshot {
    match read_snapshot() {
        Some((seq, visible, text)) => Snapshot {
            alive: true,
            visible,
            seq,
            text,
        },
        None => Snapshot {
            alive: false,
            visible: false,
            seq: 0,
            text: String::new(),
        },
    }
}

#[tauri::command]
fn overlay_quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn overlay_set_visible(window: tauri::WebviewWindow, visible: bool) {
    if visible {
        let _ = window.show();
    } else {
        let _ = window.hide();
    }
}

/// Entry point for the `overlay-window` subcommand. Blocks until clx exits.
pub fn run() {
    if !acquire_single_instance() {
        return; // an overlay is already running
    }

    use tauri::webview::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    // Top-right placement, matching the macOS overlay panel.
    let (win_w, win_h) = (460.0_f64, 320.0_f64);
    let (x, y) = top_right_position(win_w, win_h);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            overlay_poll,
            overlay_quit,
            overlay_set_visible
        ])
        .setup(move |app| {
            WebviewWindowBuilder::new(
                app.handle(),
                "overlay",
                WebviewUrl::App("overlay.html".into()),
            )
            .title("CapsLockX Brainstorm Overlay")
            .inner_size(win_w, win_h)
            .position(x, y)
            .decorations(false)
            .always_on_top(true)
            .focused(false)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false) // shown by JS when visible=1 arrives
            .build()?;

            // Apply WS_EX_NOACTIVATE so showing the overlay never steals focus.
            // Done from a short-lived thread that locates the window by title
            // (avoids coupling to tauri's `windows`-crate version).
            std::thread::spawn(apply_noactivate_when_ready);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("tauri build error (overlay window)")
        .run(|_app, _event| {});
}

/// Primary-screen top-right position with a small margin.
fn top_right_position(win_w: f64, win_h: f64) -> (f64, f64) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};
    let _ = win_h;
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f64;
    let x = (screen_w - win_w - 24.0).max(0.0);
    (x, 48.0)
}

/// Find the overlay window by title and OR in WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW
/// | WS_EX_TOPMOST. Retries briefly because the HWND may not exist the instant
/// the builder returns.
fn apply_noactivate_when_ready() {
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    };
    for _ in 0..50 {
        unsafe {
            if let Ok(hwnd) = FindWindowW(PCWSTR::null(), w!("CapsLockX Brainstorm Overlay")) {
                if !hwnd.0.is_null() {
                    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    let add = (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0) as isize;
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | add);
                    return;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Process-wide single-instance guard (named mutex). The handle is leaked so the
/// mutex is held for the process lifetime.
fn acquire_single_instance() -> bool {
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;
    unsafe {
        match CreateMutexW(None, true, w!("CapsLockX_OverlayWindow")) {
            Ok(handle) => {
                let already = GetLastError() == ERROR_ALREADY_EXISTS;
                std::mem::forget(handle);
                !already
            }
            Err(_) => true,
        }
    }
}
