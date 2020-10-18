use cgmath::Matrix3;
use gl::types::{GLboolean, GLenum, GLint, GLuint};

use crate::{CheckGl, Context};

use super::{BlendMode, BackendError, context::Program, context::UniformValue, context::VertexAttrib};

// pub struct Decomposed {

// }

#[derive(Debug, Copy, Clone)]
pub struct RenderState {
    pub transform: Matrix3<f32>,
    pub blend: Option<BlendMode>,
}


pub trait ShaderProgram {
    type Vertex: VertexData;

    fn apply(&mut self, ctx: &mut Context) -> Result<(), BackendError>;
}

pub trait VertexData: Copy {
    fn attributes() -> &'static [VertexAttrib];
}

/// A basic shader for 2D rendering that can be used in a wide variety of drawables.
pub struct BasicShader2D {
    program: Program,
    transform: ShaderParameter<Matrix3<f32>>,
    texture: ShaderParameter<i32>,
}

impl BasicShader2D {
    pub fn new(_ctx: &mut Context) -> Result<Self, BackendError> {
        // TODO: we should cache the individual shaders and only relink them
        // We probably don't want to cache the program object itself, because
        // that would mean that uniform values are also shared across instances.
        let program = Program::from_source(Self::VERTEX_SHADER_330_CORE, Self::FRAGMENT_SHADER_330_CORE)?;

        Ok(Self {
            program,
            // the identity matrix is the default matrix in the shader program.
            transform: ShaderParameter::new("Transform", cgmath::SquareMatrix::identity(), false),
            texture: ShaderParameter::new("Texture0", 0, true),
        })
    }

    /// The transform matrix used when the shader is next applied.
    pub fn param_transform(&self) -> &ShaderParameter<Matrix3<f32>> {
        &self.transform
    }

    /// The transform matrix used when the shader is next applied.
    pub fn param_transform_mut(&mut self) -> &mut ShaderParameter<Matrix3<f32>> {
        &mut self.transform
    }

    /// The texture that is used when the shader is next applied.
    pub fn param_texture(&self) -> &ShaderParameter<i32> {
        &self.texture
    }

    /// The texture that is used when the shader is next applied.
    pub fn param_texture_mut(&mut self) -> &mut ShaderParameter<i32> {
        &mut self.texture
    }

    // TODO: support multiple versions of GLSL
    const VERTEX_SHADER_330_CORE: &'static str = r"#version 330 core
    layout (location = 0) in vec2 Position;
    layout (location = 1) in vec2 TexCoord;
    layout (location = 2) in vec4 Color;

    out vec2 Vert_Frag_TexCoord;
    out vec4 Vert_Frag_Color;

    uniform mat3 Transform = mat3(1.0);

    void main()
    {
        vec3 transformed = Transform * vec3(Position, 1.0);
        gl_Position = vec4(transformed.xy, 0.0, 1.0);

        Vert_Frag_TexCoord = TexCoord;
        Vert_Frag_Color = Color;
    }";

    const FRAGMENT_SHADER_330_CORE: &'static str = r"#version 330 core
    in vec2 Vert_Frag_TexCoord;
    in vec4 Vert_Frag_Color;

    out vec4 FragColor;

    uniform sampler2D Texture0;

    void main()
    {
        vec4 tex_color = texture(Texture0, Vert_Frag_TexCoord);
        FragColor = tex_color * Vert_Frag_Color;
    }";
}

impl ShaderProgram for BasicShader2D {
    type Vertex = BasicVertex2D;

    fn apply(&mut self, _ctx: &mut Context) -> Result<(), BackendError> {
        // TODO: keep track of currently used program?
        Program::bind(&self.program)?;
        self.transform.set_uniform(&self.program)?;
        Ok(())
    }
}

/// The vertex type consumed by the `BasicShader2D`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BasicVertex2D {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
}

impl VertexData for BasicVertex2D {
    fn attributes() -> &'static [VertexAttrib] {
        const SIZE: i32 = std::mem::size_of::<BasicVertex2D>() as i32;
        &[
            VertexAttrib { index: 0, size: 2, gl_type: gl::FLOAT, normalized: gl::FALSE, stride: SIZE, offset: 0 },
            VertexAttrib { index: 1, size: 2, gl_type: gl::FLOAT, normalized: gl::FALSE, stride: SIZE, offset: 8 },
            VertexAttrib { index: 2, size: 4, gl_type: gl::FLOAT, normalized: gl::FALSE, stride: SIZE, offset: 16 },
        ]
    }
}


pub struct ShaderParameter<T> {
    name: String,
    value: T,
    dirty: bool,
}

impl<T: UniformValue + PartialEq> ShaderParameter<T> {
    fn new<S: Into<String>>(name: S, initial: T, dirty: bool) -> Self {
        Self {
            name: name.into(),
            value: initial,
            dirty
        }
    }

    pub fn set(&mut self, new_value: T) {
        if self.value != new_value {
            self.dirty = true;
            self.value = new_value;
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    fn set_uniform(&mut self, program: &Program) -> Result<(), BackendError> {
        if self.dirty {
            program.set_uniform(&self.name, &self.value)?;
            self.dirty = false;
        }
        Ok(())
    }
}
