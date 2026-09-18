//! Core-side bridge to the out-of-process voice host (`clx-voice.exe`).
//!
//! The always-on hook process (`clx.exe`) must stay a thin, stable hotkey
//! trigger, so the voice feature (otoji + STT + PTT typing) lives in a separate
//! `clx-voice.exe`. On Space+V the core just signals a named event and, if the
//! host isn't already running, spawns it. The host owns everything else and can
//! be rebuilt / restarted without touching the running core.
//!
//! The core CREATES the two key events (not the host) so they exist before the
//! host starts: an auto-reset event stays signaled until a wait consumes it, so
//! a key-down fired during the host's cold start is delivered, not lost.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{
    CreateEventW, OpenMutexW, SetEvent, SYNCHRONIZATION_ACCESS_RIGHTS,
};

/// Standard SYNCHRONIZE access right (winnt.h) — enough to probe a mutex's
/// existence via OpenMutexW.
const SYNCHRONIZE: SYNCHRONIZATION_ACCESS_RIGHTS = SYNCHRONIZATION_ACCESS_RIGHTS(0x0010_0000);

use capslockx_core::platform::voice_ipc::{ALIVE_MUTEX, KEY_DOWN_EVENT, KEY_UP_EVENT};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Raw handle wrapper so the two core-owned event handles can live in a static.
struct Events {
    down: usize,
    up: usize,
}
// SAFETY: the handles are only ever passed to SetEvent, which is thread-safe.
unsafe impl Send for Events {}
unsafe impl Sync for Events {}

static EVENTS: OnceLock<Option<Events>> = OnceLock::new();
/// Last host spawn attempt (ms since an arbitrary epoch). Debounces re-spawns
/// while a freshly launched host is still coming up.
static LAST_SPAWN_MS: AtomicU64 = AtomicU64::new(0);

fn events() -> Option<&'static Events> {
    EVENTS
        .get_or_init(|| unsafe {
            // Auto-reset (manual_reset = false), initially unsignaled.
            let dn = to_wide(KEY_DOWN_EVENT);
            let up = to_wide(KEY_UP_EVENT);
            let down = CreateEventW(None, false, false, PCWSTR(dn.as_ptr())).ok()?;
            let up = CreateEventW(None, false, false, PCWSTR(up.as_ptr())).ok()?;
            Some(Events {
                down: down.0 as usize,
                up: up.0 as usize,
            })
        })
        .as_ref()
}

/// True if a voice host is currently running (its alive-mutex exists).
fn host_alive() -> bool {
    unsafe {
        let name = to_wide(ALIVE_MUTEX);
        match OpenMutexW(SYNCHRONIZE, false, PCWSTR(name.as_ptr())) {
            Ok(h) => {
                let _ = CloseHandle(h);
                true
            }
            Err(_) => false,
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Path to `clx-voice.exe` next to the running `clx.exe`.
fn host_exe() -> Option<std::path::PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidate = dir.join("clx-voice.exe");
    candidate.is_file().then_some(candidate)
}

/// Spawn the voice host if it isn't running (debounced). Returns false when the
/// host binary is missing — the caller then falls back to in-process voice.
fn ensure_host() -> bool {
    if host_alive() {
        return true;
    }
    let Some(exe) = host_exe() else {
        return false; // no clx-voice.exe → caller uses in-process fallback
    };
    // Debounce: don't stack spawns while a host is still starting.
    let last = LAST_SPAWN_MS.load(Ordering::Relaxed);
    let now = now_ms();
    if now.saturating_sub(last) < 3000 {
        return true; // a spawn is already in flight
    }
    LAST_SPAWN_MS.store(now, Ordering::Relaxed);
    std::thread::Builder::new()
        .name("clx-voice-spawn".into())
        .spawn(move || {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new(exe)
                .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
                .spawn();
        })
        .ok();
    true
}

/// Spawn the voice host now (if not already running) without sending a key —
/// used at core startup when prewarm is enabled, so the host can load the model
/// and open the mic before the first Space+V instead of on it.
pub fn prewarm_host() {
    ensure_host();
}

/// Deliver a Space+V key-down/up to the voice host. Returns `true` when the
/// host path is usable (event signaled, host running or being spawned); `false`
/// only when there is no `clx-voice.exe` to run — the core then handles voice
/// in-process so the feature never simply disappears.
pub fn delegate(down: bool) -> bool {
    let Some(ev) = events() else {
        return false;
    };
    // Signal first (fast, safe from the hook thread); the auto-reset event
    // stays set until the host consumes it, so a cold-start key-down survives.
    unsafe {
        let h = HANDLE((if down { ev.down } else { ev.up }) as *mut _);
        let _ = SetEvent(h);
    }
    ensure_host()
}
