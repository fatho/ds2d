//! Defines a camera interface for controlling which part of the world is visible on scren.

use cgmath::Vector2;
use crate::Context;

pub struct Camera2d {
    /// The world-coordinate that is at the center of the screen.
    pub center: Vector2<f64>,
    /// How many logical screen pixels are covered by one world coordinate.
    pub logical_scale: Vector2<f64>,
}


impl Camera2d {
    /// Map physical pixel coordinates 1:1 with world coordinates.
    pub fn identity_physical(ctx: &mut Context) -> Self {
        let screen = super::screen_size(ctx);
        let dpi_scale = super::scale_factor(ctx);

        let sizef = Vector2 { x: screen.width as f64, y: screen.height as f64 };
        let center = 0.5 * sizef;
        let logical_scale = Vector2::new(1.0, 1.0) / dpi_scale;
        Self { center, logical_scale }
    }

    /// Map logical pixel coordinates 1:1 with world coordinates.
    pub fn identity_logical(ctx: &mut Context) -> Self {
        let screen = super::screen_size(ctx);
        let dpi_scale = super::scale_factor(ctx);

        let sizef = Vector2 { x: screen.width as f64, y: screen.height as f64 };
        let center = 0.5 * sizef / dpi_scale;
        let logical_scale = Vector2::new(1.0, 1.0);
        Self { center, logical_scale }
    }

    // TODO: compute view matrix (world coordinates to physical pixels)
}

// TODO: compute projection matrix (physical pixels to normalized screen space (-1.0 to 1.0))
