use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct TimerContext {
    pub last_frame: Instant,
    pub current_frame: Instant,
    /// Accumulated, but not yet processed frame time.
    pub accumulator: Duration,
}

impl TimerContext {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame: now,
            current_frame: now,
            accumulator: Duration::default(),
        }
    }

    pub fn tick(&mut self) {
        self.last_frame = self.current_frame;
        self.current_frame = Instant::now();
        let delta = self.current_frame - self.last_frame;
        self.accumulator += delta;
    }
}
