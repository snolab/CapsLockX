/// Shared memory IPC – publishes CLX mode state so AHK extensions can read it.
///
/// Layout (256 bytes, named "CapsLockX_SharedState"):
///   0x00  u32  version   (always 1)
///   0x04  u32  mode      (bitmask: CM_FN=1, CM_CLX=2)
///   0x08  u32  rust_pid
///   0x0C  [u8; 244] reserved
use std::ptr;

use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile,
    FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
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
