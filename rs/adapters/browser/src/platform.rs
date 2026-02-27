/// WebPlatform – implements the `Platform` trait for browser / WASM.
///
/// Key output: dispatches synthetic `KeyboardEvent`s on the focused element.
/// Synthetic events have `isTrusted = false`, so our hook callback ignores them.
///
/// Mouse cursor movement: dispatches a `clx:mouse_move` CustomEvent with
/// `detail = [dx, dy]`.  The page JS renders a virtual cursor overlay and
/// moves it accordingly; when the real mouse moves the overlay is hidden and
/// `vx/vy` snap back to the physical pointer.
///
/// Scroll: dispatches `clx:scroll_v` / `clx:scroll_h` CustomEvents.
/// The page JS finds the scrollable element under the virtual cursor and
/// scrolls it, falling back to `window.scrollBy`.
///
/// Focus cycling (N / P = Tab / Shift+Tab): dispatches `clx:focus` with
/// `detail = 1` (forward) or `-1` (backward).  The page JS cycles through
/// `querySelectorAll` tabbable elements.  Browser-native Tab handling does
/// not work for programmatic key events in Chrome, so we skip the keyboard
/// event entirely and handle focus ourselves.
///
/// Window management: all no-op.
use js_sys::Array;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{window, CustomEvent, CustomEventInit, Document, KeyboardEvent, KeyboardEventInit};

use capslockx_core::KeyCode;
use capslockx_core::platform::{MouseButton, Platform};
use super::key_map::keycode_to_event_strs;

// ── CustomEvent helper ────────────────────────────────────────────────────────

/// Dispatch a CustomEvent on `window` so that any `window.addEventListener`
/// handler in the page JS receives it.
fn dispatch_custom_event(name: &str, detail: JsValue) {
    let win = match window() { Some(w) => w, None => return };
    let init = CustomEventInit::new();
    init.set_detail(&detail);
    if let Ok(ev) = CustomEvent::new_with_event_init_dict(name, &init) {
        let _ = win.dispatch_event(ev.as_ref());
    }
}

// ── WebPlatform ───────────────────────────────────────────────────────────────

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
    // ── Keyboard output ───────────────────────────────────────────────────────

    fn key_down(&self, key: KeyCode) {
        self.dispatch_key(key, "keydown");
    }

    fn key_up(&self, key: KeyCode) {
        self.dispatch_key(key, "keyup");
    }

    /// Override `key_tap` so that:
    /// - `Tab` dispatches `clx:focus(1)` (focus forward in JS).
    /// - Cursor-movement keys dispatch `clx:cursor_move` with the direction
    ///   string so the JS handler can directly manipulate `selectionStart` on
    ///   `<textarea>` / `<input>` elements.  Synthetic `ArrowKey` events are
    ///   also dispatched for any other listeners (e.g. CodeMirror), but Chrome
    ///   will NOT move the native text cursor for untrusted events.
    fn key_tap(&self, key: KeyCode) {
        if key == KeyCode::Tab {
            dispatch_custom_event("clx:focus", JsValue::from(1_i32));
            return;
        }
        // For cursor keys, send clx:cursor_move so JS can move selectionStart.
        let dir = match key {
            KeyCode::Left     => Some("ArrowLeft"),
            KeyCode::Right    => Some("ArrowRight"),
            KeyCode::Up       => Some("ArrowUp"),
            KeyCode::Down     => Some("ArrowDown"),
            KeyCode::Home     => Some("Home"),
            KeyCode::End      => Some("End"),
            KeyCode::PageUp   => Some("PageUp"),
            KeyCode::PageDown => Some("PageDown"),
            _                 => None,
        };
        if let Some(d) = dir {
            dispatch_custom_event("clx:cursor_move", JsValue::from_str(d));
        }
        self.dispatch_key(key, "keydown");
        self.dispatch_key(key, "keyup");
    }

    /// Override `key_tap_shifted` so that `Shift+Tab` dispatches `clx:focus`
    /// with direction `-1` (backward) instead of a synthetic key sequence.
    fn key_tap_shifted(&self, key: KeyCode) {
        if key == KeyCode::Tab {
            dispatch_custom_event("clx:focus", JsValue::from(-1_i32));
            return;
        }
        self.dispatch_key(KeyCode::LShift, "keydown");
        self.dispatch_key(key, "keydown");
        self.dispatch_key(key, "keyup");
        self.dispatch_key(KeyCode::LShift, "keyup");
    }

    // ── Mouse output ──────────────────────────────────────────────────────────

    /// Move the virtual cursor by `(dx, dy)` pixels.
    /// Dispatches `clx:mouse_move` with `detail = [dx, dy]`.
    fn mouse_move(&self, dx: i32, dy: i32) {
        let detail = Array::of2(&JsValue::from(dx), &JsValue::from(dy));
        dispatch_custom_event("clx:mouse_move", detail.into());
    }

    /// Scroll vertically.  Platform convention: positive `delta` = scroll up.
    /// Dispatches `clx:scroll_v` with `detail` = the pixel amount (negative = down).
    fn scroll_v(&self, delta: i32) {
        dispatch_custom_event("clx:scroll_v", JsValue::from(-delta * 3));
    }

    /// Scroll horizontally.  Positive `delta` = scroll right.
    fn scroll_h(&self, delta: i32) {
        dispatch_custom_event("clx:scroll_h", JsValue::from(delta * 3));
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
