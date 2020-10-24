use super::{Context, GameResult};
use glutin::dpi::PhysicalSize;

mod color;
pub use color::Color;
mod error;
pub use error::GraphicsError;
mod rect;
pub use rect::Rect;
mod texture;
pub use texture::{Texture2D, TextureView2D};
mod mesh;
pub use mesh::Mesh;
mod sprite;
pub use sprite::Sprite;
mod blend;
pub use blend::BlendMode;
mod batch;
pub use batch::{BatchRender, Quad, QuadBuilder};

pub mod text;

pub mod primitives;
pub use primitives::RenderState;

pub(crate) mod context;

pub mod transform;

pub use context::BackendError;

/// Implemented by every "well-behaved" entity that can be drawn, meaning
/// - it respects the current coordinate system (TODO)
pub trait Drawable {
    fn draw(&mut self, ctx: &mut Context, state: RenderState) -> GameResult<()>;
}

pub fn draw<T: Drawable>(ctx: &mut Context, drawable: &mut T) -> GameResult<()> {
    let initial_state = RenderState {
        transform: ctx.graphics.pixel_projection,
    };
    drawable.draw(ctx, initial_state)
}

/// Sets the blend mode used for subsequent draw calls.
pub fn set_blend_mode(ctx: &mut Context, blend: Option<BlendMode>) -> GameResult<()> {
    ctx.graphics.set_blend_mode(blend).map_err(|e| e.into())
}

pub fn clear(_ctx: &mut Context, color: Color) {
    unsafe {
        gl::ClearColor(color.r, color.g, color.b, color.a);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

pub fn screen_size(ctx: &mut Context) -> PhysicalSize<u32> {
    ctx.graphics.screen_size
}

pub fn scale_factor(ctx: &mut Context) -> f64 {
    ctx.graphics.scale_factor
}
