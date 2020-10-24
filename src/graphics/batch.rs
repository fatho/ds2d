//! Implements batched rendering of sprites.

use cgmath::{ElementWise, Rad, Vector2};

use super::{
    context::{Buffer, Program, Texture, VertexArray},
    primitives::BasicPipeline2D,
    primitives::BasicVertex2D,
    primitives::{Pipeline, VertexData},
    BlendMode, Color, Drawable, GraphicsError, Rect, RenderState, Texture2D,
};
use crate::{CheckGl, Context, GameResult};

pub struct BatchRender {
    pipeline: BasicPipeline2D,
    vbo: Buffer,
    // TODO: try out whether using an index buffer is an improvement
    //ebo: Buffer,
    vao: VertexArray,

    batches: Vec<Batch>,
    vertices: Vec<BasicVertex2D>,
}

impl BatchRender {
    pub fn new(ctx: &mut Context) -> Result<Self, GraphicsError> {
        let pipeline = BasicPipeline2D::new(ctx)?;
        let vbo = Buffer::new()?;
        //let ebo = Buffer::new()?;
        let vao = VertexArray::new()?;

        VertexArray::bind(&vao)?;
        Buffer::bind(gl::ARRAY_BUFFER, &vbo)?;
        unsafe {
            for attrib in BasicVertex2D::attributes() {
                attrib.set_pointer()?;
                attrib.enable()?;
            }
        }
        // NOTE: The ARRAY_BUFFER has been remembered by the VAO as part of the
        // call to glVertexAttribPointer, so we can unbind it again.
        Buffer::unbind(gl::ARRAY_BUFFER)?;
        VertexArray::unbind()?;

        // for attrib in BasicVertex2D::attributes() {
        //     attrib.set_pointer()?;
        //     attrib.enable()?;
        // }

        Ok(Self {
            pipeline,
            vbo,
            //ebo,
            vao,

            batches: Vec::new(),
            vertices: Vec::new(),
        })
    }

    pub fn draw_quad<Q: Into<Quad>>(&mut self, quad: Q) {
        let quad: Quad = quad.into();
        // TODO: use index buffer?
        self.vertices.push(quad.vertices[0]);
        self.vertices.push(quad.vertices[2]);
        self.vertices.push(quad.vertices[1]);
        self.vertices.push(quad.vertices[0]);
        self.vertices.push(quad.vertices[3]);
        self.vertices.push(quad.vertices[2]);
        self.update_batch(quad.key, 6);
    }

    pub fn draw_triangle(&mut self, corners: [Vector2<f32>; 3], color: Color) {
        for corner in &corners {
            self.vertices.push(BasicVertex2D {
                position: *corner.as_ref(),
                color: color.into(),
                // no texture
                tex_coord: [0.0; 2],
            });
        }

        let key = BatchKey {
            texture: None,
            blend: None,
        };
        self.update_batch(key, 3);
    }

    /// Ensures that the most last batch in the `batches` vector matches
    /// the given key, creating a new batch if necessary.
    fn update_batch(&mut self, key: BatchKey, num_vertices: usize) {
        let max_len = std::i32::MAX as usize;
        let (new_batch, next_vertex) = self.batches.last().map_or((true, 0), |batch| {
            let same_key = batch.key == key;
            let has_room = batch.len() + num_vertices <= max_len;
            (!same_key || !has_room, batch.end)
        });
        if new_batch {
            self.batches.push(Batch {
                key,
                start: next_vertex,
                end: next_vertex,
            });
        }
        let batch = self.batches.last_mut().unwrap();
        batch.end += num_vertices;
    }
}

pub struct TextureView2D {
    pub texture: Texture2D,
    pub source: Rect<f32>,
}

impl TextureView2D {
    pub fn new(texture: Texture2D, source: Rect<f32>) -> Self {
        Self { texture, source }
    }

    pub fn size(&self) -> Vector2<f32> {
        let uv_size = self.source.size();
        Vector2 {
            x: self.texture.width() as f32 * uv_size.x,
            y: self.texture.height() as f32 * uv_size.y,
        }
    }
}

impl From<Texture2D> for TextureView2D {
    fn from(texture: Texture2D) -> Self {
        Self {
            texture,
            source: Rect::unit_square(),
        }
    }
}

pub struct Quad {
    key: BatchKey,
    vertices: [BasicVertex2D; 4],
}

impl Quad {
    pub fn textured<T: Into<TextureView2D>>(texture: T) -> QuadBuilder {
        let texture = texture.into();
        QuadBuilder {
            tint: Color::WHITE,
            position: Vector2 { x: 0.0, y: 0.0 },
            size: texture.size(),
            origin: Vector2 { x: 0.0, y: 0.0 },
            rotation: Rad(0.0),
            texture: Some(texture),
        }
    }

