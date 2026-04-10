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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, Ordering};

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
            let s = if t > 0.0 { 1.0 } else if t < 0.0 { -1.0 } else { 0.0 };
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
        eprintln!("mouse_speed=3000, 166 FPS → {:.0} px (err={:.0})", disp, err);
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
        eprintln!("cursor_speed=60, 166 FPS → {:.0} keys (err={:.0})", disp, err);
        assert!(err < 10.0, "expected ~60 keys, got {:.0}", disp);
    }

    #[test]
    fn frame_rate_independence() {
        // Same speed should produce similar displacement at different FPS.
        let d30  = simulate_1d(500.0, 1.0, 30.0);
        let d60  = simulate_1d(500.0, 1.0, 60.0);
        let d120 = simulate_1d(500.0, 1.0, 120.0);
        let d166 = simulate_1d(500.0, 1.0, 166.0);

        eprintln!("speed=500: 30fps={:.0}, 60fps={:.0}, 120fps={:.0}, 166fps={:.0}",
            d30, d60, d120, d166);

        // All should be within 15% of each other.
        let avg = (d30 + d60 + d120 + d166) / 4.0;
        for (fps, d) in [(30, d30), (60, d60), (120, d120), (166, d166)] {
            let pct = ((d - avg) / avg).abs() * 100.0;
            assert!(pct < 15.0, "{}fps deviated {:.1}% from average ({:.0} vs {:.0})",
                fps, pct, d, avg);
        }
    }
}
