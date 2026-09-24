/// Unit tests for AccModel2D — verify frame-rate independence and speed calibration.
///
/// Key property: `speed_ratio` = displacement in the FIRST second of holding.
/// E.g. mouse_speed=3000 → cursor moves ~3000px in the first second, at any FPS.
/// The acceleration curve is non-linear (polynomial + exponential), so the
/// second second will be faster than the first. The number represents the
/// first-second displacement only.
///
/// The physics is a double integral of the acceleration curve ma(t):
///   ma_raw(t) = e^t - 1 + 3 + 4t + 9t² + 16t³
///   ma(t) = ma_raw(t) / K_RAW       (K_RAW ≈ 3.935, calibration constant)
///   vel(t) = ∫₀ᵗ ma(s) × speed_ratio ds
///   pos(t) = ∫₀ᵗ vel(s) ds
///
/// Total displacement after 1s ≈ speed_ratio (by design, via K_RAW).

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;

    /// Simulate AccModel physics for `duration_s` at `fps` frames per second,
    /// with the given `speed_ratio`, holding the right direction.
    /// Returns total displacement (sum of all MOVE outputs).
    fn simulate_1d(speed_ratio: f64, duration_s: f64, fps: f64) -> f64 {
        let dt = 1.0 / fps;
        let steps = (duration_s / dt) as usize;

        let mut vel = 0.0_f64;
        let mut accum = 0.0_f64;
        let mut total_output = 0i64;

        // ma() from acc_model.rs — includes K_RAW calibration
        const K_RAW: f64 = 3.935;
        fn ma(t: f64) -> f64 {
            let s = if t > 0.0 {
                1.0
            } else if t < 0.0 {
                -1.0
            } else {
                0.0
            };
            let a = t.abs();
            s * ((a.exp() - 1.0) + 3.0 + 4.0 * a + 9.0 * a * a + 16.0 * a * a * a) / K_RAW
        }

        for step in 0..steps {
            let hold_s = (step + 1) as f64 * dt; // how long key held
            let accel = ma(hold_s) * speed_ratio;

            // damping: when accel and vel same sign, no damping
            // (this matches the acc_model.rs damping function for hold case)
            vel += accel * dt;

            accum += vel * dt;
            let out = accum as i64;
            accum -= out as f64;
            total_output += out;
        }

        total_output as f64
    }

    #[test]
    fn mouse_3000_moves_3000px_in_1s_at_60fps() {
        let disp = simulate_1d(3000.0, 1.0, 60.0);
        let err = (disp - 3000.0).abs();
        eprintln!("mouse_speed=3000, 60 FPS → {:.0} px (err={:.0})", disp, err);
        assert!(err < 200.0, "expected ~3000px, got {:.0}", disp);
    }

    #[test]
    fn mouse_3000_moves_3000px_in_1s_at_166fps() {
        let disp = simulate_1d(3000.0, 1.0, 166.0);
        let err = (disp - 3000.0).abs();
        eprintln!(
            "mouse_speed=3000, 166 FPS → {:.0} px (err={:.0})",
            disp, err
        );
        assert!(err < 200.0, "expected ~3000px, got {:.0}", disp);
    }

    #[test]
    fn mouse_3000_moves_3000px_in_1s_at_30fps() {
        let disp = simulate_1d(3000.0, 1.0, 30.0);
        let err = (disp - 3000.0).abs();
        eprintln!("mouse_speed=3000, 30 FPS → {:.0} px (err={:.0})", disp, err);
        assert!(err < 350.0, "expected ~3000px, got {:.0}", disp);
    }

    #[test]
    fn cursor_60_produces_60_keys_in_1s() {
        let disp = simulate_1d(60.0, 1.0, 166.0);
        let err = (disp - 60.0).abs();
        eprintln!(
            "cursor_speed=60, 166 FPS → {:.0} keys (err={:.0})",
            disp, err
        );
        assert!(err < 10.0, "expected ~60 keys, got {:.0}", disp);
    }

    #[test]
    fn frame_rate_independence() {
        // Same speed should produce similar displacement at different FPS.
        let d30 = simulate_1d(500.0, 1.0, 30.0);
        let d60 = simulate_1d(500.0, 1.0, 60.0);
        let d120 = simulate_1d(500.0, 1.0, 120.0);
        let d166 = simulate_1d(500.0, 1.0, 166.0);

        eprintln!(
            "speed=500: 30fps={:.0}, 60fps={:.0}, 120fps={:.0}, 166fps={:.0}",
            d30, d60, d120, d166
        );

        // All should be within 15% of each other.
        let avg = (d30 + d60 + d120 + d166) / 4.0;
        for (fps, d) in [(30, d30), (60, d60), (120, d120), (166, d166)] {
            let pct = ((d - avg) / avg).abs() * 100.0;
            assert!(
                pct < 15.0,
                "{}fps deviated {:.1}% from average ({:.0} vs {:.0})",
                fps,
                pct,
                d,
                avg
            );
        }
    }
}

