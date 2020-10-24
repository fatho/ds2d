//! A generic 2D triangle mesh.

use cgmath::{Matrix3, Rad, Vector2};

use super::{
    context::{BackendError, Buffer, Program, Texture, VertexArray},
    primitives::BasicPipeline2D,
    primitives::BasicVertex2D,
    primitives::Pipeline,
    primitives::VertexData,
    BlendMode, Color, Rect, RenderState, Texture2D,
};
use crate::{Context, GameResult};

pub struct SpriteBuilder {
    /// The texture to draw.
    texture: Texture2D,
    /// The part of the texture that should be drawn (UV coordinates)
    source: Rect<f32>,
    /// The color that is multiplied with the texture color.
    tint: Color,

    /// The position of the origin in global coordinates.
    position: Vector2<f32>,
    /// The size of the Sprite in global coordinates.
    size: Vector2<f32>,
    /// The origin for position and rotation in local coordinates where `(0, 0)` is the top-left
    /// and `(1, 1)` is the bottom-right corner of the sprite.
    origin: Vector2<f32>,
    /// Rotation angle around the origin.
    rotation: Rad<f32>,
}

impl SpriteBuilder {
    fn new(texture: Texture2D) -> Self {
        Self {
            source: Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 },
            tint: Color::WHITE,
            position: Vector2 { x: 0.0, y: 0.0 },
            size: Vector2 { x: texture.width() as f32, y: texture.height() as f32 },
            origin: Vector2 { x: 0.0, y: 0.0 },
            rotation: Rad(0.0),
            texture,
        }
    }

    pub fn with_source(mut self, source: Rect<f32>) -> Self {
        self.source = source;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    pub fn with_origin(mut self, origin: Vector2<f32>) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_position(mut self, position: Vector2<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn with_size(mut self, size: Vector2<f32>) -> Self {
        self.size = size;
        self
    }

    pub fn build(self, ctx: &mut Context) -> Result<Sprite, BackendError> {
        Sprite::new(ctx, self.texture, self.source, self.position, self.size, self.origin, self.rotation, self.tint)
    }
}

pub struct Sprite {
    pipeline: BasicPipeline2D,
    /// Vertex buffer object
    #[allow(unused)]
    vbo: Buffer,
    vao: VertexArray,

    // what to draw?
    texture: Texture2D,
    source: Rect<f32>,
    tint: Color,

    /// The position of the origin in global coordinates.
    position: Vector2<f32>,
    /// The size of the Sprite in global coordinates.
    size: Vector2<f32>,
    /// The origin for position and rotation in local coordinates where `(0, 0)` is the top-left
    /// and `(1, 1)` is the bottom-right corner of the sprite.
    origin: Vector2<f32>,
    /// Rotation angle around the origin.
    rotation: Rad<f32>,
    // TODO: sampler options
}

impl Sprite {
    pub fn build(texture: Texture2D) -> SpriteBuilder {
        SpriteBuilder::new(texture)
    }

    pub fn new(
        ctx: &mut Context,
        texture: Texture2D,
        source: Rect<f32>,
        position: Vector2<f32>,
        size: Vector2<f32>,
        origin: Vector2<f32>,
        rotation: Rad<f32>,
        tint: Color,
    ) -> Result<Sprite, BackendError> {
        let mut pipeline = BasicPipeline2D::new(ctx)?;
        // Render sprites with alpha blending by default
        pipeline.set_blend_mode(Some(BlendMode::alpha()));
        // Always use the first texture unit
        pipeline.set_texture(Some(0));

        let vbo = Buffer::new()?;
        let vao = VertexArray::new()?;

        VertexArray::bind(&vao)?;
        Buffer::bind(gl::ARRAY_BUFFER, &vbo)?;
        unsafe {
            let top_left = source.position();
            let bottom_right = top_left + source.size();

            let vertices = &[
                BasicVertex2D {
                    position: [0.0, 0.0],
                    tex_coord: [top_left.x, top_left.y],
                    color: tint.into(),
                },
                BasicVertex2D {
                    position: [1.0, 0.0],
                    tex_coord: [bottom_right.x, top_left.y],
                    color: tint.into(),
                },
                BasicVertex2D {
                    position: [0.0, 1.0],
                    tex_coord: [top_left.x, bottom_right.y],
                    color: tint.into(),
                },
                BasicVertex2D {
                    position: [1.0, 1.0],
                    tex_coord: [bottom_right.x, bottom_right.y],
                    color: tint.into(),
                },
            ];
            Buffer::data(gl::ARRAY_BUFFER, vertices, gl::STATIC_DRAW)?;

            for attrib in BasicVertex2D::attributes() {
                attrib.set_pointer()?;
                attrib.enable()?;
            }
        }
        // NOTE: The ARRAY_BUFFER has been remembered by the VAO as part of the
        // call to glVertexAttribPointer, so we can unbind it again.
        Buffer::unbind(gl::ARRAY_BUFFER)?;
        VertexArray::unbind()?;

        Ok(Sprite {
            pipeline,
            vbo,
            vao,
            texture,
            source,
            position,
            size,
            tint,
            origin,
            rotation,
        })
    }

    /// The underlying texture of this sprite.
    pub fn texture(&self) -> &Texture2D {
        &self.texture
    }

    /// The tint color of this sprite that is multiplied with the texture color.
    ///
    /// # Note
    ///
    /// The tint is baked into the vertex buffer and cannot be changed after creating the sprite.
    pub fn tint(&self) -> Color {
        self.tint
    }

    /// The part of the texture that is used for rendering this sprite,
    /// given in normalized UV coordinates.
    ///
    /// # Note
    ///
    /// The source rectangle is baked into the vertex buffer and cannot be changed after creating the sprite.
    pub fn source(&self) -> Rect<f32> {
        self.source
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    // TODO: split position rect into position + size?
    pub fn set_position(&mut self, pos: Vector2<f32>) {
        self.position = pos
    }

    pub fn rotation(&self) -> Rad<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, angle: Rad<f32>) {
        self.rotation = angle
    }

    pub fn origin(&self) -> Vector2<f32> {
        self.origin
    }

    pub fn set_origin(&mut self, origin: Vector2<f32>) {
        self.origin = origin
    }

    pub fn local_transform(&self) -> Matrix3<f32> {
        let origin = super::transform::translate(-self.origin);
        let rotate = super::transform::rotate(self.rotation);
        let scale = super::transform::scale(self.size);
        let position = super::transform::translate(self.position);
        position * scale * rotate * origin
    }
}

impl super::Drawable for Sprite {
    fn draw(&mut self, ctx: &mut Context, mut state: RenderState) -> GameResult<()> {
        state.transform = state.transform * self.local_transform();
        self.pipeline.set_transform(state.transform);
        self.pipeline.apply(ctx)?;
        self.vao.bind()?;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            Texture::bind(gl::TEXTURE_2D, self.texture.raw())?;
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            Texture::unbind(gl::TEXTURE_2D)?;
        }
        VertexArray::unbind()?;
        Program::unbind()?;
        Ok(())
    }
}
