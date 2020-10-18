use std::fmt::Debug;

use cgmath::Matrix3;

use crate::{Context};

use super::{BackendError, BlendMode, Color, context::Program, context::UniformValue, context::VertexAttrib};

// pub struct Decomposed {

// }

#[derive(Debug, Copy, Clone)]
pub struct RenderState {
    pub transform: Matrix3<f32>,
}


pub trait Pipeline {
    type Vertex: VertexData;

    fn apply(&mut self, ctx: &mut Context) -> Result<(), BackendError>;
}

pub trait VertexData: Copy {
    fn attributes() -> &'static [VertexAttrib];
}

/// A basic pipeline for 2D rendering that can be used in a wide variety of drawables.
pub struct BasicPipeline2D {
    program: Program,
    transform: ShaderParameter<Matrix3<f32>>,
    texture: ShaderParameter<i32>,
    use_texture: ShaderParameter<bool>,
    blend_mode: Option<BlendMode>,
}

impl BasicPipeline2D {
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
            use_texture: ShaderParameter::new("UseTexture0", false, false),
            blend_mode: None,
        })
    }

    /// The blend mode used when the pipeline is next applied.
    pub fn get_blend_mode(&self) -> &Option<BlendMode> {
        &self.blend_mode
    }

    /// The blend mode used when the pipeline is next applied.
    pub fn set_blend_mode(&mut self, new: Option<BlendMode>) {
        self.blend_mode = new
    }

    /// The transform matrix used when the pipeline is next applied.
    pub fn get_transform(&self) -> &Matrix3<f32> {
        &self.transform.value
    }

    /// The transform matrix used when the pipeline is next applied.
    pub fn set_transform(&mut self, new: Matrix3<f32>) {
        self.transform.set(new)
    }

    /// The texture used when the pipeline is next applied.
    pub fn get_texture(&self) -> Option<i32> {
        if self.use_texture.value {
            Some(self.texture.value)
        } else {
            None
        }
    }

    /// The texture used when the pipeline is next applied.
    pub fn set_texture(&mut self, new: Option<i32>) {
        if let Some(tex) = new {
            self.texture.set(tex);
            self.use_texture.set(true);
        } else {
            self.use_texture.set(false);
        }
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
    uniform bool UseTexture0 = false;

    void main()
    {
        if(UseTexture0) {
            vec4 tex_color = texture(Texture0, Vert_Frag_TexCoord);
            FragColor = tex_color * Vert_Frag_Color;
        } else {
            FragColor = Vert_Frag_Color;
        }
    }";
}

impl Pipeline for BasicPipeline2D {
    type Vertex = BasicVertex2D;

    fn apply(&mut self, ctx: &mut Context) -> Result<(), BackendError> {
        // TODO: keep track of currently used program?
        Program::bind(&self.program)?;
        self.transform.set_uniform(&self.program)?;
        self.texture.set_uniform(&self.program)?;
        self.use_texture.set_uniform(&self.program)?;
        ctx.graphics.set_blend_mode(self.blend_mode)?;
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

impl BasicVertex2D {
    pub fn with_position<P: Into<[f32; 2]>>(position: P) -> Self {
        Self::with_position_color(position, Color::WHITE)
    }

    pub fn with_position_color<P: Into<[f32; 2]>, C: Into<[f32; 4]>>(position: P, color: C) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
            tex_coord: [0.0; 2],
        }
    }
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

impl<T: UniformValue + PartialEq + Debug> ShaderParameter<T> {
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

    fn set_uniform(&mut self, program: &Program) -> Result<(), BackendError> {
        if self.dirty {
            log::trace!("setting uniform {} to {:?}", &self.name, &self.value);
            program.set_uniform(&self.name, &self.value)?;
            self.dirty = false;
        }
        Ok(())
    }
}
