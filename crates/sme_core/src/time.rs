//! Fixed-timestep simulation clock.
//!
//! The engine runs simulation at a locked 60 Hz (`fixed_dt = 1/60`). Each render
//! frame, `begin_frame()` measures wall-clock elapsed time and feeds it into an
//! accumulator. The game loop then calls `should_step()` in a while-loop, consuming
//! one `fixed_dt` slice per step -- this guarantees deterministic simulation
//! regardless of display refresh rate.
//!
//! **Spiral-of-death cap:** If a frame takes longer than `max_accumulator` (250ms),
//! the excess time is discarded. Without this cap, a single slow frame would queue
//! dozens of catch-up steps, which themselves take time, creating a feedback loop
//! that makes the game unrecoverably slow.
//!
//! After all fixed steps are consumed, `end_frame()` computes `interpolation_alpha`
//! (the fractional leftover in the accumulator) for optional visual interpolation
//! between the last two simulation states.

// web-time resolves to std::time on native targets and to a
// performance.now()-backed clock on wasm32, where std's Instant panics.
use web_time::Instant;

const FPS_SAMPLE_COUNT: usize = 60;

pub struct TimeState {
    pub fixed_dt: f64,
    pub max_accumulator: f64,
    accumulator: f64,
    pub total_time: f64,
    pub fixed_step_count: u64,
    pub frame_count: u64,
    pub steps_this_frame: u32,
    pub real_dt: f64,
    last_instant: Instant,
    pub interpolation_alpha: f64,

    fps_samples: [f64; FPS_SAMPLE_COUNT],
    fps_sample_index: usize,
    pub smoothed_fps: f64,
    pub smoothed_frame_time_ms: f64,
}

impl TimeState {
    pub fn new() -> Self {
        Self {
            fixed_dt: 1.0 / 60.0,
            max_accumulator: 0.25,
            accumulator: 0.0,
            total_time: 0.0,
            fixed_step_count: 0,
            frame_count: 0,
            steps_this_frame: 0,
            real_dt: 0.0,
            last_instant: Instant::now(),
            interpolation_alpha: 0.0,
            fps_samples: [1.0 / 60.0; FPS_SAMPLE_COUNT],
            fps_sample_index: 0,
            smoothed_fps: 60.0,
            smoothed_frame_time_ms: 16.667,
        }
    }

    pub fn begin_frame(&mut self) {
        let elapsed = self.measure_elapsed();
        self.advance_frame(elapsed, false, false);
    }

    /// Debug pause discards simulation debt but still measures actual frame time.
    /// `single_step` queues exactly one tick; callers must still call `should_step`.
    pub fn begin_paused_frame(&mut self, single_step: bool) {
        let elapsed = self.measure_elapsed();
        self.advance_frame(elapsed, true, single_step);
    }

    /// Rebase after focus/lifecycle transitions; hidden time is not simulation time.
    pub fn reset_elapsed(&mut self) {
        self.last_instant = Instant::now();
        self.accumulator = 0.0;
        self.interpolation_alpha = 0.0;
    }

