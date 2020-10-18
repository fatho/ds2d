//! Implements batched rendering of sprites.

use cgmath::{Matrix3, Rad, Vector2};

use super::{
    context::{BackendError, Buffer, Program, Texture, VertexArray},
    Color, GraphicsError, Rect, RenderState, Texture2D,
};
use crate::{Context, GameResult};

pub struct SpriteBatch {
    program: Program,
    /// The sprite vertices
    #[allow(unused)]
    vbo: Buffer,
    /// Each quad will be represented by 6 consecutive indices in this buffer.
    ebo: Buffer,
    vao: VertexArray,
}

impl SpriteBatch {
    /// Create a new sprite batch that can batch up to `max_sprites` at once.
    /// (Or less, in case the texture needs to be switched).
    pub fn new(max_sprites: u32) -> Result<Self, GraphicsError> {
        unimplemented!()
    }
}
