//! Implements batched rendering of sprites.

use cgmath::{Matrix3, Rad, Vector2};

use super::{
    context::{BackendError, Buffer, Program, Texture, VertexArray},
    primitives::BasicPipeline2D,
    primitives::BasicVertex2D,
    primitives::{Pipeline, VertexData},
    Color, Drawable, GraphicsError, Rect, RenderState, Texture2D,
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

    pub fn draw_triangle(&mut self, corners: [Vector2<f32>; 3], color: Color) {
        for corner in &corners {
            self.vertices.push(BasicVertex2D {
                position: *corner.as_ref(),
                color: color.into(),
                // no texture
                tex_coord: [0.0; 2],
            });
        }

        let key = BatchKey { texture: None };
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

/// Determines whether two primitives can be drawn in the same batch.
#[derive(Debug, Eq, PartialEq)]
struct BatchKey {
    texture: Option<Texture2D>,
    // TODO: blend mode
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
