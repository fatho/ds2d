use std::time::Duration;

pub(crate) mod context;

use crate::Context;

/// Time that has elapsed since the last frame.
pub fn delta(ctx: &mut Context) -> Duration {
    ctx.timer.current_frame - ctx.timer.last_frame
}

pub fn updates_per_second(ctx: &mut Context) -> f64 {
    ctx.timer.updates_per_second
}

pub fn set_updates_per_second(ctx: &mut Context, new_ups: f64) {
    ctx.timer.updates_per_second = new_ups
}

/// Should be called inside `Game::update` until it returns false,
/// in order to achieve the given update rate with fixed-timestep updates.
/// The rate takes fractional frames into account that are left over at the end of one update run.
/// If the accumulated frame time exceeds `max_updates / updates_per_second`, the remaining
/// frame time is discarded in order to avoid a spiral of death.
pub fn run_fixed_timestep(ctx: &mut Context, max_updates: u32) -> bool {
    let frame_time = Duration::from_secs_f64(1.0 / ctx.timer.updates_per_second);
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

/// The accumulated frame time that wasn't simulated yet using `run_fixed_timestep`.
pub fn remaining_frame_time(ctx: &mut Context) -> Duration {
    ctx.timer.accumulator
}

/// The amount of time simulated during one update.
pub fn timestep(ctx: &mut Context) -> Duration {
    Duration::from_secs_f64(1.0 / ctx.timer.updates_per_second)
}

/// The remaining frame time divided by the timestep.
/// This factor can be used for interpolating positions during draw calls,
/// in order to decouple the simulation and the drawing.
pub fn interpolation_factor(ctx: &mut Context) -> f64 {
    ctx.timer.accumulator.as_secs_f64() * ctx.timer.updates_per_second
}

/// Exponential moving average of the time that has elapsed since the last frame.
pub fn average_delta(ctx: &mut Context) -> Duration {
    Duration::from_secs_f64(ctx.timer.average_delta_seconds)
}
