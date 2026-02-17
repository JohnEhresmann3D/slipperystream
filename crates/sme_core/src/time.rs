use std::time::Instant;

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
        let now = Instant::now();
        self.real_dt = now.duration_since(self.last_instant).as_secs_f64();
        self.last_instant = now;

        // Spiral-of-death cap
        if self.real_dt > self.max_accumulator {
            log::warn!(
                "Frame took {:.1}ms — capping accumulator to {}ms",
                self.real_dt * 1000.0,
                self.max_accumulator * 1000.0
            );
            self.real_dt = self.max_accumulator;
        }

        self.accumulator += self.real_dt;
        self.steps_this_frame = 0;
        self.frame_count += 1;

        // FPS smoothing
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
