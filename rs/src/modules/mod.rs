pub mod edit;
pub mod media;
pub mod mouse;
pub mod window_manager;

use crate::vk::*;

/// Eagerly spin up all background threads owned by the modules.
/// Call this before installing the keyboard hook.
pub fn init() {
    edit::init();
    mouse::init();
    // media has no background threads
}

/// Called when a mapped key is pressed while CLX is active.
/// Returns true if the key was handled (hook should suppress it).
pub fn on_key_down(vk: u32) -> bool {
    edit::on_key_down(vk) || mouse::on_key_down(vk) || media::on_key_down(vk)
        || window_manager::on_key_down(vk)
}

/// Called when a key is released while CLX was active.
/// Returns true if handled.
pub fn on_key_up(vk: u32) -> bool {
    edit::on_key_up(vk) || mouse::on_key_up(vk) || media::on_key_up(vk)
        || window_manager::on_key_up(vk)
}

/// Returns true if `vk` is handled by any module in CLX mode.
/// Used to decide whether to suppress the key event at hook time.
pub fn is_mapped_key(vk: u32) -> bool {
    matches!(
        vk,
        // CLX-Edit
        VK_H | VK_J | VK_K | VK_L
        | VK_Y | VK_U | VK_I | VK_O
        | VK_G | VK_T
        | VK_P | VK_N
        // CLX-Mouse
        | VK_W | VK_A | VK_S | VK_D
        | VK_Q | VK_E
        | VK_R | VK_F
        // CLX-MediaKeys (Fn only, not interfering with CLX-Edit F-keys)
        | VK_F5 | VK_F6 | VK_F7 | VK_F8
        | VK_F9 | VK_F10 | VK_F11
    ) || window_manager::is_mapped_key(vk)
}
