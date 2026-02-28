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
use std::sync::{Arc, Condvar, Mutex};

// web-time provides performance.now()-backed Instant on wasm32-unknown-unknown.
// On native targets std::time::Instant is used directly.
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

// ──────────────────────────────── math helpers ───────────────────────────────

fn sign(x: f64) -> f64 {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

/// Acceleration function – matches the AHK polynomial + exponential formula.
fn ma(dt: f64) -> f64 {
    let s = sign(dt);
    let a = dt.abs();
    s * ((a.exp() - 1.0) + 3.0 + 4.0 * a + 9.0 * a * a + 16.0 * a * a * a)
}

/// Velocity damping applied when no / opposing input is present.
fn damping(mut v: f64, accel: f64, dt: f64, max_speed: f64) -> f64 {
    if max_speed.is_finite() { v = v.clamp(-max_speed, max_speed); }
    if accel * v > 0.0 { return v; }
    v *= (-dt * 20.0_f64).exp();
    v -= sign(v) * dt;
    if v.abs() < 1.0 { v = 0.0; }
    v
}

fn add_safe(acc: f64, x: f64) -> f64 {
    let c = acc + x;
    if c.is_finite() { c } else { acc }
}

// ──────────────────────────────── state ──────────────────────────────────────

struct State {
    left_down:  Option<Instant>,
    right_down: Option<Instant>,
    up_down:    Option<Instant>,
    down_down:  Option<Instant>,
    last_tick:  Option<Instant>,
    h_vel:   f64,
    v_vel:   f64,
    h_accum: f64,
    v_accum: f64,
    active:  bool,
    h_accel_ratio:  f64,
    v_accel_ratio:  f64,
    max_speed:      f64,
    mid_key_window: std::time::Duration,
}

impl State {
    fn any_key_held(&self) -> bool {
        self.left_down.is_some() || self.right_down.is_some()
            || self.up_down.is_some() || self.down_down.is_some()
    }
    fn reset(&mut self) {
        self.left_down  = None;
        self.right_down = None;
        self.up_down    = None;
        self.down_down  = None;
        self.last_tick  = None;
        self.h_vel   = 0.0;
        self.v_vel   = 0.0;
        self.h_accum = 0.0;
        self.v_accum = 0.0;
        self.active  = false;
    }
}

// ──────────────────────────────── public API ─────────────────────────────────

pub type ActionFn = dyn Fn(i32, i32, &str) + Send + Sync + 'static;

/// 2-D acceleration model.
///
/// On native targets a background thread drives the physics.
/// On WASM the caller must call `tick_once()` periodically (e.g. every 16 ms
/// via `setInterval`).
pub struct AccModel2D {
    inner:  Arc<(Mutex<State>, Condvar)>,
    action: Arc<ActionFn>,
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
                left_down: None, right_down: None, up_down: None, down_down: None,
                last_tick: None,
                h_vel: 0.0, v_vel: 0.0, h_accum: 0.0, v_accum: 0.0,
                active: false,
                h_accel_ratio,
                v_accel_ratio: if v_accel_ratio == 0.0 { h_accel_ratio } else { v_accel_ratio },
                max_speed,
                mid_key_window: std::time::Duration::from_millis(100),
            }),
            Condvar::new(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let inner_clone = Arc::clone(&inner);
            let action_clone = Arc::clone(&action);
            thread::Builder::new()
                .name("clx-acc-ticker".into())
                .spawn(move || ticker_thread(inner_clone, action_clone))
                .expect("failed to spawn acc ticker thread");
        }

        AccModel2D { inner, action }
    }

    /// Advance the physics by one ~16 ms step.
    ///
    /// On native this is called automatically by the background thread.
    /// On WASM the adapter calls this from a JS `setInterval(fn, 16)`.
    pub fn tick_once(&self) {
        tick_step(&self.inner, &*self.action);
    }

    fn press_dir(inner: &Arc<(Mutex<State>, Condvar)>, field: fn(&mut State) -> &mut Option<Instant>) {
        let (lock, cvar) = inner.as_ref();
        let mut st = lock.lock().unwrap();
        if field(&mut st).is_none() { *field(&mut st) = Some(Instant::now()); }
        if !st.active {
            st.active = true;
            st.last_tick = None;
            cvar.notify_all();
        }
    }

    fn release_dir(inner: &Arc<(Mutex<State>, Condvar)>, field: fn(&mut State) -> &mut Option<Instant>) {
        let (lock, _) = inner.as_ref();
        *field(&mut lock.lock().unwrap()) = None;
    }

    pub fn press_left(&self)    { Self::press_dir(&self.inner,   |s| &mut s.left_down); }
    pub fn release_left(&self)  { Self::release_dir(&self.inner, |s| &mut s.left_down); }
    pub fn press_right(&self)   { Self::press_dir(&self.inner,   |s| &mut s.right_down); }
    pub fn release_right(&self) { Self::release_dir(&self.inner, |s| &mut s.right_down); }
    pub fn press_up(&self)      { Self::press_dir(&self.inner,   |s| &mut s.up_down); }
    pub fn release_up(&self)    { Self::release_dir(&self.inner, |s| &mut s.up_down); }
    pub fn press_down(&self)    { Self::press_dir(&self.inner,   |s| &mut s.down_down); }
    pub fn release_down(&self)  { Self::release_dir(&self.inner, |s| &mut s.down_down); }

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

