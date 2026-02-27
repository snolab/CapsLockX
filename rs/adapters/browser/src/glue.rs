/// wasm-bindgen entry point – installs keyboard hook and physics ticker.
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, KeyboardEvent};
use std::sync::Arc;
use once_cell::sync::OnceCell;

use capslockx_core::{ClxEngine, CoreResponse};
use super::key_map::code_to_keycode;
use super::platform::WebPlatform;

// Engine lives for the lifetime of the WASM module.
static ENGINE: OnceCell<Arc<ClxEngine>> = OnceCell::new();

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        web_sys::console::error_1(&format!("[CLX panic] {}", msg).into());
    }));
}

/// Install the CapsLockX keyboard hook and physics ticker.
///
/// Safe to call multiple times – subsequent calls are no-ops.
///
/// ```js
/// import init, { start } from './pkg/capslockx_browser.js';
/// await init();
/// start();
/// ```
#[wasm_bindgen]
pub fn start() {
    install_panic_hook();
    let engine = ENGINE.get_or_init(|| ClxEngine::new(Arc::new(WebPlatform::new())));

    let win = match window() {
        Some(w) => w,
        None => {
            web_sys::console::error_1(&"[CLX] window not available".into());
            return;
        }
    };

    // ── keydown ───────────────────────────────────────────────────────────────
    {
        let eng = Arc::clone(engine);
        let cb = Closure::<dyn Fn(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            // Skip synthetic events (dispatched by our own Platform::key_down).
            // isTrusted is false for programmatic events – no magic-number needed.
            if !event.is_trusted() { return; }
            if event.repeat()      { return; }

            let code = code_to_keycode(&event.code());
            if eng.on_key_event(code, true) == CoreResponse::Suppress {
                event.prevent_default();
                event.stop_propagation();
            }
        });
        let _ = win.add_event_listener_with_callback_and_bool(
            "keydown",
            cb.as_ref().unchecked_ref(),
            true, // capture phase
        );
        cb.forget();
    }

    // ── keyup ─────────────────────────────────────────────────────────────────
    {
        let eng = Arc::clone(engine);
        let cb = Closure::<dyn Fn(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            if !event.is_trusted() { return; }

            let code = code_to_keycode(&event.code());
            if eng.on_key_event(code, false) == CoreResponse::Suppress {
                event.prevent_default();
                event.stop_propagation();
            }
        });
        let _ = win.add_event_listener_with_callback_and_bool(
            "keyup",
            cb.as_ref().unchecked_ref(),
            true, // capture phase
        );
        cb.forget();
    }

    // ── physics ticker (setInterval 16 ms) ────────────────────────────────────
    {
        let eng = Arc::clone(engine);
        let cb = Closure::<dyn Fn()>::new(move || {
            eng.tick();
        });
        let _ = win.set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            16,
        );
        cb.forget();
    }

    web_sys::console::log_1(&"[CLX] browser adapter active (CapsLock / Space to activate)".into());
}
