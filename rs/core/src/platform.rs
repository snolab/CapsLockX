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

/// System audio capture stream (returned by Platform::start_system_audio).
pub trait SystemAudioStream: Send {
    fn take_samples(&self) -> Vec<f32>;
    fn stop(&self);
    fn sample_rate(&self) -> u32;
}

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

    /// Ctrl+Shift+key (e.g. Ctrl+Shift+Tab for prev tab).
    fn key_tap_ctrl_shifted(&self, key: KeyCode) {
        self.key_down(KeyCode::LCtrl);
        self.key_down(KeyCode::LShift);
        self.key_tap(key);
        self.key_up(KeyCode::LShift);
        self.key_up(KeyCode::LCtrl);
    }

    /// Extended key tap (platform-specific; default forwards to `key_tap`).
    /// Windows adapter overrides this to add `KEYEVENTF_EXTENDEDKEY` for
    /// media/volume keys.
    fn key_tap_extended(&self, key: KeyCode) {
        self.key_tap(key);
    }

    /// Tap multiple keys while holding a modifier, all in one atomic SendInput
    /// call to avoid phantom key-up events between separate SendInput calls.
    /// Default implementation falls back to individual calls.
    fn key_tap_n_with_mod(&self, mod_key: KeyCode, key: KeyCode, n: i32) {
        self.key_down(mod_key);
        self.key_tap_n(key, n);
        self.key_up(mod_key);
    }

    /// Query physical key state (e.g. for Shift).  Default uses the tracked
    /// state in ClxState; Windows adapter overrides with GetAsyncKeyState.
    fn is_key_physically_down(&self, _key: KeyCode) -> bool { false }

    /// Tap a key with multiple modifier flags applied atomically.
    /// On macOS, modifier flags must be embedded in the CGEvent itself;
    /// separate key_down(Shift) + key_tap(Arrow) doesn't work.
    /// Default implementation falls back to key_down/key_tap/key_up.
    fn key_tap_with_mods(&self, key: KeyCode, mods: &[KeyCode], n: i32) {
        for m in mods { self.key_down(*m); }
        self.key_tap_n(key, n);
        for m in mods.iter().rev() { self.key_up(*m); }
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

    // ── Text input (optional, default uses key_tap per character) ──────────

    /// Type a string of text at the current cursor position.
    /// Default implementation maps ASCII characters to key taps.
    /// Platform adapters can override with native text input (e.g. CGEvent
    /// unicode strings, SendInput with KEYEVENTF_UNICODE, etc.).
    fn type_text(&self, text: &str) {
        for ch in text.chars() {
            let needs_shift = ch.is_ascii_uppercase() || matches!(ch,
                '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' |
                '_' | '+' | '{' | '}' | '|' | ':' | '"' | '<' | '>' | '?' | '~'
            );
            let key = match ch.to_ascii_lowercase() {
                'a' => Some(KeyCode::A), 'b' => Some(KeyCode::B),
                'c' => Some(KeyCode::C), 'd' => Some(KeyCode::D),
                'e' => Some(KeyCode::E), 'f' => Some(KeyCode::F),
                'g' => Some(KeyCode::G), 'h' => Some(KeyCode::H),
                'i' => Some(KeyCode::I), 'j' => Some(KeyCode::J),
                'k' => Some(KeyCode::K), 'l' => Some(KeyCode::L),
                'm' => Some(KeyCode::M), 'n' => Some(KeyCode::N),
                'o' => Some(KeyCode::O), 'p' => Some(KeyCode::P),
                'q' => Some(KeyCode::Q), 'r' => Some(KeyCode::R),
                's' => Some(KeyCode::S), 't' => Some(KeyCode::T),
                'u' => Some(KeyCode::U), 'v' => Some(KeyCode::V),
                'w' => Some(KeyCode::W), 'x' => Some(KeyCode::X),
                'y' => Some(KeyCode::Y), 'z' => Some(KeyCode::Z),
                '0' | ')' => Some(KeyCode::D0), '1' | '!' => Some(KeyCode::D1),
                '2' | '@' => Some(KeyCode::D2), '3' | '#' => Some(KeyCode::D3),
                '4' | '$' => Some(KeyCode::D4), '5' | '%' => Some(KeyCode::D5),
                '6' | '^' => Some(KeyCode::D6), '7' | '&' => Some(KeyCode::D7),
                '8' | '*' => Some(KeyCode::D8), '9' | '(' => Some(KeyCode::D9),
                ' ' => Some(KeyCode::Space),
                '\n' => Some(KeyCode::Enter),
                '\t' => Some(KeyCode::Tab),
                '.' | '>' => Some(KeyCode::Period),
                '[' | '{' => Some(KeyCode::BracketLeft),
                ']' | '}' => Some(KeyCode::BracketRight),
                '\\' | '|' => Some(KeyCode::Backslash),
                _ => None,
            };
            if let Some(k) = key {
                if needs_shift {
                    self.key_tap_shifted(k);
                } else {
                    self.key_tap(k);
                }
            }
            // Characters without a KeyCode mapping are silently skipped.
            // Platform adapters should override type_text for full Unicode support.
        }
    }

    // ── Voice overlay (optional, default = no-op) ─────────────────────────────

    /// Start capturing system audio. Returns a boxed trait with take_samples()/stop().
    fn start_system_audio(&self) -> Option<Box<dyn SystemAudioStream>> { None }

    /// Start capturing mic with echo cancellation (AEC).
    /// Returns a boxed trait with take_samples()/stop()/sample_rate().
    /// Default returns None (falls back to cpal AudioCapture).
    fn start_aec_mic(&self) -> Option<Box<dyn SystemAudioStream>> { None }

    fn show_voice_overlay(&self) {}
    fn hide_voice_overlay(&self) {}
    fn update_voice_overlay(&self, _mic_levels: &[f32], _mic_vad: bool, _sys_levels: &[f32], _sys_vad: bool) {}
    fn update_voice_subtitle(&self, _text: &str) {}

    // ── Brainstorm overlay (optional, default = no-op) ─────────────────────

    /// Get clipboard text content.
    fn get_clipboard_text(&self) -> String { String::new() }
    /// Set clipboard text content.
    fn set_clipboard_text(&self, _text: &str) {}
    /// Show brainstorm floating overlay with streaming text.
    fn show_brainstorm_overlay(&self, _text: &str) {}
    /// Hide brainstorm overlay.
    fn hide_brainstorm_overlay(&self) {}
    /// Show a prompt input dialog. Returns the user's input, or None if cancelled.
    /// `prefill` is shown in the text field (e.g., clipboard content).
    fn show_prompt_input(&self, _title: &str, _message: &str, _prefill: &str) -> Option<String> { None }

    // ── Lifecycle (optional, default = no-op) ────────────────────────────────

    /// Restart the entire application (spawn new instance, exit current).
    fn restart(&self) {}
}
