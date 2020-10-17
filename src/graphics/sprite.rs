//! A generic 2D triangle mesh.

use cgmath::Vector2;

use super::{
    context::{BackendError, Buffer, Program, Texture, VertexArray},
    Color, Rect, Texture2D,
};
use crate::{Context, GameResult};

pub struct Sprite {
    program: Program,
    /// Vertex buffer object
    vbo: Buffer,
    vao: VertexArray,

    position: Vector2<f32>,
    texture: Texture2D,
    tint: Color,
    // TODO: source rectangle
    // TODO: sampler options
    // TODO: blend options
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SpriteVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    /// TODO: can we pass color more efficiently as byte vector?
    color: [f32; 4],
}

impl SpriteVertex {
    const SIZE: i32 = std::mem::size_of::<SpriteVertex>() as i32;
}

impl Sprite {
    const VERTEX_SHADER: &'static str = r"#version 330 core
    layout (location = 0) in vec2 position;
    layout (location = 1) in vec2 tex_coord_vert;
    layout (location = 2) in vec4 color_vert;

    out vec2 tex_coord_frag;
    out vec4 color_frag;

    uniform mat3 model_view_projection = mat3(1.0);

    void main()
    {
        vec3 transformed = model_view_projection * vec3(position, 1.0);
        gl_Position = vec4(transformed.xy, 0.0, 1.0);

        tex_coord_frag = tex_coord_vert;
        color_frag = color_vert;
    }";

    const FRAGMENT_SHADER: &'static str = r"#version 330 core
    in vec2 tex_coord_frag;
    in vec4 color_frag;

    out vec4 FragColor;

    uniform sampler2D texture0;

    void main()
    {
        vec4 tex_color = texture(texture0, tex_coord_frag);
        FragColor = tex_color * color_frag;
    }";

    pub fn new(
        _ctx: &mut Context,
        texture: Texture2D,
        position: Vector2<f32>,
        tint: Color,
    ) -> Result<Sprite, BackendError> {
        // TODO: cache/share programs across instances
        let program = Program::from_source(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)?;
        let vbo = Buffer::new()?;
        let vao = VertexArray::new()?;

        VertexArray::bind(&vao)?;
        Buffer::bind(gl::ARRAY_BUFFER, &vbo)?;
        unsafe {
            Buffer::alloc(gl::ARRAY_BUFFER, 4 * SpriteVertex::SIZE, gl::DYNAMIC_DRAW)?;

            // Position
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, SpriteVertex::SIZE, 0 as _);
            // Texture coordinate
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, SpriteVertex::SIZE, 8 as _);
            // Color
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, SpriteVertex::SIZE, 16 as _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
        }
        // NOTE: The ARRAY_BUFFER has been remembered by the VAO as part of the
        // call to glVertexAttribPointer, so we can unbind it again.
        Buffer::unbind(gl::ARRAY_BUFFER)?;
        VertexArray::unbind()?;

        Ok(Sprite {
            program,
            vbo,
            vao,
            texture,
            position,
            tint,
        })
    }

    fn update_vertex_data(&mut self) -> Result<(), BackendError> {
        Buffer::bind(gl::ARRAY_BUFFER, &self.vbo)?;

        let right = Vector2::new(self.texture.width() as f32, 0.0);
        let down = Vector2::new(0.0, self.texture.height() as f32);
        let color = self.tint.into();

        let vertices = &[
            SpriteVertex {
                position: self.position.into(),
                tex_coord: [0.0, 0.0],
                color,
            },
            SpriteVertex {
                position: (self.position + right).into(),
                tex_coord: [1.0, 0.0],
                color,
            },
            SpriteVertex {
                position: (self.position + down).into(),
                tex_coord: [0.0, 1.0],
                color,
            },
            SpriteVertex {
                position: (self.position + right + down).into(),
                tex_coord: [1.0, 1.0],
                color,
            },
        ];
        unsafe {
            Buffer::data(gl::ARRAY_BUFFER, vertices, gl::DYNAMIC_DRAW)?;
        }
        Buffer::unbind(gl::ARRAY_BUFFER)
    }
}

impl super::Drawable for Sprite {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // TODO: only update vertex data when something was changed
        self.update_vertex_data()?;
        self.program.bind()?;
        self.program
            .set_uniform_mat3("model_view_projection", &ctx.graphics.pixel_projection)?;
        self.program.set_uniform_sampler2d("texture0", 0)?;
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
