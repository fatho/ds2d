//! A generic 2D triangle mesh.

use cgmath::Vector2;

use super::{RenderState, context::{BackendError, Buffer, Program, VertexArray}};
use crate::{Context, GameResult};

pub struct Mesh {
    program: Program,
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
    const VERTEX_SHADER: &'static str = r"#version 330 core
    layout (location = 0) in vec2 position;

    uniform mat3 model_view_projection = mat3(1.0);

    void main()
    {
        vec3 transformed = model_view_projection * vec3(position, 1.0);
        gl_Position = vec4(transformed.xy, 0.0, 1.0);
    }";

    const FRAGMENT_SHADER: &'static str = r"#version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }";

    pub fn new(
        _ctx: &mut Context,
        vertices: &[Vector2<f32>],
        indices: &[u32],
    ) -> Result<Mesh, BackendError> {
        if vertices.len() > std::u32::MAX as usize {
            return Err(BackendError::TooLarge);
        }
        if indices.len() > std::i32::MAX as usize {
            return Err(BackendError::TooLarge);
        }

        // TODO: cache/share programs across instances
        let program = Program::from_source(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)?;
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
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<cgmath::Vector2<f32>>() as i32,
                0 as _,
            );
            gl::EnableVertexAttribArray(0);
        }
        // NOTE: The ARRAY_BUFFER has been remembered by the VAO as part of the
        // call to glVertexAttribPointer, so we can unbind it again.
        Buffer::unbind(gl::ARRAY_BUFFER)?;
        VertexArray::unbind()?;

        Ok(Mesh {
            program,
            vbo,
            ebo,
            vao,
            num_elements: indices.len() as i32,
        })
    }
}

impl super::Drawable for Mesh {
    fn draw(&mut self, ctx: &mut Context, state: RenderState) -> GameResult<()> {
        self.program.bind()?;
        super::set_blend_mode(ctx, state.blend)?;
        self.program.set_uniform("model_view_projection", &state.transform)?;
        self.vao.bind()?;
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.num_elements, gl::UNSIGNED_INT, 0 as _);
        }
        VertexArray::unbind()?;
        Program::unbind()?;
        Ok(())
    }
}
