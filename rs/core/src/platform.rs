/// The interface that every platform adapter must implement.
///
/// # Adapter responsibilities
///
/// | Platform | Hook mechanism | Suppression | Output |
/// |----------|---------------|-------------|--------|
/// | Windows  | `WH_KEYBOARD_LL` or `RegisterHotKey` | return non-zero from hook proc | `SendInput` |
/// | macOS    | `CGEventTap` | consume event | `CGEventPost` |
/// | Linux    | `evdev` / `XGrabKey` / `uinput` | drop / remap event | `uinput` write |
/// | Browser  | `window.addEventListener('keydown', …)` (WASM) | `event.preventDefault()` | `dispatchEvent(new KeyboardEvent(…))` |
/// | Android  | `InputMethodService` / `AccessibilityService` | consume | `performGlobalAction` / inject |
///
/// Window-management methods have no-op default implementations so adapters
/// that don't support them (e.g. browser) compile without changes.
use crate::key_code::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton { Left, Right, Middle }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangeMode { Stacked, SideBySide }

pub trait Platform: Send + Sync + 'static {
    // ── Keyboard output ───────────────────────────────────────────────────────

    fn key_down(&self, key: KeyCode);
    fn key_up(&self, key: KeyCode);

    /// Press and immediately release a key.
    fn key_tap(&self, key: KeyCode) {
        self.key_down(key);
        self.key_up(key);
    }

    /// Press a key `n` times (clamped to 128).
    fn key_tap_n(&self, key: KeyCode, n: i32) {
        for _ in 0..n.clamp(0, 128) {
            self.key_tap(key);
        }
    }

    /// Shift+key.
    fn key_tap_shifted(&self, key: KeyCode) {
        self.key_down(KeyCode::LShift);
        self.key_tap(key);
        self.key_up(KeyCode::LShift);
    }

    /// Shift+key, repeated `n` times.
    fn key_tap_shifted_n(&self, key: KeyCode, n: i32) {
        for _ in 0..n.clamp(0, 32) {
            self.key_tap_shifted(key);
        }
    }

    /// Ctrl+key (e.g. Ctrl+W for close-tab).
    fn key_tap_ctrl(&self, key: KeyCode) {
        self.key_down(KeyCode::LCtrl);
        self.key_tap(key);
        self.key_up(KeyCode::LCtrl);
    }

    /// Extended key tap (platform-specific; default forwards to `key_tap`).
    /// Windows adapter overrides this to add `KEYEVENTF_EXTENDEDKEY` for
    /// media/volume keys.
    fn key_tap_extended(&self, key: KeyCode) {
        self.key_tap(key);
    }

    // ── Mouse output ──────────────────────────────────────────────────────────

    fn mouse_move(&self, dx: i32, dy: i32);
    fn scroll_v(&self, delta: i32);
    fn scroll_h(&self, delta: i32);
    fn mouse_button(&self, button: MouseButton, pressed: bool);

    // ── Window management (optional, default = no-op) ─────────────────────────

    fn cycle_windows(&self, _dir: i32) {}
    fn arrange_windows(&self, _mode: ArrangeMode) {}
    fn close_tab(&self) {}
    fn close_window(&self) {}
    fn kill_window(&self) {}
    fn set_window_transparent(&self, _alpha: u8) {}
    fn restore_window(&self) {}
    fn toggle_window_topmost(&self) {}

    // ── Virtual desktop (optional, default = no-op) ────────────────────────────

    /// Switch to virtual desktop by 1-based index.
    fn switch_to_desktop(&self, _idx: u32) {}
    /// Move the active window to virtual desktop by 1-based index, then follow.
    fn move_window_to_desktop(&self, _idx: u32) {}

    // ── Lifecycle (optional, default = no-op) ────────────────────────────────

    /// Restart the entire application (spawn new instance, exit current).
    fn restart(&self) {}
}
