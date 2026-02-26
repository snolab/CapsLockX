/// CLX-MediaKeys – F5–F11 → media / volume keys while CLX is active.
///
/// Mirrors AHK module `CLX-MediaKeys.ahk`.
/// These are only active in FN mode (trigger held) OR CLX locked mode,
/// matching `#if !!(CapsLockXMode & CM_FN) || !!(CapsLockXMode & CM_CapsLockX)`.
use crate::input;
use crate::vk::*;

pub fn on_key_down(vk: u32) -> bool {
    match vk {
        VK_F5  => { input::tap_extended_key(VK_MEDIA_PLAY_PAUSE); true }
        VK_F6  => { input::tap_extended_key(VK_MEDIA_PREV_TRACK); true }
        VK_F7  => { input::tap_extended_key(VK_MEDIA_NEXT_TRACK); true }
        VK_F8  => { input::tap_extended_key(VK_MEDIA_STOP);       true }
        VK_F9  => { input::tap_extended_key(VK_VOLUME_UP);        true }
        VK_F10 => { input::tap_extended_key(VK_VOLUME_DOWN);      true }
        VK_F11 => { input::tap_extended_key(VK_VOLUME_MUTE);      true }
        _ => false,
    }
}

pub fn on_key_up(_vk: u32) -> bool {
    // Media keys are instantaneous taps; no release handling needed.
    false
}
