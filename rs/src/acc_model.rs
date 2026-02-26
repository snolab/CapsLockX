/// AccModel2D – time-based 2-D acceleration physics model
///
/// Ported from AHK `AccModel2D` class (Modules/AccModel/AccModel.ahk).
///
/// Each instance owns a background thread that ticks at ~16 ms intervals
/// and calls the user-supplied `action` callback with (dx, dy, phase) where
/// phase is one of: "启动" (start), "移动" (move), "止动" (stop),
/// "横中键" (h-mid), "纵中键" (v-mid).
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use windows::Win32::System::Performance::{
    QueryPerformanceCounter, QueryPerformanceFrequency,
};

// ──────────────────────────────── helpers ────────────────────────────────────

fn qpc() -> i64 {
    let mut c = 0i64;
    unsafe { let _ = QueryPerformanceCounter(&mut c); }
    c
}

fn qpf() -> i64 {
    let mut f = 0i64;
    unsafe { let _ = QueryPerformanceFrequency(&mut f); }
    f
}

fn sign(x: f64) -> f64 {
    if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 }
}

/// Acceleration function – returns force given how long the key has been held
/// (dt in seconds). Matches the AHK polynomial + exponential formula.
fn ma(dt: f64) -> f64 {
    let s = sign(dt);
    let a = dt.abs();
    s * ((a.exp() - 1.0) + 3.0 + 4.0 * a + 9.0 * a * a + 16.0 * a * a * a)
}

/// Velocity damping – applied when no input (or opposing input) is present.
fn damping(mut v: f64, accel: f64, dt: f64, max_speed: f64) -> f64 {
    // Hard clamp to max speed
    if max_speed.is_finite() {
        v = v.clamp(-max_speed, max_speed);
    }
    // If acceleration and velocity are in the same direction, don't damp
    if accel * v > 0.0 {
        return v;
    }
    // Exponential decay + linear nudge toward zero
    v *= (-dt * 20.0_f64).exp();
    v -= sign(v) * dt;
    // Snap to zero below threshold
    if v.abs() < 1.0 { v = 0.0; }
    v
}

/// Overflow-safe accumulator add
fn add_safe(acc: f64, x: f64) -> f64 {
    let c = acc + x;
    if c.is_finite() { c } else { acc }
}

// ──────────────────────────────── state ──────────────────────────────────────

struct State {
    // QPC timestamps of when each directional key was pressed (0 = not held)
    left_tick: i64,
    right_tick: i64,
    up_tick: i64,
    down_tick: i64,
    /// QPC tick of the last physics update (0 = not started)
    motion_tick: i64,
    /// Horizontal / vertical velocity (px/s conceptually)
    h_vel: f64,
    v_vel: f64,
    /// Sub-pixel accumulator
    h_accum: f64,
    v_accum: f64,
    /// Whether the background thread should keep ticking
    active: bool,
    /// Tuning parameters
    h_accel_ratio: f64,
    v_accel_ratio: f64,
    max_speed: f64,
    /// Window for simultaneous left+right / up+down = "mid key" (in QPC ticks)
    mid_key_window: i64,
}

impl State {
    fn any_key_held(&self) -> bool {
        self.left_tick > 0 || self.right_tick > 0 || self.up_tick > 0 || self.down_tick > 0
    }

    fn reset(&mut self) {
        self.left_tick = 0;
        self.right_tick = 0;
        self.up_tick = 0;
        self.down_tick = 0;
        self.motion_tick = 0;
        self.h_vel = 0.0;
        self.v_vel = 0.0;
        self.h_accum = 0.0;
        self.v_accum = 0.0;
        self.active = false;
    }
}

// ──────────────────────────────── public API ─────────────────────────────────

pub type ActionFn = dyn Fn(i32, i32, &str) + Send + Sync + 'static;

/// 2-D acceleration physics model with a background ticker thread.
///
/// `AccModel2D` is `Send + Sync` because it wraps `Arc<(Mutex<State>, Condvar)>`.
pub struct AccModel2D {
    inner: Arc<(Mutex<State>, Condvar)>,
}

