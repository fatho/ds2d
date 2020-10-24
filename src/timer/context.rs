use std::time::{Duration, Instant};

#[derive(Debug)]
pub(crate) struct TimerContext {
    /// The desired number of updates to the game state per second.
    pub updates_per_second: f64,
    pub last_frame: Instant,
    pub current_frame: Instant,
    /// Accumulated, but not yet processed frame time.
    pub accumulator: Duration,
}

impl TimerContext {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            updates_per_second: 60.0,
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
