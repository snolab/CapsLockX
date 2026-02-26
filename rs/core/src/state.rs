/// Global CapsLockX mode state (atomics → no locking in hot path).
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::key_code::KeyCode;

// ── Mode bitmask constants ─────────────────────────────────────────────────────
pub const CM_NORMAL: u32 = 0;
pub const CM_FN:     u32 = 1;  // trigger key held
pub const CM_CLX:    u32 = 2;  // CapsLock locked mode

// ── Trigger-key configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClxConfig {
    pub use_capslock:    bool,
    pub use_space:       bool,
    pub use_insert:      bool,
    pub use_scroll_lock: bool,
    pub use_ralt:        bool,
}

impl Default for ClxConfig {
    fn default() -> Self {
        Self {
            use_capslock:    true,
            use_space:       true,
            use_insert:      false,
            use_scroll_lock: false,
            use_ralt:        false,
        }
    }
}

// ── State struct ──────────────────────────────────────────────────────────────

pub struct ClxState {
    pub config: ClxConfig,
    mode:   AtomicU32,
    paused: AtomicBool,
}

impl Default for ClxState {
    fn default() -> Self {
        Self::new(ClxConfig::default())
    }
}

impl ClxState {
    pub fn new(config: ClxConfig) -> Self {
        Self {
            config,
            mode:   AtomicU32::new(CM_NORMAL),
            paused: AtomicBool::new(false),
        }
    }

    #[inline] pub fn mode(&self)            -> u32  { self.mode.load(Ordering::Relaxed) }
    #[inline] pub fn is_clx_active(&self)   -> bool { self.mode() != CM_NORMAL && !self.paused.load(Ordering::Relaxed) }
    #[inline] pub fn is_clx_locked(&self)   -> bool { self.mode() & CM_CLX != 0 }

    #[inline]
    pub fn is_trigger_key(&self, key: KeyCode) -> bool {
        (self.config.use_capslock    && key == KeyCode::CapsLock)
        || (self.config.use_space       && key == KeyCode::Space)
        || (self.config.use_insert      && key == KeyCode::Insert)
        || (self.config.use_scroll_lock && key == KeyCode::ScrollLock)
        || (self.config.use_ralt        && key == KeyCode::RAlt)
    }

    pub fn enter_fn_mode(&self) {
        self.mode.fetch_or(CM_FN, Ordering::Relaxed);
        self.mode.fetch_and(!CM_CLX, Ordering::Relaxed);
    }
    pub fn exit_fn_mode(&self)  { self.mode.fetch_and(!CM_FN,  Ordering::Relaxed); }
    pub fn enter_clx_mode(&self){ self.mode.fetch_or(CM_CLX,  Ordering::Relaxed); }
    pub fn exit_clx_mode(&self) { self.mode.fetch_and(!CM_CLX, Ordering::Relaxed); }
}
