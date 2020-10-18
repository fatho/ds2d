//! A generic 2D triangle mesh.

use super::{
    context::{BackendError, Buffer, Program, VertexArray},
    primitives::BasicPipeline2D,
    primitives::BasicVertex2D,
    primitives::Pipeline,
    primitives::VertexData,
    RenderState,
};
use crate::{Context, GameResult};

pub struct Mesh {
    pipeline: BasicPipeline2D,
    #[allow(unused)]
    /// Vertex buffer object
    vbo: Buffer,
    #[allow(unused)]
    /// Elements buffer object
    ebo: Buffer,
    vao: VertexArray,

    num_elements: i32,
}

impl Mesh {
    pub fn new(
        ctx: &mut Context,
        vertices: &[BasicVertex2D],
        indices: &[u32],
    ) -> Result<Mesh, BackendError> {
        if vertices.len() > std::u32::MAX as usize {
            return Err(BackendError::TooLarge);
        }
        if indices.len() > std::i32::MAX as usize {
            return Err(BackendError::TooLarge);
        }

        let pipeline = BasicPipeline2D::new(ctx)?;
        let vbo = Buffer::new()?;
        let ebo = Buffer::new()?;
        let vao = VertexArray::new()?;

        VertexArray::bind(&vao)?;
        Buffer::bind(gl::ARRAY_BUFFER, &vbo)?;
        // NOTE: the ELEMENT_ARRAY_BUFFER binding point is a property of the VAO
        // Unbinding the VAO unbinds the ELEMENT_ARRAY_BUFFER, and without a bound VAO,
        // the ELEMENT_ARRAY_BUFFER cannot be bound.
        Buffer::bind(gl::ELEMENT_ARRAY_BUFFER, &ebo)?;
        unsafe {
            // Safe because cgmath::Vector2 is repr(C)
            Buffer::data(gl::ARRAY_BUFFER, vertices, gl::STATIC_DRAW)?;
            Buffer::data(gl::ELEMENT_ARRAY_BUFFER, indices, gl::STATIC_DRAW)?;
            // Safe because it corresponds to the layout of our buffer above
            for attrib in BasicVertex2D::attributes() {
                attrib.set_pointer()?;
                attrib.enable()?;
            }
        }
        // NOTE: The ARRAY_BUFFER has been remembered by the VAO as part of the
        // call to glVertexAttribPointer, so we can unbind it again.
        Buffer::unbind(gl::ARRAY_BUFFER)?;
        VertexArray::unbind()?;

        Ok(Mesh {
            pipeline,
            vbo,
            ebo,
            vao,
            num_elements: indices.len() as i32,
        })
    }
}

impl super::Drawable for Mesh {
    fn draw(&mut self, ctx: &mut Context, state: RenderState) -> GameResult<()> {
        self.pipeline.set_transform(state.transform);
        self.pipeline.apply(ctx)?;
        self.vao.bind()?;
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.num_elements, gl::UNSIGNED_INT, 0 as _);
        }
        VertexArray::unbind()?;
        Program::unbind()?;
        Ok(())
    }
}
