//! A generic 2D triangle mesh.

use cgmath::{Matrix3, Vector2};

use super::{Color, Rect, RenderState, Texture2D, context::{BackendError, Buffer, Program, Texture, VertexArray}};
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
}

impl SpriteVertex {
    const SIZE: i32 = std::mem::size_of::<SpriteVertex>() as i32;
}

impl Sprite {
    const VERTEX_SHADER: &'static str = r"#version 330 core
    layout (location = 0) in vec2 position;
    layout (location = 1) in vec2 tex_coord_vert;

    out vec2 tex_coord_frag;
    out vec4 color_frag;

    uniform mat3 model_view_projection = mat3(1.0);

    void main()
    {
        vec3 transformed = model_view_projection * vec3(position, 1.0);
        gl_Position = vec4(transformed.xy, 0.0, 1.0);

        tex_coord_frag = tex_coord_vert;
    }";

    const FRAGMENT_SHADER: &'static str = r"#version 330 core
    in vec2 tex_coord_frag;

    out vec4 FragColor;

    uniform sampler2D texture0;
    uniform vec4 tint;

    void main()
    {
        vec4 tex_color = texture(texture0, tex_coord_frag);
        FragColor = tex_color * tint;
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
            let right = Vector2::new(texture.width() as f32, 0.0);
            let down = Vector2::new(0.0, texture.height() as f32);

            let vertices = &[
                SpriteVertex {
                    position: [0.0, 0.0],
                    tex_coord: [0.0, 0.0],
                },
                SpriteVertex {
                    position: right.into(),
                    tex_coord: [1.0, 0.0],
                },
                SpriteVertex {
                    position: down.into(),
                    tex_coord: [0.0, 1.0],
                },
                SpriteVertex {
                    position: (right + down).into(),
                    tex_coord: [1.0, 1.0],
                },
            ];
            Buffer::data(gl::ARRAY_BUFFER, vertices, gl::STATIC_DRAW)?;

            // Position
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, SpriteVertex::SIZE, 0 as _);
            // Texture coordinate
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, SpriteVertex::SIZE, 8 as _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
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

    pub fn set_position(&mut self, new_pos: Vector2<f32>) {
        self.position = new_pos;
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn local_transform(&self) -> Matrix3<f32> {
        super::transform::translate(self.position)
    }
}

impl super::Drawable for Sprite {
    fn draw(&mut self, ctx: &mut Context, mut state: RenderState) -> GameResult<()> {
        state.transform = state.transform * self.local_transform();

        self.program.bind()?;
        super::set_blend_mode(ctx, state.blend)?;
        self.program
            .set_uniform_mat3("model_view_projection", &state.transform)?;
        self.program.set_uniform_sampler2d("texture0", 0)?;
        self.program.set_uniform_vec4("tint", self.tint.into())?;
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