/// The runaway guard: a model whose key-up never arrives must stop itself.
///
/// This is the bug from 2026-09-22 — CLX+Z tapped once, but the foreground
/// landed on a window a non-elevated hook cannot see past (an elevated app, or
/// the DWM `Ghost` of an unresponsive one), so the key-up was never delivered
/// and the cycle accelerated to max_speed forever. Same shape as the Space
/// flood recorded in `tmp/2026-09-11-clx-wedge-incident.md`.
#[cfg(test)]
mod watchdog_tests {
    use crate::acc_model::AccModel2D;
    use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
    use std::sync::Arc;

    /// Drive a model by hand (no ticker thread), counting MOVE actions.
    fn model(watchdog_alive: Arc<AtomicBool>, install: bool) -> (AccModel2D, Arc<AtomicI64>) {
        crate::acc_model::set_external_tick(true);
        let moves = Arc::new(AtomicI64::new(0));
        let m2 = Arc::clone(&moves);
        let m = AccModel2D::new(
            Arc::new(move |dx, _dy, phase| {
                if phase == "MOVE" {
                    m2.fetch_add(dx.abs() as i64, Ordering::Relaxed);
                }
            }),
            // A high ratio so the integral crosses whole units quickly: these
            // tests assert on *whether* motion continues, not how fast.
            3000.0,
            3000.0,
            250.0,
        );
        if install {
            let alive = Arc::clone(&watchdog_alive);
            m.set_watchdog(Arc::new(move || alive.load(Ordering::Relaxed)));
        }
        (m, moves)
    }

    #[test]
    fn watchdog_stops_a_model_whose_key_up_never_arrived() {
        let alive = Arc::new(AtomicBool::new(true));
        let (m, moves) = model(Arc::clone(&alive), true);

        // Key down, and a few ticks of genuine motion.
        m.press_right();
        for _ in 0..60 {
            m.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let while_held = moves.load(Ordering::Relaxed);
        assert!(while_held > 0, "expected motion while the key is held");

        // The key is physically released, but no key-up event ever arrives —
        // so `stop()` is never called. Only the watchdog can notice.
        alive.store(false, Ordering::Relaxed);
        m.tick_once();
        let after_release = moves.load(Ordering::Relaxed);

        // Everything past this point must be silence, however long we tick.
        for _ in 0..120 {
            m.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            moves.load(Ordering::Relaxed),
            after_release,
            "the model kept moving after the watchdog said the key was up"
        );
    }

    #[test]
    fn without_a_watchdog_the_model_runs_away() {
        // Characterises the bug: the same sequence with no watchdog keeps
        // emitting forever. If this ever stops on its own, the physics changed
        // and the guard above may no longer be the only thing saving us.
        let alive = Arc::new(AtomicBool::new(true));
        let (m, moves) = model(alive, false);
        m.press_right();
        for _ in 0..60 {
            m.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let before = moves.load(Ordering::Relaxed);
        for _ in 0..60 {
            m.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(
            moves.load(Ordering::Relaxed) > before,
            "expected the unguarded model to keep running (that is the bug)"
        );
    }

    #[test]
    fn a_watchdog_that_stays_true_does_not_interfere() {
        let alive = Arc::new(AtomicBool::new(true));
        let (m, moves) = model(Arc::clone(&alive), true);
        m.press_right();
        for _ in 0..60 {
            m.tick_once();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(
            moves.load(Ordering::Relaxed) > 0,
            "a live watchdog must not suppress normal motion"
        );
    }
}
