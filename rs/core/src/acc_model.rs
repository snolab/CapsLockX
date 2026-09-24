use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
/// AccModel2D – time-based 2-D acceleration physics model.
///
/// Ported from AHK `AccModel2D` (Modules/AccModel/AccModel.ahk).
/// Uses `std::time::Instant` for cross-platform timing.
///
/// On native platforms the model runs a background thread that ticks every 16 ms.
/// On WASM (single-threaded) the thread is omitted; callers must drive ticks via
/// `AccModel2D::tick_once()` from a JS `setInterval` (see `ClxEngine::tick()`).
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// When true, AccModel2D does NOT spawn background ticker threads.
/// The caller must drive ticks externally (e.g. via SetTimer on Windows).
/// This ensures SendInput runs on the main/hook thread, avoiding phantom
/// modifier key-up events that the OS inserts for cross-thread injections.
static EXTERNAL_TICK: AtomicBool = AtomicBool::new(false);

/// Call before creating any AccModel2D instances to disable background threads.
pub fn set_external_tick(enabled: bool) {
    EXTERNAL_TICK.store(enabled, Ordering::SeqCst);
}

// web-time provides performance.now()-backed Instant on wasm32-unknown-unknown.
// On native targets std::time::Instant is used directly.
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

// ──────────────────────────────── math helpers ───────────────────────────────

fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Acceleration function – matches the AHK polynomial + exponential formula.
/// Raw output is divided by K_RAW so that `speed_ratio` = units/second.
///
/// The double integral ∫₀¹∫₀ᵗ ma_raw(s) ds dt ≈ 3.935, meaning holding a key
/// for 1 second with speed_ratio=1 produces ~1 unit of displacement.
/// See acc_model_test.rs for FPS-independence verification.
const K_RAW: f64 = 3.935;

fn ma(dt: f64) -> f64 {
    let s = sign(dt);
    let a = dt.abs();
    s * ((a.exp() - 1.0) + 3.0 + 4.0 * a + 9.0 * a * a + 16.0 * a * a * a) / K_RAW
}

/// Velocity damping applied when no / opposing input is present.
fn damping(mut v: f64, accel: f64, dt: f64, max_speed: f64) -> f64 {
    if max_speed.is_finite() {
        v = v.clamp(-max_speed, max_speed);
    }
    if accel * v > 0.0 {
        return v;
    }
    v *= (-dt * 20.0_f64).exp();
    v -= sign(v) * dt;
    if v.abs() < 1.0 {
        v = 0.0;
    }
    v
}

fn add_safe(acc: f64, x: f64) -> f64 {
    let c = acc + x;
    if c.is_finite() {
        c
    } else {
        acc
    }
}

// ──────────────────────────────── state ──────────────────────────────────────

struct State {
    left_down: Option<Instant>,
    right_down: Option<Instant>,
    up_down: Option<Instant>,
    down_down: Option<Instant>,
    last_tick: Option<Instant>,
    h_vel: f64,
    v_vel: f64,
    h_accum: f64,
    v_accum: f64,
    active: bool,
    max_active: std::time::Duration,
    // Separate from acceleration timestamps: a repeat must not restart physics.
    evidence: [Option<Instant>; 4],
    h_accel_ratio: f64,
    v_accel_ratio: f64,
    max_speed: f64,
    mid_key_window: std::time::Duration,
    /// Step units this activation may still emit, or `None` for unlimited.
    /// See [`AccModel2D::set_max_steps`].
    steps_left: Option<u32>,
    max_steps: Option<u32>,
}

impl State {
    fn any_key_held(&self) -> bool {
        self.left_down.is_some()
            || self.right_down.is_some()
            || self.up_down.is_some()
            || self.down_down.is_some()
    }
    fn reset(&mut self) {
        self.left_down = None;
        self.right_down = None;
        self.up_down = None;
        self.down_down = None;
        self.last_tick = None;
        self.h_vel = 0.0;
        self.v_vel = 0.0;
        self.h_accum = 0.0;
        self.v_accum = 0.0;
        self.active = false;
        self.evidence = [None; 4];
        // A fresh press gets a fresh budget: the cap bounds one gesture, and the
        // user pressing the key again is always allowed to continue.
        self.steps_left = self.max_steps;
    }
}

