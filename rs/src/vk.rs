#![allow(dead_code)]
/// Virtual key code constants (u32, matching KBDLLHOOKSTRUCT.vkCode / Windows SDK)

// --- Trigger keys ---
pub const VK_CAPITAL: u32 = 0x14; // CapsLock
pub const VK_SPACE: u32 = 0x20;
pub const VK_INSERT: u32 = 0x2D;
pub const VK_SCROLL: u32 = 0x91; // ScrollLock
pub const VK_RMENU: u32 = 0xA5;  // Right Alt

// --- Modifier keys (for bypass detection) ---
pub const VK_LSHIFT: u32 = 0xA0;
pub const VK_RSHIFT: u32 = 0xA1;
pub const VK_LCONTROL: u32 = 0xA2;
pub const VK_RCONTROL: u32 = 0xA3;
pub const VK_LMENU: u32 = 0xA4; // Left Alt
pub const VK_LWIN: u32 = 0x5B;
pub const VK_RWIN: u32 = 0x5C;

// --- Navigation / editing ---
pub const VK_RETURN: u32 = 0x0D;
pub const VK_TAB: u32 = 0x09;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_PRIOR: u32 = 0x21; // Page Up
pub const VK_NEXT: u32 = 0x22;  // Page Down
pub const VK_HOME: u32 = 0x24;
pub const VK_END: u32 = 0x23;
pub const VK_SHIFT: u32 = 0x10;

// --- Alpha keys (A=0x41 … Z=0x5A, same as ASCII uppercase) ---
pub const VK_A: u32 = 0x41;
pub const VK_D: u32 = 0x44;
pub const VK_E: u32 = 0x45;
pub const VK_F: u32 = 0x46;
pub const VK_G: u32 = 0x47;
pub const VK_H: u32 = 0x48;
pub const VK_I: u32 = 0x49;
pub const VK_J: u32 = 0x4A;
pub const VK_K: u32 = 0x4B;
pub const VK_L: u32 = 0x4C;
pub const VK_N: u32 = 0x4E;
pub const VK_O: u32 = 0x4F;
pub const VK_P: u32 = 0x50;
pub const VK_Q: u32 = 0x51;
pub const VK_R: u32 = 0x52;
pub const VK_S: u32 = 0x53;
pub const VK_T: u32 = 0x54;
pub const VK_U: u32 = 0x55;
pub const VK_W: u32 = 0x57;
pub const VK_Y: u32 = 0x59;

// --- Function keys ---
pub const VK_F1: u32 = 0x70;
pub const VK_F2: u32 = 0x71;
pub const VK_F3: u32 = 0x72;
pub const VK_F4: u32 = 0x73;
pub const VK_F5: u32 = 0x74;
pub const VK_F6: u32 = 0x75;
pub const VK_F7: u32 = 0x76;
pub const VK_F8: u32 = 0x77;
pub const VK_F9: u32 = 0x78;
pub const VK_F10: u32 = 0x79;
pub const VK_F11: u32 = 0x7A;
pub const VK_F12: u32 = 0x7B;

// --- Media keys ---
pub const VK_MEDIA_NEXT_TRACK: u32 = 0xB0;
pub const VK_MEDIA_PREV_TRACK: u32 = 0xB1;
pub const VK_MEDIA_STOP: u32 = 0xB2;
pub const VK_MEDIA_PLAY_PAUSE: u32 = 0xB3;
pub const VK_VOLUME_MUTE: u32 = 0xAD;
pub const VK_VOLUME_DOWN: u32 = 0xAE;
pub const VK_VOLUME_UP: u32 = 0xAF;

// --- KBDLLHOOKSTRUCT.flags bits ---
pub const LLKHF_UP: u32 = 0x80;       // key-up event
pub const LLKHF_INJECTED: u32 = 0x10; // injected by SendInput etc.

/// Returns true if `vk` is a modifier key (LCtrl/RCtrl/LShift/RShift/LAlt/RAlt/LWin/RWin)
pub fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        VK_LSHIFT | VK_RSHIFT
        | VK_LCONTROL | VK_RCONTROL
        | VK_LMENU | VK_RMENU
        | VK_LWIN | VK_RWIN
    )
}
