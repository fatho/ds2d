use std::rc::Rc;

use crate::context::graphics::{Buffer, BufferTarget, BufferUsage, Program, VertexArray};

use super::{Context, GameResult};
use glutin::dpi::PhysicalSize;

mod color;
pub use color::Color;

pub mod primitives;


pub struct Mesh {
    program: Rc<Program>,
    vertices: Rc<Buffer>,
    vao: Rc<VertexArray>,
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


    pub fn new(ctx: &mut Context, points: &[cgmath::Vector2<f64>]) -> GameResult<Mesh> {
        // TODO: cache/share programs across instances
        let program = Rc::new(Program::from_source(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)?);
        let vertices = Rc::new(Buffer::new()?);
        let vao = Rc::new(VertexArray::new()?);

        ctx.graphics.state.with_buffer(BufferTarget::Vertex, vertices.clone(), |state| {
            // Safe because cgmath::Vector2 is repr(C)
            unsafe { Buffer::data(BufferTarget::Vertex, points, BufferUsage::StaticDraw)?; }

            state.with_vao(vao.clone(), |_| {
                // TODO: set up the VAO
                Ok(())
            })
        })?;

        Ok(Mesh {
            program,
            vertices,
            vao,
        })
    }
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
