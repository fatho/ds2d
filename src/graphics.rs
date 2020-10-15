use crate::context::graphics::{Buffer, BufferTarget, BufferUsage, Program, VertexArray};

use super::{Context, GameResult};
use glutin::dpi::PhysicalSize;

mod color;
pub use color::Color;

pub mod primitives;


pub struct Mesh {
    program: Program,
    #[allow(unused)]
    vertices: Buffer,
    vao: VertexArray,
    num_vertices: i32,
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


    pub fn new(_ctx: &mut Context, points: &[cgmath::Vector2<f32>]) -> GameResult<Mesh> {
        if points.len() > std::i32::MAX as usize {
            return Err(crate::GameError::Graphics("Too many points".into()))
        }

        // TODO: cache/share programs across instances
        let program = Program::from_source(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)?;
        let vertices = Buffer::new()?;
        let vao = VertexArray::new()?;

        Buffer::bind(BufferTarget::Vertex, &vertices)?;
        VertexArray::bind(&vao)?;
        unsafe {
            // Safe because cgmath::Vector2 is repr(C)
            Buffer::data(BufferTarget::Vertex, points, BufferUsage::StaticDraw)?;
            // Safe because it corresponds to the layout of our buffer above
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<cgmath::Vector2<f32>>() as i32, 0 as _);
            gl::EnableVertexAttribArray(0);
        }
        Buffer::unbind(BufferTarget::Vertex)?;
        VertexArray::unbind()?;

        Ok(Mesh {
            program,
            vertices,
            vao,
            num_vertices: points.len() as i32,
        })
    }
}

impl Drawable for Mesh {
    fn draw(&self, _ctx: &mut Context) -> GameResult<()> {
        self.program.bind()?;
        self.vao.bind()?;
        unsafe { gl::DrawArrays(gl::TRIANGLES, 0, self.num_vertices as i32); }
        VertexArray::unbind()?;
        Ok(())
    }
}

pub trait Drawable {
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
}

pub fn draw<T: Drawable>(ctx: &mut Context, drawable: &T) -> GameResult<()> {
    drawable.draw(ctx)
}

pub fn screen_size(ctx: &mut Context) -> PhysicalSize<u32> {
    ctx.graphics.screen_size
}

pub fn scale_factor(ctx: &mut Context) -> f64 {
    ctx.graphics.scale_factor
}

pub fn clear(_ctx: &mut Context, color: Color) {
    unsafe {
        gl::ClearColor(color.r, color.g, color.b, color.a);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}