impl AccModel2D {
    /// Create a new model.
    ///
    /// * `action`        – callback invoked each tick with (dx, dy, phase)
    /// * `h_accel_ratio` – horizontal acceleration multiplier (e.g. 15 for cursor, 240 for mouse)
    /// * `v_accel_ratio` – vertical acceleration multiplier (0 = same as h)
    /// * `max_speed`     – velocity cap (pixels/s); use f64::INFINITY for uncapped
    pub fn new(
        action: Arc<ActionFn>,
        h_accel_ratio: f64,
        v_accel_ratio: f64,
        max_speed: f64,
    ) -> Self {
        let freq = qpf();
        let inner = Arc::new((
            Mutex::new(State {
                left_tick: 0,
                right_tick: 0,
                up_tick: 0,
                down_tick: 0,
                motion_tick: 0,
                h_vel: 0.0,
                v_vel: 0.0,
                h_accum: 0.0,
                v_accum: 0.0,
                active: false,
                h_accel_ratio,
                v_accel_ratio: if v_accel_ratio == 0.0 { h_accel_ratio } else { v_accel_ratio },
                max_speed,
                mid_key_window: (freq as f64 * 0.1) as i64, // 100 ms
            }),
            Condvar::new(),
        ));

        let inner_clone = Arc::clone(&inner);
        thread::Builder::new()
            .name("clx-acc-ticker".into())
            .spawn(move || ticker_thread(inner_clone, action))
            .expect("failed to spawn acc ticker thread");

        AccModel2D { inner }
    }

    fn press_dir(inner: &Arc<(Mutex<State>, Condvar)>, tick_field: fn(&mut State) -> &mut i64) {
        let (lock, cvar) = inner.as_ref();
        let mut st = lock.lock().unwrap();
        let t = tick_field(&mut st);
        if *t == 0 {
            *t = qpc();
        }
        if !st.active {
            st.active = true;
            st.motion_tick = 0; // trigger fast-start on first tick
            cvar.notify_all();
        }
    }

    fn release_dir(inner: &Arc<(Mutex<State>, Condvar)>, tick_field: fn(&mut State) -> &mut i64) {
        let (lock, _) = inner.as_ref();
        let mut st = lock.lock().unwrap();
        *tick_field(&mut st) = 0;
    }

    pub fn press_left(&self) {
        Self::press_dir(&self.inner, |s| &mut s.left_tick);
    }
    pub fn release_left(&self) {
        Self::release_dir(&self.inner, |s| &mut s.left_tick);
    }
    pub fn press_right(&self) {
        Self::press_dir(&self.inner, |s| &mut s.right_tick);
    }
    pub fn release_right(&self) {
        Self::release_dir(&self.inner, |s| &mut s.right_tick);
    }
    pub fn press_up(&self) {
        Self::press_dir(&self.inner, |s| &mut s.up_tick);
    }
    pub fn release_up(&self) {
        Self::release_dir(&self.inner, |s| &mut s.up_tick);
    }
    pub fn press_down(&self) {
        Self::press_dir(&self.inner, |s| &mut s.down_tick);
    }
    pub fn release_down(&self) {
        Self::release_dir(&self.inner, |s| &mut s.down_tick);
    }

    /// Force-stop the model (all keys up, velocity zeroed)
    pub fn stop(&self) {
        let (lock, _) = self.inner.as_ref();
        let mut st = lock.lock().unwrap();
        st.reset();
    }
}

// SAFETY: AccModel2D only holds Arc<(Mutex<State>, Condvar)> which are Sync.
unsafe impl Sync for AccModel2D {}
unsafe impl Send for AccModel2D {}

// ──────────────────────────────── ticker thread ──────────────────────────────