// ──────────────────────────────── public API ─────────────────────────────────

pub type ActionFn = dyn Fn(i32, i32, &str) + Send + Sync + 'static;

/// Asked once per tick while the model is running: "is this motion still
/// legitimate?" Returning `false` stops the model immediately.
///
/// This exists because every exit from the physics loop used to require the
/// key-UP event to reach the engine. When the foreground window belongs to a
/// higher-integrity process (an elevated app, or the DWM-owned "Ghost" window
/// Windows puts up in front of an unresponsive app), a non-elevated
/// `WH_KEYBOARD_LL` hook stops receiving input entirely — so the key-up never
/// arrives, the direction stays latched, and the model accelerates to
/// `max_speed` forever. A single tap then behaves exactly like holding the key
/// down for good. The same bug in the Space auto-repeat path is recorded in
/// `tmp/2026-09-11-clx-wedge-incident.md`; this is the general cure for the
/// whole family.
pub type WatchdogFn = dyn Fn() -> bool + Send + Sync + 'static;

/// Default maximum time without down evidence for any held direction.
/// Non-repeating holds also reach this ceiling. Raw releases remain primary.
const MAX_ACTIVE_MS: u128 = 20_000;

/// 2-D acceleration model.
///
/// On native targets a background thread drives the physics.
/// On WASM the caller must call `tick_once()` periodically (e.g. every 16 ms
/// via `setInterval`).
pub struct AccModel2D {
    inner: Arc<(Mutex<State>, Condvar)>,
    action: Arc<ActionFn>,
    watchdog: Arc<Mutex<Option<Arc<WatchdogFn>>>>,
}

impl AccModel2D {
    pub fn new(
        action: Arc<ActionFn>,
        h_accel_ratio: f64,
        v_accel_ratio: f64,
        max_speed: f64,
    ) -> Self {
        let inner = Arc::new((
            Mutex::new(State {
                left_down: None,
                right_down: None,
                up_down: None,
                down_down: None,
                last_tick: None,
                h_vel: 0.0,
                v_vel: 0.0,
                h_accum: 0.0,
                v_accum: 0.0,
                active: false,
                max_active: std::time::Duration::from_millis(MAX_ACTIVE_MS as u64),
                evidence: [None; 4],
                h_accel_ratio,
                v_accel_ratio: if v_accel_ratio == 0.0 {
                    h_accel_ratio
                } else {
                    v_accel_ratio
                },
                max_speed,
                steps_left: None,
                max_steps: None,
                mid_key_window: std::time::Duration::from_millis(100),
            }),
            Condvar::new(),
        ));

        let watchdog: Arc<Mutex<Option<Arc<WatchdogFn>>>> = Arc::new(Mutex::new(None));

        #[cfg(not(target_arch = "wasm32"))]
        if !EXTERNAL_TICK.load(Ordering::SeqCst) {
            let inner_clone = Arc::clone(&inner);
            let action_clone = Arc::clone(&action);
            let wd_clone = Arc::clone(&watchdog);
            thread::Builder::new()
                .name("clx-acc-ticker".into())
                .spawn(move || ticker_thread(inner_clone, action_clone, wd_clone))
                .expect("failed to spawn acc ticker thread");
        }

        AccModel2D {
            inner,
            action,
            watchdog,
        }
    }

    /// Install the runaway watchdog. Called once per model, by the module that
    /// owns it, with a closure that knows which physical keys drive it.
    pub fn set_watchdog(&self, f: Arc<WatchdogFn>) {
        *self.watchdog.lock().unwrap() = Some(f);
    }

