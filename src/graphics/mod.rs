use super::{Context, GameResult};
use cgmath::Vector2;
use glutin::dpi::PhysicalSize;



mod color;
mod rect;
mod camera;

pub(crate) mod context;

pub mod primitives;

pub use color::Color;
pub use rect::Rect;
pub use camera::Camera2d;

use context::{Buffer, Program, VertexArray};

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
    layout (location = 0) in vec2 pos;

    void main()
    {
        gl_Position = vec4(pos.x, pos.y,0.0, 1.0);
    }";

    const FRAGMENT_SHADER: &'static str = r"#version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }";


    pub fn new(_ctx: &mut Context, vertices: &[Vector2<f32>], indices: &[u32]) -> GameResult<Mesh> {
        if vertices.len() > std::u32::MAX as usize {
            return Err(crate::GameError::Graphics("Too many vertices".into()))
        }
        if indices.len() > std::i32::MAX as usize {
            return Err(crate::GameError::Graphics("Too many indices".into()))
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
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<cgmath::Vector2<f32>>() as i32, 0 as _);
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

impl Drawable for Mesh {
    fn draw(&self, _ctx: &mut Context) -> GameResult<()> {
        self.program.bind()?;
        self.vao.bind()?;
        unsafe { gl::DrawElements(gl::TRIANGLES, self.num_elements, gl::UNSIGNED_INT, 0 as _); }
        VertexArray::unbind()?;
        Program::unbind()?;
        Ok(())
    }
}


/// Implemented by every "well-behaved" entity that can be drawn, meaning
/// - it respects the current coordinate system (TODO)
/// - it can additionally be transformed as part of the draw call (TODO)
pub trait Drawable {
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
}


pub fn draw<T: Drawable>(ctx: &mut Context, drawable: &T) -> GameResult<()> {
    drawable.draw(ctx)
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