fn ticker_thread(inner: Arc<(Mutex<State>, Condvar)>, action: Arc<ActionFn>) {
    let (lock, cvar) = inner.as_ref();
    loop {
        // ── Wait until activated ──────────────────────────────────────────────
        {
            let mut st = lock.lock().unwrap();
            while !st.active {
                st = cvar.wait(st).unwrap();
            }
        }

        // ── Tick loop ─────────────────────────────────────────────────────────
        loop {
            thread::sleep(Duration::from_millis(16));

            let now = qpc();
            let freq = qpf() as f64;

            let mut st = lock.lock().unwrap();
            if !st.active {
                break;
            }

            let dt = if st.motion_tick == 0 {
                // First tick → fast-start call, init accumulator
                st.motion_tick = now;
                let h_sign = sign(
                    if st.right_tick > 0 { 1.0 } else { 0.0 }
                        - if st.left_tick > 0 { 1.0 } else { 0.0 },
                );
                let v_sign = sign(
                    if st.down_tick > 0 { 1.0 } else { 0.0 }
                        - if st.up_tick > 0 { 1.0 } else { 0.0 },
                );
                st.h_accum = h_sign;
                st.v_accum = v_sign;
                drop(st);
                action(0, 0, "启动");
                continue;
            } else {
                (now - st.motion_tick) as f64 / freq
            };
            st.motion_tick = now;

            // ── Compute hold durations for each direction ──────────────────
            let left_s = if st.left_tick > 0 { (now - st.left_tick) as f64 / freq } else { 0.0 };
            let right_s = if st.right_tick > 0 { (now - st.right_tick) as f64 / freq } else { 0.0 };
            let up_s = if st.up_tick > 0 { (now - st.up_tick) as f64 / freq } else { 0.0 };
            let down_s = if st.down_tick > 0 { (now - st.down_tick) as f64 / freq } else { 0.0 };

            // ── Simultaneous opposite keys → "mid key" ─────────────────────
            let mid_win = st.mid_key_window;
            if st.left_tick > 0 && st.right_tick > 0 {
                let diff = (st.right_tick - st.left_tick).abs();
                if diff < mid_win {
                    let s = sign((st.right_tick - st.left_tick) as f64) as i32;
                    st.reset();
                    drop(st);
                    action(s, 0, "横中键");
                    break;
                }
            }
            if st.up_tick > 0 && st.down_tick > 0 {
                let diff = (st.down_tick - st.up_tick).abs();
                if diff < mid_win {
                    let s = sign((st.down_tick - st.up_tick) as f64) as i32;
                    st.reset();
                    drop(st);
                    action(0, s, "纵中键");
                    break;
                }
            }

            // ── Physics integration ────────────────────────────────────────
            let h_accel = ma(right_s - left_s) * st.h_accel_ratio;
            let v_accel = ma(down_s - up_s) * st.v_accel_ratio;

            st.h_vel = add_safe(st.h_vel, h_accel * dt);
            st.v_vel = add_safe(st.v_vel, v_accel * dt);
            st.h_vel = damping(st.h_vel, h_accel, dt, st.max_speed);
            st.v_vel = damping(st.v_vel, v_accel, dt, st.max_speed);

            st.h_accum = add_safe(st.h_accum, st.h_vel * dt);
            st.v_accum = add_safe(st.v_accum, st.v_vel * dt);

            // Truncate to integer output, keep remainder
            let h_out = st.h_accum as i32;
            let v_out = st.v_accum as i32;
            st.h_accum -= h_out as f64;
            st.v_accum -= v_out as f64;

            let h_vel = st.h_vel;
            let v_vel = st.v_vel;
            let any_key = st.any_key_held();
            drop(st);

            if h_out != 0 || v_out != 0 {
                action(h_out, v_out, "移动");
            }

            // ── Stop when fully at rest and no keys held ───────────────────
            if h_vel == 0.0 && v_vel == 0.0 && h_out == 0 && v_out == 0 && !any_key {
                let mut st = lock.lock().unwrap();
                st.active = false;
                drop(st);
                action(0, 0, "止动");
                break;
            }
        }
    }
}