    /// Bound time without per-key down evidence. Non-repeating deliberate holds
    /// also reach this ceiling; a new press starts again.
    pub fn set_max_active(&self, duration: std::time::Duration) {
        self.inner.0.lock().unwrap().max_active = duration;
    }

    /// Cap how many step units one activation may emit.
    ///
    /// The time-based guards above bound how *long* a runaway lasts, which is
    /// the right currency for continuous motion — an extra second of cursor
    /// drift is a nuisance. It is the wrong currency for a discrete action with
    /// a heavy side effect: two seconds of window cycling at full speed is
    /// hundreds of window switches, and the user experiences that as an endless
    /// flood however promptly it "stops".
    ///
    /// A count bounds the damage regardless of *why* a release went missing,
    /// which is what makes it a guarantee rather than another special case —
    /// every previous attempt at this bug enumerated the windows that swallow
    /// input (elevated, Ghost, hung, RDP, and now Magnifier) and the list was
    /// never finished.
    ///
    /// The budget is **spent by steps and refilled by evidence that the key is
    /// still down**: every typematic repeat that reaches
    /// [`refresh_direction`](Self::refresh_direction), and every fresh press,
    /// tops it back up. So a genuine hold, which produces a repeat every few
    /// tens of milliseconds, cycles as far as the user likes and is effectively
    /// not capped; a lost release produces no repeats, runs the budget down, and
    /// goes quiet. The cap only ever bites when clx has stopped hearing the
    /// keyboard, which is exactly the condition it is there for.
    ///
    /// Running out *pauses* the model rather than stopping it, so the next repeat
    /// resumes a genuine hold seamlessly. Stopping would have required a release
    /// and a new press — and a lost key-up means no release is coming.
    pub fn set_max_steps(&self, steps: u32) {
        let mut st = self.inner.0.lock().unwrap();
        st.max_steps = Some(steps);
        st.steps_left = Some(steps);
    }

    /// Extend only an already-held direction (left, right, up, down = 0..4).
    /// A late repeat cannot restart a model that already reached its ceiling.
    pub fn refresh_direction(&self, direction: usize) {
        let mut st = self.inner.0.lock().unwrap();
        if let Some(evidence) = st.evidence.get_mut(direction) {
            if evidence.is_some() {
                *evidence = Some(Instant::now());
                // A repeat is proof the key is still physically down, which is
                // what refills the step budget. See `set_max_steps`.
                st.steps_left = st.max_steps;
            }
        }
    }

    /// Advance the physics by one ~16 ms step.
    ///
    /// On native this is called automatically by the background thread.
    /// On WASM the adapter calls this from a JS `setInterval(fn, 16)`.
    pub fn tick_once(&self) {
        let wd = self.watchdog.lock().unwrap().clone();
        tick_step(&self.inner, &*self.action, wd.as_deref());
    }

    fn press_dir(
        inner: &Arc<(Mutex<State>, Condvar)>,
        field: fn(&mut State) -> &mut Option<Instant>,
        direction: usize,
    ) {
        let (lock, cvar) = inner.as_ref();
        let mut st = lock.lock().unwrap();
        if field(&mut st).is_none() {
            *field(&mut st) = Some(Instant::now());
            st.evidence[direction] = Some(Instant::now());
        }
        // A press is the strongest evidence the key is down, so it refills the
        // step budget exactly as a repeat does — otherwise a gesture that had
        // paused at the cap would stay paused through a deliberate new press.
        st.steps_left = st.max_steps;
        if !st.active {
            st.active = true;
            st.last_tick = None;
            cvar.notify_all();
        }
    }

    fn release_dir(
        inner: &Arc<(Mutex<State>, Condvar)>,
        field: fn(&mut State) -> &mut Option<Instant>,
        direction: usize,
    ) {
        let (lock, _) = inner.as_ref();
        let mut st = lock.lock().unwrap();
        *field(&mut st) = None;
        st.evidence[direction] = None;
    }

