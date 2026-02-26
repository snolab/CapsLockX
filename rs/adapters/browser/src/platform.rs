/// WebPlatform – implements the `Platform` trait for browser / WASM.
///
/// Key output: dispatches synthetic `KeyboardEvent`s on the focused element.
/// Synthetic events have `isTrusted = false`, so our hook callback ignores them.
///
/// Mouse cursor movement: no-op (browsers cannot move the system cursor).
/// Scroll: `window.scrollBy()`.
/// Window management: all no-op.
use wasm_bindgen::JsCast;
use web_sys::{window, Document, KeyboardEvent, KeyboardEventInit};

use capslockx_core::KeyCode;
use capslockx_core::platform::{MouseButton, Platform};
use super::key_map::keycode_to_event_strs;

pub struct WebPlatform;

impl WebPlatform {
    pub fn new() -> Self { Self }

    fn dispatch_key(&self, key: KeyCode, event_type: &str) {
        let win = match window() { Some(w) => w, None => return };
        let doc: Document = match win.document() { Some(d) => d, None => return };

        let (key_str, code_str) = keycode_to_event_strs(key);
        let init = KeyboardEventInit::new();
        init.set_key(key_str);
        init.set_code(code_str);
        init.set_bubbles(true);
        init.set_cancelable(true);

        let event = match KeyboardEvent::new_with_keyboard_event_init_dict(event_type, &init) {
            Ok(e) => e,
            Err(_) => return,
        };

        // Dispatch on the focused element, or the document as fallback.
        let target: web_sys::EventTarget = match doc.active_element() {
            Some(el) => el.unchecked_into(),
            None     => doc.unchecked_into(),
        };
        let _ = target.dispatch_event(&event);
    }
}

impl Platform for WebPlatform {
    fn key_down(&self, key: KeyCode) {
        self.dispatch_key(key, "keydown");
    }

    fn key_up(&self, key: KeyCode) {
        self.dispatch_key(key, "keyup");
    }

    // key_tap / key_tap_n / etc. use the default implementations from Platform.

    fn mouse_move(&self, _dx: i32, _dy: i32) {
        // Cannot move the system cursor from a browser context.
    }

    fn scroll_v(&self, delta: i32) {
        // Platform convention: positive delta = scroll up.
        // window.scrollBy(x, y): positive y = scroll down → negate.
        if let Some(win) = window() {
            let _ = win.scroll_by_with_x_and_y(0.0, (-delta * 3) as f64);
        }
    }

    fn scroll_h(&self, delta: i32) {
        // Positive delta = scroll right.
        if let Some(win) = window() {
            let _ = win.scroll_by_with_x_and_y((delta * 3) as f64, 0.0);
        }
    }

    fn mouse_button(&self, _button: MouseButton, _pressed: bool) {
        // No reliable way to inject mouse button events in browsers.
    }

    // close_tab: default uses key_tap_ctrl(W), which dispatches Ctrl+W.
    fn close_tab(&self) {
        self.key_tap_ctrl(KeyCode::W);
    }

    // All window management methods use the inherited no-op defaults.
}
