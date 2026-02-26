// Hide console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CapsLockX – Rust core
//!
//! Entry point: eagerly initialises module threads, installs the low-level
//! keyboard hook, then runs the Win32 message loop on the main thread
//! (WH_KEYBOARD_LL callbacks are delivered to the installing thread).
//!
//! Build (debug):   cargo build
//! Build (release): cargo build --release

mod acc_model;
mod hook;
mod input;
mod modules;
mod state;
mod vk;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

fn main() {
    eprintln!("[CLX] CapsLockX Rust core starting…");

    // Spin up AccModel background threads before the hook fires any events
    modules::init();

    // Install the WH_KEYBOARD_LL hook
    hook::install_hook();

    eprintln!("[CLX] running – hold CapsLock/Space to activate");
    eprintln!("[CLX] close window or send WM_QUIT to exit");

    // Win32 message loop (required to receive WH_KEYBOARD_LL callbacks)
    unsafe {
        let mut msg = MSG::default();
        loop {
            // NULL hwnd = receive all messages for this thread
            let ret = GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0);
            if ret.0 == 0 || ret.0 == -1 {
                break; // WM_QUIT received or error
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    hook::uninstall_hook();
    eprintln!("[CLX] exited cleanly");
}