    pub fn press_left(&self) {
        Self::press_dir(&self.inner, |s| &mut s.left_down, 0);
    }
    pub fn release_left(&self) {
        Self::release_dir(&self.inner, |s| &mut s.left_down, 0);
    }
    pub fn press_right(&self) {
        Self::press_dir(&self.inner, |s| &mut s.right_down, 1);
    }
    pub fn release_right(&self) {
        Self::release_dir(&self.inner, |s| &mut s.right_down, 1);
    }
    pub fn press_up(&self) {
        Self::press_dir(&self.inner, |s| &mut s.up_down, 2);
    }
    pub fn release_up(&self) {
        Self::release_dir(&self.inner, |s| &mut s.up_down, 2);
    }
    pub fn press_down(&self) {
        Self::press_dir(&self.inner, |s| &mut s.down_down, 3);
    }
    pub fn release_down(&self) {
        Self::release_dir(&self.inner, |s| &mut s.down_down, 3);
    }

    pub fn set_ratios(&self, h: f64, v: f64, max: f64) {
        let (lock, _) = self.inner.as_ref();
        let mut st = lock.lock().unwrap();
        st.h_accel_ratio = h;
        st.v_accel_ratio = if v == 0.0 { h } else { v };
        st.max_speed = max;
    }

    pub fn stop(&self) {
        let (lock, _) = self.inner.as_ref();
        lock.lock().unwrap().reset();
    }
}

unsafe impl Sync for AccModel2D {}
unsafe impl Send for AccModel2D {}

#[cfg(test)]
mod step_cap_tests {
    use super::*;
    use std::sync::atomic::AtomicI32;

    /// A model with a step cap, and a counter of the step units it emitted.
    fn capped(steps: u32) -> (AccModel2D, Arc<AtomicI32>) {
        set_external_tick(true);
        let emitted = Arc::new(AtomicI32::new(0));
        let sink = Arc::clone(&emitted);
        let model = AccModel2D::new(
            Arc::new(move |dx, _dy, phase| {
                if phase == "MOVE" {
                    sink.fetch_add(dx.abs(), Ordering::Relaxed);
                }
            }),
            30.0,
            30.0,
            250.0,
        );
        model.set_max_steps(steps);
        (model, emitted)
    }

