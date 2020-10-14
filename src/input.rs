/// Handle keyboard input
pub mod keyboard {
    use crate::Context;

    pub type KeyCode = glutin::event::VirtualKeyCode;

    pub fn is_down(ctx: &mut Context, key: KeyCode) -> bool {
        ctx.keyboard.pressed_keys.contains(&key)
    }

    pub fn is_up(ctx: &mut Context, key: KeyCode) -> bool {
        !is_down(ctx, key)
    }

    /// Emulate an analog axis with two keys (useful for movement with WSAD).
    /// The `negative_key` corresponds to -1, the `positive_key` to 1.
    /// If both keys are or no key is pressed, 0 is returned.
    pub fn axis(ctx: &mut Context, negative_key: KeyCode, positive_key: KeyCode) -> i32 {
        let mut output = 0;
        if is_down(ctx, negative_key) {
            output -= 1
        }
        if is_down(ctx, positive_key) {
            output += 1
        }
        output
    }

    /// Returns `axis` converted as float.
    pub fn axis_f32(ctx: &mut Context, negative_key: KeyCode, positive_key: KeyCode) -> f32 {
        axis(ctx, negative_key, positive_key) as f32
    }

    /// Return the text that was entered since the last update.
    pub fn text(ctx: &mut Context) -> &str {
        &ctx.keyboard.unicode_text
    }
}

pub mod mouse {
    use crate::Context;
    use glutin::{dpi::PhysicalPosition, event::MouseButton};

    type Button = MouseButton;

    pub fn position(ctx: &mut Context) -> PhysicalPosition<f64> {
        ctx.mouse.position
    }

    pub fn is_down(ctx: &mut Context, key: Button) -> bool {
        ctx.mouse.pressed_buttons.contains(&key)
    }

    pub fn is_up(ctx: &mut Context, key: Button) -> bool {
        !is_down(ctx, key)
    }
}
