use super::{Context, GameResult};
use glow::HasContext;

/// A color represented as normalized 32 bit float RGBA value.
#[derive(Debug, Copy, Clone)]
pub struct Color {
    /// red component
    pub r: f32,
    /// green component
    pub g: f32,
    /// blue component
    pub b: f32,
    /// alpha component
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::from_rgba(0.0, 0.0, 0.0, 1.0);
    pub const CORNFLOWER_BLUE: Color =
        Color::from_rgba(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
    pub const MAGENTA: Color = Color::from_rgba(1.0, 0.0, 1.0, 1.0);

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::from_rgba(r, g, b, 1.0)
    }

    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba_u8(r, g, b, 255)
    }
}

pub fn clear(ctx: &mut Context, color: Color) {
    unsafe {
        ctx.graphics
            .gl
            .clear_color(color.r, color.g, color.b, color.a);
        ctx.graphics.gl.clear(glow::COLOR_BUFFER_BIT);
    }
}