    /// Drive the model hard enough that an uncapped one would run away. The
    /// sleep is not decoration: the physics integrates over real elapsed time,
    /// so a tight tick loop advances by dt≈0 and emits nothing at all.
    fn spin(model: &AccModel2D, ticks: usize) {
        for _ in 0..ticks {
            model.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn a_lost_key_up_cannot_emit_more_than_the_cap() {
        let (model, emitted) = capped(8);
        model.press_right();
        // No release and no repeat ever arrives — the Magnifier/Task Manager
        // case, where the window we cycled into swallows the keyboard.
        spin(&model, 120);
        let after_cap = emitted.load(Ordering::Relaxed);
        assert!(after_cap <= 8, "emitted {after_cap} steps, cap was 8");
        // And it stays quiet: the flood is bounded, not merely slowed. (The
        // model is still *live* — see `set_max_steps` — the time ceiling and the
        // watchdog are what finally retire it.)
        spin(&model, 120);
        assert_eq!(
            emitted.load(Ordering::Relaxed),
            after_cap,
            "an exhausted budget must not leak further steps"
        );
    }

    #[test]
    fn a_genuine_hold_refills_the_budget_and_is_not_capped() {
        let (model, emitted) = capped(8);
        model.press_right();
        // A held key produces typematic repeats; each one is proof it is still
        // down, so the gesture must keep going well past the cap. Long enough
        // (~1.2 s) that the acceleration curve has produced more than 8 steps —
        // over a short window an uncapped model would not reach 8 either, and
        // the test would prove nothing.
        for _ in 0..24 {
            spin(&model, 10);
            model.refresh_direction(1);
        }
        assert!(
            emitted.load(Ordering::Relaxed) > 8,
            "a held key was capped at {} steps",
            emitted.load(Ordering::Relaxed)
        );
        assert!(
            model.inner.0.lock().unwrap().active,
            "must still be running"
        );
    }

    #[test]
    fn pressing_again_after_the_cap_starts_a_fresh_budget() {
        let (model, emitted) = capped(8);
        model.press_right();
        spin(&model, 120);
        let first = emitted.load(Ordering::Relaxed);
        model.press_right();
        spin(&model, 120);
        assert!(
            emitted.load(Ordering::Relaxed) > first,
            "a new press must be allowed to move again"
        );
    }
}

#[cfg(test)]
mod ceiling_tests {
    use super::*;

    #[test]
    fn per_model_ceiling_stops_stuck_input_and_allows_a_new_press() {
        set_external_tick(true);
        let short = AccModel2D::new(Arc::new(|_, _, _| {}), 30.0, 30.0, 250.0);
        let default = AccModel2D::new(Arc::new(|_, _, _| {}), 30.0, 30.0, 250.0);
        short.set_max_active(std::time::Duration::from_secs(2));
        for model in [&short, &default] {
            model.press_right();
            model.inner.0.lock().unwrap().evidence[1] =
                Some(Instant::now() - std::time::Duration::from_secs(3));
            model.tick_once();
        }
        assert!(!short.inner.0.lock().unwrap().active);
        assert!(default.inner.0.lock().unwrap().active);
        short.press_right();
        short.tick_once();
        assert!(short.inner.0.lock().unwrap().active);
        default.inner.0.lock().unwrap().evidence[1] =
            Some(Instant::now() - std::time::Duration::from_secs(21));
        default.tick_once();
        assert!(!default.inner.0.lock().unwrap().active);
        short.stop();
    }

    #[test]
    fn repeat_evidence_extends_hold_without_restarting_acceleration() {
        set_external_tick(true);
        let model = AccModel2D::new(Arc::new(|_, _, _| {}), 30.0, 30.0, 250.0);
        model.set_max_active(std::time::Duration::from_secs(2));
        model.press_right();
        let original = model.inner.0.lock().unwrap().right_down;
        model.inner.0.lock().unwrap().evidence[1] =
            Some(Instant::now() - std::time::Duration::from_secs(3));
        model.refresh_direction(1);
        model.tick_once();
        let st = model.inner.0.lock().unwrap();
        assert!(st.active);
        assert_eq!(st.right_down, original);
        drop(st);
        model.stop();
        model.refresh_direction(1);
        assert!(!model.inner.0.lock().unwrap().active);
    }

    #[test]
    fn another_direction_repeat_cannot_hide_a_stale_latch() {
        set_external_tick(true);
        let model = AccModel2D::new(Arc::new(|_, _, _| {}), 30.0, 30.0, 250.0);
        model.set_max_active(std::time::Duration::from_secs(2));
        model.press_right();
        model.press_down();
        model.inner.0.lock().unwrap().evidence[1] =
            Some(Instant::now() - std::time::Duration::from_secs(3));
        model.refresh_direction(3);
        model.tick_once();
        assert!(!model.inner.0.lock().unwrap().active);
    }
}

// ──────────────────────────────── tick logic ─────────────────────────────────

/// One physics step (no sleep).  Returns `true` to keep ticking, `false` if
/// the model has settled and the caller can stop driving it.
fn tick_step(
    inner: &Arc<(Mutex<State>, Condvar)>,
    action: &ActionFn,
    watchdog: Option<&WatchdogFn>,
) -> bool {
    let now = Instant::now();
    let (lock, _cvar) = inner.as_ref();

    let mut st = lock.lock().unwrap();
    if !st.active {
        return false;
    }

    // ── Runaway guards ───────────────────────────────────────────────────────
    // Both run before the physics, so a model whose key-up was never delivered
    // stops here instead of integrating its way up to `max_speed` forever.
    let max_active = st.max_active;
    let over_ceiling = st
        .evidence
        .iter()
        .flatten()
        .any(|t| now.duration_since(*t) >= max_active);
    if over_ceiling {
        st.reset();
        drop(st);
        eprintln!(
            "[CLX] acc model had no key-down evidence for {} ms — release and press again",
            max_active.as_millis()
        );
        action(0, 0, "STOP");
        return false;
    }
    if let Some(wd) = watchdog {
        // Drop the lock across the callback: it calls into the platform
        // (GetAsyncKeyState), and holding the state mutex through an FFI call
        // is how deadlocks get written.
        drop(st);
        let alive = wd();
        if !alive {
            let (lock2, _) = inner.as_ref();
            lock2.lock().unwrap().reset();
            action(0, 0, "STOP");
            return false;
        }
        st = lock.lock().unwrap();
        if !st.active {
            return false;
        }
    }

    // Fast-start: first tick just fires the "started" callback and sets direction.
    if st.last_tick.is_none() {
        st.last_tick = Some(now);
        let h_sign = sign(
            if st.right_down.is_some() { 1.0 } else { 0.0 }
                - if st.left_down.is_some() { 1.0 } else { 0.0 },
        );
        let v_sign = sign(
            if st.down_down.is_some() { 1.0 } else { 0.0 }
                - if st.up_down.is_some() { 1.0 } else { 0.0 },
        );
        st.h_accum = h_sign;
        st.v_accum = v_sign;
        drop(st);
        action(0, 0, "启动");
        return true;
    }

    let dt = {
        let last = st.last_tick.unwrap();
        let d = now.duration_since(last).as_secs_f64();
        st.last_tick = Some(now);
        d
    };

    // Hold durations
    let left_s = st
        .left_down
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);
    let right_s = st
        .right_down
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);
    let up_s = st.up_down.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
    let down_s = st
        .down_down
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0);

    // Mid-key: simultaneous opposite directions
    let mid_win = st.mid_key_window;
    if let (Some(lt), Some(rt)) = (st.left_down, st.right_down) {
        let diff = if rt > lt { rt - lt } else { lt - rt };
        if diff < mid_win {
            let s = if rt > lt { 1i32 } else { -1i32 };
            st.reset();
            drop(st);
            action(s, 0, "H_MIDKEY");
            return false;
        }
    }
    if let (Some(ut), Some(dt_inst)) = (st.up_down, st.down_down) {
        let diff = if dt_inst > ut {
            dt_inst - ut
        } else {
            ut - dt_inst
        };
        if diff < mid_win {
            let s = if dt_inst > ut { 1i32 } else { -1i32 };
            st.reset();
            drop(st);
            action(0, s, "V_MIDKEY");
            return false;
        }
    }

    // Physics integration
    let h_accel = ma(right_s - left_s) * st.h_accel_ratio;
    let v_accel = ma(down_s - up_s) * st.v_accel_ratio;

    st.h_vel = add_safe(st.h_vel, h_accel * dt);
    st.v_vel = add_safe(st.v_vel, v_accel * dt);
    st.h_vel = damping(st.h_vel, h_accel, dt, st.max_speed);
    st.v_vel = damping(st.v_vel, v_accel, dt, st.max_speed);

    st.h_accum = add_safe(st.h_accum, st.h_vel * dt);
    st.v_accum = add_safe(st.v_accum, st.v_vel * dt);

    let mut h_out = st.h_accum as i32;
    st.h_accum -= h_out as f64;
    let mut v_out = st.v_accum as i32;
    st.v_accum -= v_out as f64;

    // Spend the step budget. An exhausted budget *pauses* the model rather than
    // stopping it: the motion is withheld, but the model stays live so that the
    // next typematic repeat can refill the budget and carry a genuine hold
    // straight on. That is what lets the cap be small enough to matter without
    // making a real hold stutter to a halt — a stop would need the user to
    // release and press again, and a lost key-up means no release is coming.
    //
    // Clamping the tick rather than discarding it keeps a single fast tick from
    // overshooting the cap, and the accumulators are dropped while paused so no
    // burst of withheld motion builds up to be released all at once.
    if let Some(left) = st.steps_left {
        let want = h_out.unsigned_abs() + v_out.unsigned_abs();
        if want > left {
            // Give what remains to the dominant axis; mixed-axis overrun at the
            // very last step is not worth splitting hairs over.
            let allowed = left as i32;
            if h_out.abs() >= v_out.abs() {
                h_out = h_out.signum() * allowed.min(h_out.abs());
                v_out = 0;
            } else {
                v_out = v_out.signum() * allowed.min(v_out.abs());
                h_out = 0;
            }
            st.steps_left = Some(0);
            st.h_accum = 0.0;
            st.v_accum = 0.0;
        } else {
            st.steps_left = Some(left - want);
        }
    }

    let h_vel = st.h_vel;
    let v_vel = st.v_vel;
    let any_key = st.any_key_held();
    drop(st);

    if h_out != 0 || v_out != 0 {
        action(h_out, v_out, "MOVE");
    }

    if h_vel == 0.0 && v_vel == 0.0 && h_out == 0 && v_out == 0 && !any_key {
        lock.lock().unwrap().active = false;
        action(0, 0, "STOP");
        return false;
    }
    true
}