// ──────────────────────────────── tick logic ─────────────────────────────────

/// One physics step (no sleep).  Returns `true` to keep ticking, `false` if
/// the model has settled and the caller can stop driving it.
fn tick_step(inner: &Arc<(Mutex<State>, Condvar)>, action: &ActionFn) -> bool {
    let now = Instant::now();
    let (lock, _cvar) = inner.as_ref();

    let mut st = lock.lock().unwrap();
    if !st.active { return false; }

    // Fast-start: first tick just fires the "started" callback and sets direction.
    if st.last_tick.is_none() {
        st.last_tick = Some(now);
        let h_sign = sign(
            if st.right_down.is_some() { 1.0 } else { 0.0 }
            - if st.left_down.is_some()  { 1.0 } else { 0.0 },
        );
        let v_sign = sign(
            if st.down_down.is_some() { 1.0 } else { 0.0 }
            - if st.up_down.is_some()  { 1.0 } else { 0.0 },
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
    let left_s  = st.left_down .map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
    let right_s = st.right_down.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
    let up_s    = st.up_down   .map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
    let down_s  = st.down_down .map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);

    // Mid-key: simultaneous opposite directions
    let mid_win = st.mid_key_window;
    if let (Some(lt), Some(rt)) = (st.left_down, st.right_down) {
        let diff = if rt > lt { rt - lt } else { lt - rt };
        if diff < mid_win {
            let s = if rt > lt { 1i32 } else { -1i32 };
            st.reset(); drop(st);
            action(s, 0, "横中键");
            return false;
        }
    }
    if let (Some(ut), Some(dt_inst)) = (st.up_down, st.down_down) {
        let diff = if dt_inst > ut { dt_inst - ut } else { ut - dt_inst };
        if diff < mid_win {
            let s = if dt_inst > ut { 1i32 } else { -1i32 };
            st.reset(); drop(st);
            action(0, s, "纵中键");
            return false;
        }
    }

    // Physics integration
    let h_accel = ma(right_s - left_s) * st.h_accel_ratio;
    let v_accel = ma(down_s  - up_s)   * st.v_accel_ratio;

    st.h_vel = add_safe(st.h_vel, h_accel * dt);
    st.v_vel = add_safe(st.v_vel, v_accel * dt);
    st.h_vel = damping(st.h_vel, h_accel, dt, st.max_speed);
    st.v_vel = damping(st.v_vel, v_accel, dt, st.max_speed);

    st.h_accum = add_safe(st.h_accum, st.h_vel * dt);
    st.v_accum = add_safe(st.v_accum, st.v_vel * dt);

    let h_out = st.h_accum as i32; st.h_accum -= h_out as f64;
    let v_out = st.v_accum as i32; st.v_accum -= v_out as f64;
    let h_vel = st.h_vel;
    let v_vel = st.v_vel;
    let any_key = st.any_key_held();
    drop(st);

    if h_out != 0 || v_out != 0 { action(h_out, v_out, "移动"); }

    if h_vel == 0.0 && v_vel == 0.0 && h_out == 0 && v_out == 0 && !any_key {
        lock.lock().unwrap().active = false;
        action(0, 0, "止动");
        return false;
    }
    true
}

// ──────────────────────────────── native ticker thread ────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn ticker_thread(inner: Arc<(Mutex<State>, Condvar)>, action: Arc<ActionFn>) {
    let (lock, cvar) = inner.as_ref();
    loop {
        // Wait until activated
        {
            let mut st = lock.lock().unwrap();
            while !st.active { st = cvar.wait(st).unwrap(); }
        }
        // Tick loop (sleep first, then step)
        loop {
            thread::sleep(Duration::from_millis(16));
            if !tick_step(&inner, &*action) { break; }
        }
    }
}
