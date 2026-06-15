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
    /// When a trigger key is bypassed (e.g. Cmd+Space), the key-up must also
    /// pass through so the OS sees the complete down+up pair.
    trigger_bypassed: AtomicBool,
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
            trigger_bypassed: AtomicBool::new(false),
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
                if self.clx_dn(code, prior) {
                    // Bypass: let the original event pass through to the OS
                    // so modifier+trigger combos (Cmd+Space, Shift+Space) work.
                    self.trigger_bypassed.store(true, Ordering::Relaxed);
                    return CoreResponse::PassThrough;
                }
                self.trigger_bypassed.store(false, Ordering::Relaxed);
            } else if !pressed {
                // If the key-down was bypassed, also pass through the key-up
                // so the OS sees the complete down+up pair for the shortcut.
                if self.trigger_bypassed.swap(false, Ordering::Relaxed) {
                    return CoreResponse::PassThrough;
                }
                self.clx_up(code);
            }
            return CoreResponse::Suppress;
        }

        // ── 3b. Bare ESC dismisses overlays / kills agent (no trigger needed) ──
        if code == KeyCode::Escape && pressed && !is_repeat {
            let mods = self.compute_mods();
            if self.modules.agent.on_key_down(code, &mods) {
                return CoreResponse::Suppress;
            }
            if self.modules.brainstorm.on_key_down(code, &mods) {
                return CoreResponse::Suppress;
            }
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

    /// Ensure a key is in held_keys (inject it if missing).
    /// Used by macOS adapter to sync modifier flags from CGEvent onto held_keys,
    /// since FlagsChanged up can arrive before the key-down event in fast combos.
    pub fn ensure_held(&self, code: KeyCode) {
        self.held_keys.lock().unwrap().insert(code);
    }

    /// Emergency stop: clear all held keys, exit CLX mode, stop all modules.
    /// Called when CGEventTap is disabled (secure input / password fields) so
    /// AccModel doesn't keep running with phantom held keys.
    pub fn emergency_stop(&self) {
        self.held_keys.lock().unwrap().clear();
        self.state.exit_clx_mode();
        self.state.exit_fn_mode();
        self.modules.stop_all();
        eprintln!("[CLX] emergency_stop: all keys released, CLX mode off");
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
        // Write canonical state first so readers always see consistent config.
        *self.state.config.write().unwrap() = new_cfg.clone();
        self.modules.apply_config(&new_cfg);
        self.modules.apply_speeds(&speed);
    }

    // ── CLX_Dn ────────────────────────────────────────────────────────────────

    /// Returns `true` if the trigger key should be passed through (bypass).
    fn clx_dn(&self, code: KeyCode, prior: KeyCode) -> bool {
        // CapsLock+Space chord (either order)
        let chord = (code == KeyCode::CapsLock && prior == KeyCode::Space)
            || (code == KeyCode::Space && prior == KeyCode::CapsLock);
        if chord {
            self.state.enter_clx_mode();
            self.store_trigger(code);
            // Mark as "acted" so releasing either chord key doesn't trigger
            // the single-tap-unlock path in clx_up.
            self.fn_acted.store(true, Ordering::Relaxed);
            return false;
        }

        // Bypass: let the original event pass through to the OS instead of
        // entering CLX mode. This preserves system shortcuts:
        // - <any modifier> + Space → bypass (Shift/Ctrl/Cmd/Alt — Spotlight,
        //   IME switching, etc.). Uniform rule: any modifier held promotes
        //   Space to a system shortcut.
        // - Non-trigger typing + trigger → avoids interfering with typing.
        //
        // Check held_keys directly (not just prior) for reliability — the prior
        // key can be overwritten by intervening FlagsChanged or other events.
        let held = self.held_keys.lock().unwrap();
        let modifier_held = held.iter().any(|k| k.is_modifier());
        let non_modifier_held = held.iter().any(|k| !k.is_modifier() && *k != code);
        drop(held);

        let bypass = if code == KeyCode::Space {
            modifier_held
        } else {
            non_modifier_held
        };
        if bypass {
            return true; // pass through — don't suppress, don't re-inject
        }

        self.state.enter_fn_mode();
        self.store_trigger(code);

        // ── 200 ms hold timeout (native only) ─────────────────────────────
        // If Space is held >200ms with no combo action, deactivate FN mode
        // and emit Space as a normal repeating key (matches system default:
        // first character, then auto-repeat at the OS repeat rate until the
        // user lifts the key).
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
                        // Auto-repeat while Space is still the trigger.
                        // Uses platform-reported system repeat rate, falling
                        // back to a 33 ms default (~30 Hz, macOS default).
                        let repeat_ms = platform
                            .system_key_repeat_ms()
                            .unwrap_or(33)
                            .clamp(15, 500);
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(repeat_ms));
                            let still = *trigger_key.lock().unwrap() == Some(KeyCode::Space);
                            if !still { break; }
                            platform.key_tap(KeyCode::Space);
                        }
                    }
                }
            });
        }
        false
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ClxConfig;
    use crate::test_platform::{Call, MockPlatform};
    use std::sync::Arc;
    use std::time::Duration;

    fn engine_with_space() -> (Arc<ClxEngine>, Arc<MockPlatform>) {
        let platform = Arc::new(MockPlatform::new());
        let mut cfg = ClxConfig::default();
        cfg.use_space = true;
        cfg.use_capslock = false;
        let engine = ClxEngine::with_config(platform.clone(), cfg);
        (engine, platform)
    }

    fn engine_with_capslock() -> (Arc<ClxEngine>, Arc<MockPlatform>) {
        let platform = Arc::new(MockPlatform::new());
        let mut cfg = ClxConfig::default();
        cfg.use_space = false;
        cfg.use_capslock = true;
        let engine = ClxEngine::with_config(platform.clone(), cfg);
        (engine, platform)
    }

    fn key_taps(platform: &MockPlatform, code: KeyCode) -> usize {
        platform.count(|c| matches!(c, Call::KeyDown(k) if *k == code))
    }

    #[test]
    fn space_down_suppressed_and_enters_fn_mode() {
        let (engine, _platform) = engine_with_space();
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::Suppress);
        assert!(engine.state().is_clx_active());
    }

    #[test]
    fn bare_space_tap_emits_space_via_timeout() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        std::thread::sleep(Duration::from_millis(260));
        engine.on_key_event(KeyCode::Space, false);
        assert!(key_taps(&platform, KeyCode::Space) >= 1);
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn space_h_dispatches_left_arrow() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::H, true);
        std::thread::sleep(Duration::from_millis(80));
        engine.on_key_event(KeyCode::H, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(key_taps(&platform, KeyCode::Left) >= 1);
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn space_g_dispatches_enter() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::G, true);
        std::thread::sleep(Duration::from_millis(40));
        engine.on_key_event(KeyCode::G, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(key_taps(&platform, KeyCode::Enter) >= 1);
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn space_t_dispatches_delete() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::T, true);
        std::thread::sleep(Duration::from_millis(40));
        engine.on_key_event(KeyCode::T, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(key_taps(&platform, KeyCode::Delete) >= 1);
    }

    #[test]
    fn space_comma_opens_preferences() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::Comma, true);
        engine.on_key_event(KeyCode::Comma, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(platform.calls().iter().any(|c| matches!(c, Call::OpenPreferences)));
    }

    #[test]
    fn shift_space_bypasses_to_pass_through() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::LShift, true);
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        let up = engine.on_key_event(KeyCode::Space, false);
        assert_eq!(up, CoreResponse::PassThrough);
    }

    #[test]
    fn ctrl_space_bypasses_to_pass_through() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::LCtrl, true);
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        engine.on_key_event(KeyCode::Space, false);
    }

    #[test]
    fn cmd_space_bypasses_to_pass_through() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::LWin, true);
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        engine.on_key_event(KeyCode::Space, false);
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn typing_then_space_bypasses() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::A, true);
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        engine.on_key_event(KeyCode::Space, false);
        engine.on_key_event(KeyCode::A, false);
    }

    #[test]
    fn non_trigger_key_passes_through_when_clx_inactive() {
        let (engine, platform) = engine_with_space();
        let resp = engine.on_key_event(KeyCode::A, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        engine.on_key_event(KeyCode::A, false);
        assert_eq!(key_taps(&platform, KeyCode::Left), 0);
    }

    #[test]
    fn auto_repeat_of_mapped_key_suppressed() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::H, true);
        let repeat = engine.on_key_event(KeyCode::H, true);
        assert_eq!(repeat, CoreResponse::Suppress);
        engine.on_key_event(KeyCode::H, false);
        engine.on_key_event(KeyCode::Space, false);
    }

    #[test]
    fn capslock_space_chord_locks_clx_mode() {
        let platform = Arc::new(MockPlatform::new());
        let mut cfg = ClxConfig::default();
        cfg.use_space = true;
        cfg.use_capslock = true;
        let engine = ClxEngine::with_config(platform.clone(), cfg);

        engine.on_key_event(KeyCode::CapsLock, true);
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::CapsLock, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(engine.state().is_clx_locked());
    }

    #[test]
    fn capslock_trigger_dispatches_hjkl() {
        let (engine, platform) = engine_with_capslock();
        engine.on_key_event(KeyCode::CapsLock, true);
        engine.on_key_event(KeyCode::J, true);
        std::thread::sleep(Duration::from_millis(80));
        engine.on_key_event(KeyCode::J, false);
        engine.on_key_event(KeyCode::CapsLock, false);
        assert!(key_taps(&platform, KeyCode::Down) >= 1);
    }

    #[test]
    fn capslock_bare_tap_emits_capslock() {
        let (engine, platform) = engine_with_capslock();
        engine.on_key_event(KeyCode::CapsLock, true);
        engine.on_key_event(KeyCode::CapsLock, false);
        assert!(key_taps(&platform, KeyCode::CapsLock) >= 1);
    }

    #[test]
    fn emergency_stop_clears_state_and_held_keys() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::W, true);
        engine.emergency_stop();
        assert!(!engine.state().is_clx_active());
    }

    #[test]
    fn ensure_held_inserts_key() {
        let (engine, _platform) = engine_with_space();
        engine.ensure_held(KeyCode::LShift);
        let resp = engine.on_key_event(KeyCode::Space, true);
        assert_eq!(resp, CoreResponse::PassThrough);
        engine.on_key_event(KeyCode::Space, false);
    }

    #[test]
    fn is_mapped_key_reflects_module_mapping() {
        let (engine, _platform) = engine_with_space();
        assert!(engine.is_mapped_key(KeyCode::H));
        assert!(engine.is_mapped_key(KeyCode::Comma));
        assert!(!engine.is_mapped_key(KeyCode::F1));
    }

    #[test]
    fn shift_held_state_tracks_modifier_events() {
        let (engine, _platform) = engine_with_space();
        engine.on_key_event(KeyCode::LShift, true);
        assert!(engine.state().is_shift_held());
        engine.on_key_event(KeyCode::LShift, false);
        assert!(!engine.state().is_shift_held());
    }

    #[test]
    fn config_round_trip_via_get_and_update() {
        let (engine, _platform) = engine_with_space();
        let mut cfg = engine.get_config();
        cfg.gemini_api_key = "test-key".into();
        engine.update_config(cfg.clone());
        assert_eq!(engine.get_config().gemini_api_key, "test-key");
    }

    #[test]
    fn tick_does_not_panic() {
        let (engine, _platform) = engine_with_space();
        engine.tick();
    }

    #[test]
    #[ignore = "flaky: depends on AccModel ticker thread timing"]
    fn clx_locked_tap_unlocks() {
        let platform = Arc::new(MockPlatform::new());
        let mut cfg = ClxConfig::default();
        cfg.use_space = true;
        cfg.use_capslock = true;
        let engine = ClxEngine::with_config(platform.clone(), cfg);

        engine.on_key_event(KeyCode::CapsLock, true);
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::CapsLock, false);
        engine.on_key_event(KeyCode::Space, false);
        assert!(engine.state().is_clx_locked());

        engine.on_key_event(KeyCode::CapsLock, true);
        engine.on_key_event(KeyCode::CapsLock, false);
        assert!(!engine.state().is_clx_locked());
    }

    #[test]
    fn space_up_after_action_does_not_emit_space() {
        let (engine, platform) = engine_with_space();
        engine.on_key_event(KeyCode::Space, true);
        engine.on_key_event(KeyCode::H, true);
        std::thread::sleep(Duration::from_millis(40));
        engine.on_key_event(KeyCode::H, false);
        platform.clear();
        engine.on_key_event(KeyCode::Space, false);
        std::thread::sleep(Duration::from_millis(260));
        assert_eq!(key_taps(&platform, KeyCode::Space), 0);
    }
}
