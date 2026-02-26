/// Global CapsLockX mode state (all atomics → no locking in the hot path)
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

// ──────────────────────────────── mode constants ─────────────────────────────
/// Normal keyboard state – trigger key is not held
pub const CM_NORMAL: u32 = 0;
/// FN mode – trigger key is currently held down
pub const CM_FN: u32 = 1;
/// CapsLockX locked mode – entered via long-press or CapsLock+Space
pub const CM_CLX: u32 = 2;

// ──────────────────────────────── config flags ───────────────────────────────
/// Which physical keys act as the CLX trigger (defaults match AHK config)
pub const CFG_USE_CAPSLOCK: bool = true;
pub const CFG_USE_SPACE: bool = true;
pub const CFG_USE_INSERT: bool = false;
pub const CFG_USE_SCROLL_LOCK: bool = false;
pub const CFG_USE_RALT: bool = false;

// ──────────────────────────────── global atomics ─────────────────────────────
/// Current CLX mode bitmask (CM_NORMAL / CM_FN / CM_CLX)
pub static CLX_MODE: AtomicU32 = AtomicU32::new(CM_NORMAL);

/// Whether CLX processing is paused (e.g. user pressed CLX+Pause)
pub static CLX_PAUSED: AtomicBool = AtomicBool::new(false);

/// VK code of the trigger key currently being held (0 = none)
pub static TRIGGER_VK: AtomicU32 = AtomicU32::new(0);

/// VK code of the key pressed just before the current key (AHK's A_PriorKey)
pub static PRIOR_VK: AtomicU32 = AtomicU32::new(0);

/// Timestamp (GetTickCount64 value) when the trigger key was pressed
pub static TRIGGER_PRESS_TICK: AtomicU64 = AtomicU64::new(0);

/// Whether any non-trigger key was acted on while the trigger was held
pub static CLX_FN_ACTED: AtomicBool = AtomicBool::new(false);

// ──────────────────────────────── helpers ────────────────────────────────────

/// Returns the current CLX mode bitmask
#[inline]
pub fn clx_mode() -> u32 {
    CLX_MODE.load(Ordering::Relaxed)
}

/// Returns true if CLX mode is active (FN or locked) and not paused
#[inline]
pub fn is_clx_active() -> bool {
    clx_mode() != CM_NORMAL && !CLX_PAUSED.load(Ordering::Relaxed)
}

/// Returns true if `vk` is one of the configured trigger keys
#[inline]
pub fn is_trigger_vk(vk: u32) -> bool {
    use crate::vk::*;
    (CFG_USE_CAPSLOCK && vk == VK_CAPITAL)
        || (CFG_USE_SPACE && vk == VK_SPACE)
        || (CFG_USE_INSERT && vk == VK_INSERT)
        || (CFG_USE_SCROLL_LOCK && vk == VK_SCROLL)
        || (CFG_USE_RALT && vk == VK_RMENU)
}

/// Enter FN mode (trigger key just pressed)
#[inline]
pub fn enter_fn_mode() {
    CLX_MODE.fetch_or(CM_FN, Ordering::Relaxed);
    CLX_MODE.fetch_and(!CM_CLX, Ordering::Relaxed);
}

/// Exit FN mode (trigger key released)
#[inline]
pub fn exit_fn_mode() {
    CLX_MODE.fetch_and(!CM_FN, Ordering::Relaxed);
}

/// Enter CLX locked mode
#[inline]
pub fn enter_clx_mode() {
    CLX_MODE.fetch_or(CM_CLX, Ordering::Relaxed);
}

/// Exit CLX locked mode
#[inline]
pub fn exit_clx_mode() {
    CLX_MODE.fetch_and(!CM_CLX, Ordering::Relaxed);
}