    fn measure_elapsed(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_instant).as_secs_f64();
        self.last_instant = now;
        elapsed
    }

    fn advance_frame(&mut self, elapsed: f64, paused: bool, single_step: bool) {
        // Retain public configuration compatibility while preventing invalid settings
        // from creating infinite tick loops. A future builder can make these private.
        if !self.fixed_dt.is_finite() || self.fixed_dt <= 0.0 {
            self.fixed_dt = 1.0 / 60.0;
        }
        if !self.max_accumulator.is_finite() || self.max_accumulator < self.fixed_dt {
            self.max_accumulator = 0.25_f64.max(self.fixed_dt);
        }
        self.real_dt = if elapsed.is_finite() {
            elapsed.max(0.0)
        } else {
            0.0
        };
        if paused {
            self.accumulator = if single_step { self.fixed_dt } else { 0.0 };
        } else {
            // Cap the total, including debt left by a caller that stopped stepping.
            self.accumulator = (self.accumulator + self.real_dt).min(self.max_accumulator);
        }
        self.steps_this_frame = 0;
        self.frame_count += 1;
        // Diagnostics retain the uncapped wall-clock delta, including long stalls.
        self.fps_samples[self.fps_sample_index] = self.real_dt;
        self.fps_sample_index = (self.fps_sample_index + 1) % FPS_SAMPLE_COUNT;
        let avg_dt: f64 = self.fps_samples.iter().sum::<f64>() / FPS_SAMPLE_COUNT as f64;
        self.smoothed_frame_time_ms = avg_dt * 1000.0;
        self.smoothed_fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };
    }

    pub fn should_step(&mut self) -> bool {
        if self.accumulator >= self.fixed_dt {
            self.accumulator -= self.fixed_dt;
            self.total_time += self.fixed_dt;
            self.fixed_step_count += 1;
            self.steps_this_frame += 1;
            true
        } else {
            false
        }
    }

    pub fn end_frame(&mut self) {
        self.interpolation_alpha = self.accumulator / self.fixed_dt;
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl TimeState {
    /// Simulate a frame with a known delta time, bypassing `Instant::now()`.
    /// Mirrors the logic of `begin_frame()` but with an injected dt.
    fn simulate_frame(&mut self, dt: f64) {
        self.advance_frame(dt, false, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-9;

    #[test]
    fn paused_frames_have_no_debt_and_single_step_counts_only_real_ticks() {
        let mut ts = TimeState::new();
        for _ in 0..600 {
            ts.advance_frame(0.05, true, false);
            assert!(!ts.should_step());
            ts.end_frame();
            assert_eq!(ts.interpolation_alpha, 0.0);
        }
        assert_eq!(ts.fixed_step_count, 0);
        ts.advance_frame(10.0, true, true);
        assert!(ts.should_step());
        assert!(!ts.should_step());
        assert_eq!(ts.fixed_step_count, 1);
        ts.advance_frame(ts.fixed_dt, false, false);
        assert!(ts.should_step());
        assert!(!ts.should_step());
    }

    #[test]
    fn undrained_debt_is_bounded_and_stalls_are_measured() {
        let mut ts = TimeState::new();
        for _ in 0..60 {
            ts.simulate_frame(1.0);
        }
        assert_eq!(ts.accumulator, ts.max_accumulator);
        assert_eq!(ts.smoothed_fps, 1.0);
        let mut ticks = 0;
        while ts.should_step() {
            ticks += 1;
        }
        assert_eq!(ticks, 15);
    }

    #[test]
    fn invalid_configuration_is_normalized_and_focus_reset_discards_debt() {
        let mut ts = TimeState::new();
        ts.fixed_dt = 0.0;
        ts.max_accumulator = f64::NAN;
        ts.simulate_frame(1.0);
        assert!(ts.fixed_dt > 0.0 && ts.max_accumulator.is_finite());
        ts.reset_elapsed();
        assert!(!ts.should_step());
        assert_eq!(ts.interpolation_alpha, 0.0);
    }

    #[test]
    fn test_new_defaults() {
        let ts = TimeState::new();
        assert!((ts.fixed_dt - 1.0 / 60.0).abs() < EPSILON);
        assert!((ts.max_accumulator - 0.25).abs() < EPSILON);
        assert!((ts.total_time - 0.0).abs() < EPSILON);
        assert_eq!(ts.fixed_step_count, 0);
        assert_eq!(ts.frame_count, 0);
        assert_eq!(ts.steps_this_frame, 0);
        assert!((ts.real_dt - 0.0).abs() < EPSILON);
        assert!((ts.interpolation_alpha - 0.0).abs() < EPSILON);
        assert!((ts.smoothed_fps - 60.0).abs() < 0.1);
        assert!((ts.smoothed_frame_time_ms - 16.667).abs() < 0.01);
    }

    #[test]
    fn test_should_step_consumes_accumulator() {
        let mut ts = TimeState::new();
        let dt = 1.0 / 60.0;
        ts.simulate_frame(dt);

        // First call: enough accumulator for one step
        assert!(ts.should_step());
        assert_eq!(ts.fixed_step_count, 1);
        assert_eq!(ts.steps_this_frame, 1);
        assert!((ts.total_time - dt).abs() < EPSILON);

        // Second call: accumulator should be drained
        assert!(!ts.should_step());
        assert_eq!(ts.fixed_step_count, 1);
        assert_eq!(ts.steps_this_frame, 1);
    }

    #[test]
    fn test_multiple_steps_per_frame() {
        let mut ts = TimeState::new();
        let dt = 3.0 / 60.0; // three fixed steps worth
        ts.simulate_frame(dt);

        assert!(ts.should_step());
        assert!(ts.should_step());
        assert!(ts.should_step());
        assert!(!ts.should_step());

        assert_eq!(ts.steps_this_frame, 3);
        assert_eq!(ts.fixed_step_count, 3);
        assert!((ts.total_time - 3.0 * ts.fixed_dt).abs() < EPSILON);
    }

    #[test]
    fn test_spiral_of_death_cap() {
        let mut ts = TimeState::new();
        ts.simulate_frame(1.0); // 1 second, way over max_accumulator of 0.25

        // Actual elapsed time stays visible even though simulation debt is capped.
        assert!((ts.real_dt - 1.0).abs() < EPSILON);

        // Count how many steps are consumed
        let mut step_count = 0u32;
        while ts.should_step() {
            step_count += 1;
        }

        // 0.25 / (1/60) = 15
        assert_eq!(step_count, 15);
        assert_eq!(ts.steps_this_frame, 15);
    }

    #[test]
    fn test_interpolation_alpha() {
        let mut ts = TimeState::new();
        let dt = 1.5 * ts.fixed_dt; // 1.5 steps worth
        ts.simulate_frame(dt);

        // Consume exactly one step
        assert!(ts.should_step());
        // After one step, 0.5 * fixed_dt should remain in accumulator
        assert!(!ts.should_step());

        ts.end_frame();

        // alpha = remaining_accumulator / fixed_dt ≈ 0.5
        assert!(
            (ts.interpolation_alpha - 0.5).abs() < 1e-6,
            "Expected alpha ≈ 0.5, got {}",
            ts.interpolation_alpha
        );
    }

    #[test]
    fn test_frame_count_increments() {
        let mut ts = TimeState::new();
        assert_eq!(ts.frame_count, 0);

        ts.simulate_frame(1.0 / 60.0);
        assert_eq!(ts.frame_count, 1);

        ts.simulate_frame(1.0 / 60.0);
        assert_eq!(ts.frame_count, 2);

        ts.simulate_frame(1.0 / 60.0);
        assert_eq!(ts.frame_count, 3);

        for _ in 0..10 {
            ts.simulate_frame(1.0 / 60.0);
        }
        assert_eq!(ts.frame_count, 13);
    }

    #[test]
    fn test_fps_smoothing() {
        let mut ts = TimeState::new();
        let dt = 1.0 / 30.0; // 30 FPS

        // Fill all 60 samples with the 30-FPS dt to flush the initial values
        for _ in 0..FPS_SAMPLE_COUNT {
            ts.simulate_frame(dt);
            // Drain accumulator so it doesn't grow unboundedly
            while ts.should_step() {}
        }

        assert!(
            (ts.smoothed_fps - 30.0).abs() < 0.1,
            "Expected smoothed_fps ≈ 30, got {}",
            ts.smoothed_fps
        );
        assert!(
            (ts.smoothed_frame_time_ms - 33.333).abs() < 0.1,
            "Expected smoothed_frame_time_ms ≈ 33.333, got {}",
            ts.smoothed_frame_time_ms
        );
    }

    #[test]
    fn test_accumulator_does_not_go_negative() {
        let mut ts = TimeState::new();

        // Try several different frame deltas
        let deltas = [1.0 / 60.0, 2.5 / 60.0, 0.1, 0.001, 0.25];
        for &dt in &deltas {
            ts.simulate_frame(dt);
            while ts.should_step() {}
            ts.end_frame();

            // interpolation_alpha = accumulator / fixed_dt, so if accumulator >= 0
            // and < fixed_dt, alpha should be in [0, 1).
            assert!(
                ts.interpolation_alpha >= 0.0,
                "Alpha went negative: {} for dt={}",
                ts.interpolation_alpha,
                dt
            );
            assert!(
                ts.interpolation_alpha < 1.0,
                "Alpha >= 1.0: {} for dt={} (accumulator should be < fixed_dt)",
                ts.interpolation_alpha,
                dt
            );
        }
    }
}