    pub fn untextured(size: Vector2<f32>) -> QuadBuilder {
        QuadBuilder {
            tint: Color::WHITE,
            position: Vector2 { x: 0.0, y: 0.0 },
            size,
            origin: Vector2 { x: 0.0, y: 0.0 },
            rotation: Rad(0.0),
            texture: None,
        }
    }
}

pub struct QuadBuilder {
    /// The texture to draw.
    texture: Option<TextureView2D>,
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

impl QuadBuilder {
    /// Sets the position, but also sets the origin to the center of the quad.
    pub fn centered_at(self, position: Vector2<f32>) -> Self {
        self.with_position(position)
            .with_origin(Vector2::new(0.5, 0.5))
    }

    /// Multiply the current size of the builder by this amount.
    pub fn scale(mut self, scale: f32) -> Self {
        self.size *= scale;
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

    pub fn with_rotation(mut self, rotation: Rad<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn build(self) -> Quad {
        let uv = self
            .texture
            .as_ref()
            .map_or(Rect::unit_square(), |tex| tex.source);
        // Unrotated rect of the right shape, with the origin at (0,0) in global coordinates
        let unrotated = Rect {
            top_left: (Vector2::new(0.0, 0.0) - self.origin).mul_element_wise(self.size),
            bottom_right: (Vector2::new(1.0, 1.0) - self.origin).mul_element_wise(self.size),
        };
        let (r_sin, r_cos) = self.rotation.0.sin_cos();
        let rotation = cgmath::Matrix2::new(r_cos, r_sin, -r_sin, r_cos);

        let mut corners = unrotated.corners();
        for corner in corners.iter_mut() {
            *corner = self.position + rotation * *corner
        }

        let vertices = [
            BasicVertex2D {
                position: corners[0].into(),
                tex_coord: [uv.top_left.x, uv.top_left.y],
                color: self.tint.into(),
            },
            BasicVertex2D {
                position: corners[1].into(),
                tex_coord: [uv.bottom_right.x, uv.top_left.y],
                color: self.tint.into(),
            },
            BasicVertex2D {
                position: corners[2].into(),
                tex_coord: [uv.bottom_right.x, uv.bottom_right.y],
                color: self.tint.into(),
            },
            BasicVertex2D {
                position: corners[3].into(),
                tex_coord: [uv.top_left.x, uv.bottom_right.y],
                color: self.tint.into(),
            },
        ];
        Quad {
            key: BatchKey {
                texture: self.texture.map(|view| view.texture),
                blend: Some(BlendMode::alpha()),
            },
            vertices,
        }
    }
}

impl From<QuadBuilder> for Quad {
    fn from(builder: QuadBuilder) -> Self {
        builder.build()
    }
}

/// Determines whether two primitives can be drawn in the same batch.
#[derive(Debug, Eq, PartialEq)]
struct BatchKey {
    texture: Option<Texture2D>,
    blend: Option<BlendMode>,
}

struct Batch {
    key: BatchKey,
    /// Start of this batch in the vertex buffer
    start: usize,
    /// Number of vertices in this batch
    end: usize,
}

impl Batch {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl Drawable for BatchRender {
    fn draw(&mut self, ctx: &mut Context, state: RenderState) -> GameResult<()> {
        let mut last_tex_id = 0;
        unsafe {
            // Ensure we're using the first texture unit
            CheckGl!(gl::ActiveTexture(gl::TEXTURE0))?;
        }
        self.pipeline.set_transform(state.transform);

        self.vao.bind()?;
        Buffer::bind(gl::ARRAY_BUFFER, &self.vbo)?;

        for batch in self.batches.drain(..) {
            if let Some(ref tex) = batch.key.texture {
                let cur_tex_id = tex.raw().id();
                if last_tex_id != cur_tex_id {
                    last_tex_id = cur_tex_id;
                    Texture::bind(gl::TEXTURE_2D, tex.raw())?;
                }
                // 0 refers to the texture unit, not the texture id
                self.pipeline.set_texture(Some(0));
            } else {
                self.pipeline.set_texture(None);
            }
            self.pipeline.set_blend_mode(batch.key.blend);
            self.pipeline.apply(ctx)?;
            // TODO: better buffer handling
            unsafe {
                Buffer::data(
                    gl::ARRAY_BUFFER,
                    &self.vertices[batch.start..batch.end],
                    gl::STATIC_DRAW,
                )?;
                // Batches are limited so that the len always fits into i32
                gl::DrawArrays(gl::TRIANGLES, 0, batch.len() as i32);
            }
        }
        VertexArray::unbind()?;
        Program::unbind()?;
        Texture::unbind(gl::TEXTURE_2D)?;

        self.vertices.clear();
        Ok(())
    }
}
