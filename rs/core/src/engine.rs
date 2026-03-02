/// ClxEngine – platform-agnostic CLX state machine.
///
/// Instantiate with a `Arc<dyn Platform>`, then call `on_key_event` for every
/// (non-injected) key event from the adapter.  Returns whether to suppress the
/// event or let it pass through to the application.
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::key_code::{KeyCode, Modifiers};
use crate::modules::Modules;
use crate::platform::Platform;
use crate::state::{ClxConfig, ClxState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreResponse {
    Suppress,
    PassThrough,
}

pub struct ClxEngine {
    state:       Arc<ClxState>,
    modules:     Modules,
    platform:    Arc<dyn Platform>,
    held_keys:   Arc<Mutex<HashSet<KeyCode>>>,
    prior_key:   Mutex<KeyCode>,
    trigger_key: Arc<Mutex<Option<KeyCode>>>,
    fn_acted:    Arc<AtomicBool>,
    /// CAS flag: whichever of the timeout thread or clx_up swaps this
    /// from false→true first gets to emit the native key character.
    trigger_timeout_fired: Arc<AtomicBool>,
}

impl ClxEngine {
    pub fn new(platform: Arc<dyn Platform>) -> Arc<Self> {
        Self::with_config(platform, ClxConfig::default())
    }

    pub fn with_config(platform: Arc<dyn Platform>, config: ClxConfig) -> Arc<Self> {
        let state = Arc::new(ClxState::new(config));
        let modules = Modules::new(Arc::clone(&platform), Arc::clone(&state));
        Arc::new(Self {
            state,
            modules,
            platform,
            held_keys:   Arc::new(Mutex::new(HashSet::new())),
            prior_key:   Mutex::new(KeyCode::Unknown(0)),
            trigger_key: Arc::new(Mutex::new(None)),
            fn_acted:    Arc::new(AtomicBool::new(false)),
            trigger_timeout_fired: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Process one key event from the adapter.
    ///
    /// `pressed = true`  → key down (first press only; adapter must filter repeats
    ///                      or pass them through; the engine itself also deduplicates)
    /// `pressed = false` → key up
    pub fn on_key_event(&self, code: KeyCode, pressed: bool) -> CoreResponse {
        // ── 1. Maintain held-key set; detect auto-repeat ──────────────────────
        let is_repeat = {
            let mut held = self.held_keys.lock().unwrap();
            if pressed {
                !held.insert(code)      // false if newly inserted → not a repeat
            } else {
                held.remove(&code);
                false
            }
        };

        // ── 2. Track prior key ────────────────────────────────────────────────
        let prior = *self.prior_key.lock().unwrap();
        if pressed && !is_repeat {
            *self.prior_key.lock().unwrap() = code;
        }

        // ── 3a. Track Shift for AccModel callbacks ────────────────────────────
        if matches!(code, KeyCode::Shift | KeyCode::LShift | KeyCode::RShift) {
            self.state.set_shift_held(pressed);
        }

        // ── 3. Trigger key ────────────────────────────────────────────────────
        if self.state.is_trigger_key(code) {
            if pressed && !is_repeat {
                self.clx_dn(code, prior);
            } else if !pressed {
                self.clx_up(code);
            }
            return CoreResponse::Suppress;
        }

        // ── 4. Non-trigger key while CLX is active ────────────────────────────
        if self.state.is_clx_active() {
            if pressed && !is_repeat {
                self.fn_acted.store(true, Ordering::Relaxed);
                let mods = self.compute_mods();
                if self.modules.on_key_down(code, &mods) {
                    return CoreResponse::Suppress;
                }
            } else if pressed && is_repeat && self.modules.is_mapped_key(code) {
                // Suppress auto-repeat of mapped keys so they don't leak through.
                return CoreResponse::Suppress;
            } else if !pressed && self.modules.on_key_up(code) {
                return CoreResponse::Suppress;
            }
        }

        CoreResponse::PassThrough
    }

    /// Returns true if `code` is a mapped key (used by adapters to decide
    /// whether to eagerly route the event, though calling `on_key_event` for
    /// every key is also correct).
    pub fn is_mapped_key(&self, code: KeyCode) -> bool {
        self.modules.is_mapped_key(code)
    }

    /// Advance all AccModel physics by one step.
    ///
    /// **Native**: the background ticker threads handle this automatically — you
    /// do not need to call `tick()`.
    ///
    /// **WASM**: call this every ~16 ms from a JS `setInterval` to drive the
    /// cursor and scroll acceleration models.
    pub fn tick(&self) {
        self.modules.tick();
    }

    pub fn state(&self) -> &Arc<ClxState> { &self.state }

    pub fn get_config(&self) -> ClxConfig {
        self.state.config.read().unwrap().clone()
    }

    pub fn update_config(&self, new_cfg: ClxConfig) {
        let speed = new_cfg.speed.clone();
        *self.state.config.write().unwrap() = new_cfg;
        self.modules.apply_speeds(&speed);
    }

    // ── CLX_Dn ────────────────────────────────────────────────────────────────

    fn clx_dn(&self, code: KeyCode, prior: KeyCode) {
        // CapsLock+Space chord (either order)
        let chord = (code == KeyCode::CapsLock && prior == KeyCode::Space)
            || (code == KeyCode::Space && prior == KeyCode::CapsLock);
        if chord {
            self.state.enter_clx_mode();
            self.store_trigger(code);
            // Mark as "acted" so releasing either chord key doesn't trigger
            // the single-tap-unlock path in clx_up.
            self.fn_acted.store(true, Ordering::Relaxed);
            return;
        }

        // Bypass: pass the trigger key through instead of entering CLX mode.
        // - Space + Shift held → bypass (preserves Shift+Space for input method switching)
        // - Non-Space trigger + non-modifier key held → bypass (avoids interfering with typing)
        // Note: Ctrl/Alt/Win + Space should NOT bypass — they enter CLX mode so
        // combos like Ctrl+Space+E (Ctrl+Click) work.
        let prior_held = prior != KeyCode::Unknown(0)
            && prior != code
            && self.held_keys.lock().unwrap().contains(&prior);
        let prior_is_shift = matches!(prior, KeyCode::Shift | KeyCode::LShift | KeyCode::RShift);
        let bypass = if code == KeyCode::Space {
            prior_is_shift && prior_held
        } else {
            !prior.is_modifier() && prior_held
        };
        if bypass {
            self.platform.key_tap(code);
            return;
        }

        self.state.enter_fn_mode();
        self.store_trigger(code);

        // ── 200 ms hold timeout (native only) ─────────────────────────────
        // If Space is held >200ms with no combo action, emit a space
        // character and deactivate FN mode — matches original AHK behaviour.
        #[cfg(not(target_arch = "wasm32"))]
        if code == KeyCode::Space {
            let trigger_key = Arc::clone(&self.trigger_key);
            let fn_acted    = Arc::clone(&self.fn_acted);
            let timeout     = Arc::clone(&self.trigger_timeout_fired);
            let platform    = Arc::clone(&self.platform);
            let state       = Arc::clone(&self.state);
            let held_keys = Arc::clone(&self.held_keys);
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let still_held = *trigger_key.lock().unwrap() == Some(KeyCode::Space);
                let acted      = fn_acted.load(Ordering::Relaxed);
                // Don't fire timeout if a modifier is held — user intends
                // Space as a trigger for combos like Ctrl+Space+E.
                let modifier_held = {
                    let hk = held_keys.lock().unwrap();
                    hk.iter().any(|k| k.is_modifier())
                };
                if still_held && !acted && !modifier_held {
                    // CAS: only the first path (timeout vs key-up) to swap
                    // false→true gets to emit the character.
                    if !timeout.swap(true, Ordering::SeqCst) {
                        platform.key_tap(KeyCode::Space);
                        state.exit_fn_mode();
                    }
                }
            });
        }
    }

    // ── CLX_Up ────────────────────────────────────────────────────────────────

    fn clx_up(&self, code: KeyCode) {
        let trigger  = *self.trigger_key.lock().unwrap();
        let fn_acted = self.fn_acted.load(Ordering::Relaxed);

        self.state.exit_fn_mode();

        // Stop physics if CLX mode is now fully off
        if !self.state.is_clx_active() {
            self.modules.stop_all();
        }

        // Single tap with no action → restore native key function.
        // CAS on trigger_timeout_fired: if the 200ms timeout thread already
        // emitted the character and exited FN mode, the swap returns true
        // and we skip the duplicate tap.
        if trigger == Some(code) && !fn_acted {
            if !self.trigger_timeout_fired.swap(true, Ordering::SeqCst) {
                if self.state.is_clx_locked() {
                    // Tap inside locked mode → unlock
                    self.state.exit_clx_mode();
                    self.modules.stop_all();
                } else {
                    match code {
                        KeyCode::CapsLock => self.platform.key_tap(KeyCode::CapsLock),
                        KeyCode::Space    => self.platform.key_tap(KeyCode::Space),
                        _ => {}
                    }
                }
            }
        }

        *self.trigger_key.lock().unwrap() = None;
        self.fn_acted.store(false, Ordering::Relaxed);
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn store_trigger(&self, code: KeyCode) {
        *self.trigger_key.lock().unwrap() = Some(code);
        self.fn_acted.store(false, Ordering::Relaxed);
        self.trigger_timeout_fired.store(false, Ordering::Relaxed);
    }

    fn compute_mods(&self) -> Modifiers {
        Modifiers::from_held(&self.held_keys.lock().unwrap())
    }
}
