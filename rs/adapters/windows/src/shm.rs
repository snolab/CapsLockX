/// Shared memory IPC – publishes CLX mode state so AHK extensions can read it.
///
/// Layout (256 bytes, named "CapsLockX_SharedState"):
///   0x00  u32  version   (always 1)
///   0x04  u32  mode      (bitmask: CM_FN=1, CM_CLX=2)
///   0x08  u32  rust_pid
///   0x0C  u32  reserved
///   0x10  u32  ahk_pid   (set by write_ahk_pid after spawning AHK)
///   0x14  [u8; 236] reserved
use std::ptr;

use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile,
    FILE_MAP_ALL_ACCESS, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    CreateEventW, OpenEventW, OpenProcess,
    SetEvent, TerminateProcess, WaitForSingleObject,
    EVENT_MODIFY_STATE, PROCESS_TERMINATE, PROCESS_SYNCHRONIZE,
};

const SHM_SIZE: u32 = 256;
const VERSION: u32 = 1;

pub struct SharedState {
    handle: HANDLE,
    ptr: *mut u8,
}

// Safety: the pointer targets a shared memory region written only via volatile
// writes from the hook callback (always on the same thread).
unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}

impl SharedState {
    /// Kill every other `clx-rust.exe` instance on the system, plus any
    /// orphaned AHK child whose pid is recorded in the previous shared
    /// memory header. Strategy:
    ///   1. Signal the named `CapsLockX_Quit` event once — every previous
    ///      instance that's listening will call `app.exit()` cleanly.
    ///   2. Enumerate `clx-rust.exe` processes via Toolhelp32, skipping our
    ///      own pid. For each: wait briefly for graceful exit, then
    ///      `TerminateProcess` as fallback.
    /// This catches multi-instance / orphan / crashed-prior-state cases
    /// that the old shm-pid lookup couldn't see.
    /// Returns `true` if at least one previous instance could not be opened
    /// for termination (typically: it's elevated and we're not). The caller
    /// should relaunch self elevated and retry.
    pub fn kill_previous() -> bool {
        let mut needs_elevation = false;
        crate::hook::debug_log(&format!(
            "[CLX] kill_previous: scanning for old clx-rust.exe (self_pid={})",
            std::process::id()
        ));
        unsafe {
            // ── Step 0: kill any orphaned AHK child recorded in shm ────
            if let Ok(handle) =
                OpenFileMappingW(FILE_MAP_READ.0, false, w!("CapsLockX_SharedState"))
            {
                let view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, SHM_SIZE as usize);
                if !view.Value.is_null() {
                    let ahk_pid =
                        ptr::read_volatile((view.Value as *const u8).add(0x10) as *const u32);
                    let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: view.Value });
                    if ahk_pid != 0 {
                        if let Ok(ahk_proc) = OpenProcess(PROCESS_TERMINATE, false, ahk_pid) {
                            crate::hook::debug_log(&format!("[CLX] killing previous AHK child (pid={ahk_pid})"));
                            let _ = TerminateProcess(ahk_proc, 1);
                            let _ = CloseHandle(ahk_proc);
                        }
                    }
                }
                let _ = CloseHandle(handle);
            }

            // ── Step 1: signal graceful-quit event (wakes all listeners) ─
            if let Ok(evt) = OpenEventW(EVENT_MODIFY_STATE, false, w!("CapsLockX_Quit")) {
                let _ = SetEvent(evt);
                let _ = CloseHandle(evt);
            }

            // ── Step 2: enumerate every clx-rust.exe and kill it ───────
            let self_pid = std::process::id();
            let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) if h != INVALID_HANDLE_VALUE => h,
                _ => return needs_elevation,
            };

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            let mut victims: Vec<u32> = Vec::new();
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                    // Match both the dev binary (`clx-rust.exe`) and the
                    // packaged installer rename (`clx.exe`). Without this,
                    // a hung packaged build that ignores the Quit event
                    // can't be killed by a subsequent launch and the
                    // single-instance guarantee silently regresses.
                    let is_clx = name.eq_ignore_ascii_case("clx-rust.exe")
                        || name.eq_ignore_ascii_case("clx.exe");
                    if is_clx && entry.th32ProcessID != self_pid {
                        victims.push(entry.th32ProcessID);
                    }
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);

            crate::hook::debug_log(&format!(
                "[CLX] kill_previous: found {} victim(s): {:?}",
                victims.len(), victims
            ));

            for pid in victims {
                let proc = match OpenProcess(
                    PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
                    false,
                    pid,
                ) {
                    Ok(h) => h,
                    Err(e) => {
                        // Likely access denied — old instance is elevated
                        // and we're not. Flag it so main can re-launch
                        // self elevated and retry.
                        crate::hook::debug_log(&format!("[CLX] cannot open previous pid={pid} ({e:?}) — needs elevation"));
                        needs_elevation = true;
                        continue;
                    }
                };

                // Wait briefly for the SetEvent above to take effect.
                let r = WaitForSingleObject(proc, 1500);
                if r == WAIT_TIMEOUT {
                    crate::hook::debug_log(&format!("[CLX] force-killing previous pid={pid}"));
                    let _ = TerminateProcess(proc, 1);
                    let _ = WaitForSingleObject(proc, 1000);
                } else {
                    crate::hook::debug_log(&format!("[CLX] previous pid={pid} exited gracefully"));
                }
                let _ = CloseHandle(proc);
            }
        }
        needs_elevation
    }

    /// Create the named quit event. Returns a handle the caller can wait on.
    pub fn create_quit_event() -> Option<HANDLE> {
        unsafe {
            let h = CreateEventW(None, true, false, w!("CapsLockX_Quit")).ok()?;
            // Reset in case a previous instance left it signaled.
            use windows::Win32::System::Threading::ResetEvent;
            let _ = ResetEvent(h);
            Some(h)
        }
    }

    /// Create the named shared memory region and initialise the header.
    pub fn create() -> Option<Self> {
        unsafe {
            let handle = CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                SHM_SIZE,
                w!("CapsLockX_SharedState"),
            )
            .ok()?;

            let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, SHM_SIZE as usize);
            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return None;
            }

            let p = view.Value as *mut u8;

            // version = 1
            ptr::write_volatile(p as *mut u32, VERSION);
            // mode = 0
            ptr::write_volatile(p.add(4) as *mut u32, 0);
            // rust_pid
            ptr::write_volatile(p.add(8) as *mut u32, std::process::id());

            Some(Self { handle, ptr: p })
        }
    }

    /// Update the mode field (offset 0x04) with a volatile write.
    #[inline]
    pub fn write_mode(&self, mode: u32) {
        unsafe {
            ptr::write_volatile(self.ptr.add(4) as *mut u32, mode);
        }
    }

    /// Store the AHK child PID (offset 0x10) so a future instance can kill it.
    pub fn write_ahk_pid(&self, pid: u32) {
        unsafe {
            ptr::write_volatile(self.ptr.add(0x10) as *mut u32, pid);
        }
    }
}

impl Drop for SharedState {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.ptr as *mut _,
            });
            let _ = CloseHandle(self.handle);
        }
    }
}
