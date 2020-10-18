pub(crate) mod context;

/// Handle keyboard input
pub mod keyboard {
    use crate::Context;
    use cgmath::Vector2;

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
    pub fn axis1d(ctx: &mut Context, negative_key: KeyCode, positive_key: KeyCode) -> f32 {
        let mut output = 0;
        if is_down(ctx, negative_key) {
            output -= 1
        }
        if is_down(ctx, positive_key) {
            output += 1
        }
        output as f32
    }

    /// Emulate two analog axes with four keys (useful for movement with WSAD).
    /// The magnitude of the resulting vector is at most 1.
    pub fn axis2d(
        ctx: &mut Context,
        x_neg: KeyCode,
        x_pos: KeyCode,
        y_neg: KeyCode,
        y_pos: KeyCode,
    ) -> Vector2<f32> {
        use cgmath::InnerSpace;

        let x = axis1d(ctx, x_neg, x_pos);
        let y = axis1d(ctx, y_neg, y_pos);
        let v = Vector2::new(x, y);
        let len = v.magnitude();
        if len > 1.0 {
            v / len
        } else {
            v
        }
    }

    /// Return the text that was entered since the last update.
    pub fn text(ctx: &mut Context) -> &str {
        &ctx.keyboard.unicode_text
    }
}

/// Handle mouse input
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
