// Hide console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CapsLockX – Windows adapter entry point.
//!
//! Build (debug):   cargo build -p capslockx-windows
//! Build (release): cargo build -p capslockx-windows --release

mod hook;
mod output;
mod vk;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

fn main() {
    eprintln!("[CLX] CapsLockX Windows adapter starting…");

    hook::install_hook();

    eprintln!("[CLX] running – hold CapsLock/Space to activate");
    eprintln!("[CLX] close window or send WM_QUIT to exit");

    unsafe {
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0);
            if ret.0 == 0 || ret.0 == -1 { break; }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    hook::uninstall_hook();
    eprintln!("[CLX] exited cleanly");
}
