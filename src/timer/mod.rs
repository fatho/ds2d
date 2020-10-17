use std::time::Duration;

pub (crate) mod context;

use crate::Context;

/// Time that has elapsed since the last frame.
pub fn delta(ctx: &mut Context) -> Duration {
    ctx.timer.current_frame - ctx.timer.last_frame
}

/// Should be called inside `Game::update` until it returns false,
/// in order to achieve the given update rate with fixed-timestep updates.
/// The rate takes fractional frames into account that are left over at the end of one update run.
/// If the accumulated frame time exceeds `max_updates / updates_per_second`, the remaining
/// frame time is discarded in order to avoid a spiral of death.
pub fn run_fixed_timestep(ctx: &mut Context, updates_per_second: f64, max_updates: u32) -> bool {
    let frame_time = Duration::from_secs_f64(1.0 / updates_per_second);
    if ctx.timer.accumulator >= frame_time {
        let max_time = frame_time * max_updates;
        if ctx.timer.accumulator >= max_time {
            log::warn!(
                "Simulation running slow, discarding {:.3}s of frame time",
                (ctx.timer.accumulator - max_time).as_secs_f64()
            );
            ctx.timer.accumulator = max_time;
        }
        ctx.timer.accumulator -= frame_time;
        true
    } else {
        false
    }
}

// TODO: implement FPS timer