// ──────────────────────────────── native ticker thread ────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn ticker_thread(
    inner: Arc<(Mutex<State>, Condvar)>,
    action: Arc<ActionFn>,
    watchdog: Arc<Mutex<Option<Arc<WatchdogFn>>>>,
) {
    use std::sync::atomic::AtomicU64;

    // FPS logger: every 2 seconds, log actual tick rate to stderr.
    // Helps diagnose lag — if actual FPS << target, the thread is being
    // starved by scheduling or memory pressure.
    static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
    static LAST_FPS_LOG: AtomicU64 = AtomicU64::new(0);

    let (lock, cvar) = inner.as_ref();
    loop {
        // Wait until activated
        {
            let mut st = lock.lock().unwrap();
            while !st.active {
                st = cvar.wait(st).unwrap();
            }
        }
        // Reset FPS counter on activation
        TICK_COUNT.store(0, Ordering::Relaxed);
        LAST_FPS_LOG.store(0, Ordering::Relaxed);

        // Tick loop — target 6ms (~166 FPS), comfortably above 144Hz screens.
        // The old 16ms sleep gave ~62 FPS, visibly choppy on high-refresh displays.
        // Uses sleep + spin for sub-ms precision without burning a full core.
        let mut next_tick = Instant::now();
        loop {
            next_tick += Duration::from_millis(6);
            let now = Instant::now();
            if next_tick > now {
                let remaining = next_tick - now;
                if remaining > Duration::from_millis(2) {
                    thread::sleep(remaining - Duration::from_millis(1));
                }
                while Instant::now() < next_tick {
                    std::hint::spin_loop();
                }
            }

            // FPS logging (every 2s while active)
            let n = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let last = LAST_FPS_LOG.load(Ordering::Relaxed);
            if last == 0 {
                LAST_FPS_LOG.store(now_secs, Ordering::Relaxed);
                TICK_COUNT.store(0, Ordering::Relaxed);
            } else if now_secs.saturating_sub(last) >= 2 {
                let elapsed = now_secs - last;
                let fps = n / elapsed;
                eprintln!("[CLX] acc-ticker: {} FPS (target 166)", fps);
                LAST_FPS_LOG.store(now_secs, Ordering::Relaxed);
                TICK_COUNT.store(0, Ordering::Relaxed);
            }

            let wd = watchdog.lock().unwrap().clone();
            if !tick_step(&inner, &*action, wd.as_deref()) {
                break;
            }
        }
    }
}
