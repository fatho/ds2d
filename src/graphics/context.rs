//! Implementation of the graphics stack.
//! A lot of this assumes the presence of the global and all-encompassing GL context.

use cgmath::{Matrix3, Vector2};
use gl::types::GLenum;
use glutin::{dpi::PhysicalSize, WindowedContext};
use std::{collections::HashMap, rc::Rc};

use crate::{GameError, GameResult};

#[derive(Debug)]
pub(crate) struct GraphicsContext {
    pub windowed_context: Rc<WindowedContext<glutin::PossiblyCurrent>>,
    pub screen_size: PhysicalSize<u32>,
    pub scale_factor: f64,
    pub can_debug: bool,
    /// Converting pixel coordinates to normalized device coordinates
    pub pixel_projection: Matrix3<f32>,
}

impl GraphicsContext {
    pub fn new(windowed_context: WindowedContext<glutin::PossiblyCurrent>) -> Self {
        let windowed_context = Rc::new(windowed_context);
        gl::load_with({
            let windowed_context = windowed_context.clone();
            move |s| windowed_context.get_proc_address(s) as *const _
        });

        let version = unsafe { gl::GetString(gl::VERSION) };
        let version = if version == std::ptr::null() {
            std::borrow::Cow::Borrowed("version unavailable")
        } else {
            unsafe { std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy() }
        };

        let shader_version = unsafe { gl::GetString(gl::SHADING_LANGUAGE_VERSION) };
        let shader_version = if shader_version == std::ptr::null() {
            std::borrow::Cow::Borrowed("version unavailable")
        } else {
            unsafe { std::ffi::CStr::from_ptr(shader_version as *const i8).to_string_lossy() }
        };

        log::info!("Using {} (shading language {})", version, shader_version);

        let mut num_extensions = 0;
        unsafe { gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions as *mut _) };
        let mut can_debug = false;
        for i in 0..num_extensions {
            let ext = unsafe { gl::GetStringi(gl::EXTENSIONS, i as u32) };
            if ext != std::ptr::null() {
                let ext = unsafe { std::ffi::CStr::from_ptr(ext as *const i8).to_string_lossy() };
                if ext == "GL_KHR_debug" {
                    can_debug = true;
                }
                log::debug!("Found extension {}", ext);
            }
        }
        let screen_size = windowed_context.window().inner_size();
        let scale_factor = windowed_context.window().scale_factor();

        Self {
            windowed_context,
            screen_size,
            scale_factor,
            can_debug,
            pixel_projection: compute_pixel_projection(screen_size),
        }
    }

    pub fn init_debug(&mut self) {
        if self.can_debug {
            unsafe {
                gl::Enable(gl::DEBUG_OUTPUT);

                use gl::types::{GLchar, GLsizei, GLuint};
                use std::ffi::c_void;

                extern "system" fn callback(
                    source: GLenum,
                    gltype: GLenum,
                    id: GLuint,
                    severity: GLenum,
                    length: GLsizei,
                    message: *const GLchar,
                    _user: *mut c_void,
                ) {
                    let source = match source {
                        gl::DEBUG_SOURCE_API => "api",
                        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window",
                        gl::DEBUG_SOURCE_SHADER_COMPILER => "shader",
                        gl::DEBUG_SOURCE_THIRD_PARTY => "thirdparty",
                        gl::DEBUG_SOURCE_APPLICATION => "user",
                        _ => "other",
                    };
                    let msg_type = match gltype {
                        gl::DEBUG_TYPE_ERROR => "error",
                        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated",
                        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
                        gl::DEBUG_TYPE_PORTABILITY => "portability",
                        gl::DEBUG_TYPE_PERFORMANCE => "performance",
                        gl::DEBUG_TYPE_MARKER => "marker",
                        gl::DEBUG_TYPE_PUSH_GROUP => "push group",
                        gl::DEBUG_TYPE_POP_GROUP => "pop group",
                        _ => "other",
                    };
                    let severity = match severity {
                        gl::DEBUG_SEVERITY_HIGH => log::Level::Error,
                        gl::DEBUG_SEVERITY_MEDIUM => log::Level::Warn,
                        gl::DEBUG_SEVERITY_LOW => log::Level::Info,
                        gl::DEBUG_SEVERITY_NOTIFICATION => log::Level::Debug,
                        _ => log::Level::Info,
                    };
                    let msgbytes: &[u8] = unsafe {
                        std::slice::from_raw_parts(message as *const u8, length as usize)
                    };
                    let msg_str = String::from_utf8_lossy(msgbytes);
                    log::log!(target: "opengl", severity, "{} {} {}: {}", source, msg_type, id, msg_str);
                }

                gl::DebugMessageCallback(Some(callback), std::ptr::null());
            }
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.windowed_context.resize(new_size);
        unsafe { gl::Viewport(0, 0, new_size.width as i32, new_size.height as i32) };
        self.screen_size = new_size;
        self.pixel_projection = compute_pixel_projection(new_size);

        log::trace!("Top left: {:?}", self.pixel_projection * Vector2::new(0.0, 0.0).extend(1.0));
        log::trace!("Top right: {:?}", self.pixel_projection * Vector2::new(new_size.width as f32, 0.0).extend(1.0));
        log::trace!("Bottom left: {:?}", self.pixel_projection * Vector2::new(0.0, new_size.height as f32).extend(1.0));
        log::trace!("Bottom right: {:?}", self.pixel_projection * Vector2::new(new_size.width as f32, new_size.height as f32).extend(1.0));
    }
}

pub fn compute_pixel_projection(screen_size: PhysicalSize<u32>) -> cgmath::Matrix3<f32> {
    let scale = super::transform::scale(
        Vector2::new(2.0 / screen_size.width as f32, -2.0 / screen_size.height as f32)
    );
    let translate = super::transform::translate(
        Vector2::new(-1.0, 1.0)
    );
    translate * scale
}


pub fn get_error(file: &str, line: u32, column: u32, expression: &str) -> GameResult<()> {
    let err = unsafe { gl::GetError() };
    let description = match err {
        gl::NO_ERROR => return Ok(()),
        gl::INVALID_ENUM => "GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        // Shouldn't actually occur according to spec
        _ => "UNKNOWN",
    };
    Err(GameError::Graphics(format!("{} ({}:{}:{}) returned error {}", expression, file, line, column, description)))
}

macro_rules! CheckGl {
    ($gl_call:expr) => {
        {
            let result = $gl_call;
            match get_error(file!(), line!(), column!(), stringify!($gl_call)) {
                Ok(()) => Ok(result),
                Err(err) => Err(err),
            }
        }
    };
}

macro_rules! CheckGlNonZero {
    ($gl_call:expr) => {
        {
            let result = $gl_call;
            if result != 0 {
                Ok(result)
            } else {
                match get_error(file!(), line!(), column!(), stringify!($gl_call)) {
                    Ok(()) => Err(GameError::Graphics("Unexpected GL_NO_ERROR".to_string())),
                    Err(err) => Err(err),
                }
            }
        }
    };
}

/// The supported kinds of shaders.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ShaderType {
    /// A vertex shader, for processing the vertices of a mesh.
    Vertex,
    /// A fragment shader, for processing the rasterized pixels of a mesh.
    Fragment,
}

#[derive(Debug)]
pub struct Shader {
    id: u32,
}

impl Shader {
    /// Create a shader by compiling source code.
    pub fn new(shader_type: ShaderType, source: &str) -> GameResult<Shader> {
        let gl_type = match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        };
        let shader = unsafe { CheckGlNonZero!(gl::CreateShader(gl_type))? };
        log::trace!("CreateShader() = {}", shader);
        let shader = Shader { id: shader };

        unsafe {
            // Load and compile source code
            CheckGl!(gl::ShaderSource(shader.id, 1, &(source.as_ptr() as _), &(source.len() as _)))?;
            CheckGl!(gl::CompileShader(shader.id))?;
            // Check result
            let mut status = 0i32;
            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut status);
            if status != gl::TRUE as i32 {
                // Return shader compiler output as error
                let mut info_log_length = 0;
                gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut info_log_length);
                let mut buffer = vec![0u8; info_log_length as usize];
                gl::GetShaderInfoLog(shader.id, info_log_length, std::ptr::null_mut(), buffer.as_mut_ptr() as _);
                let log = String::from_utf8_lossy(&buffer[0..buffer.len().saturating_sub(1)]);
                return Err(GameError::Graphics(format!("Failed to compile shader:\n{}", log)));
            }
        }

        Ok(shader)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        // TODO: how to make sure this happens in a safe way?
        log::trace!("DeleteShader({})", self.id);
        unsafe { gl::DeleteShader(self.id); }
    }
}

#[derive(Debug)]
pub struct Program {
    id: u32,
    uniforms: HashMap<String, UniformInfo>,
}

impl Program {
    /// Create a program by linking individual shaders.
    pub fn new(shaders: &[Shader]) -> GameResult<Program> {
        let program = unsafe { CheckGlNonZero!(gl::CreateProgram())? };
        log::trace!("CreateProgram() = {}", program);
        let mut program = Program { id: program, uniforms: HashMap::new() };

        unsafe {
            // Link all the shaders
            for shader in shaders {
                CheckGl!(gl::AttachShader(program.id, shader.id))?;
            }
            CheckGl!(gl::LinkProgram(program.id))?;
            // Check result
            let mut status = 0i32;
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut status);
            if status != gl::TRUE as i32 {
                // Return program compiler output as error
                let mut info_log_length = 0;
                gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut info_log_length);
                let mut buffer = vec![0u8; info_log_length as usize];
                gl::GetProgramInfoLog(program.id, info_log_length, std::ptr::null_mut(), buffer.as_mut_ptr() as _);
                let log = String::from_utf8_lossy(&buffer[0..buffer.len().saturating_sub(1)]);
                return Err(GameError::Graphics(format!("Failed to link program:\n{}", log)));
            }

            // query number of uniforms
            let mut num_uniforms = 0;
            CheckGl!(gl::GetProgramiv(program.id, gl::ACTIVE_UNIFORMS, &mut num_uniforms))?;
            // query maximum length of a uniform name including the null terminator
            let mut max_len_uniform = 0;
            CheckGl!(gl::GetProgramiv(program.id, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_len_uniform))?;
            assert!(max_len_uniform >= 0);
            // query uniforms
            let mut uniform_name_buffer = vec![0u8; max_len_uniform as usize];
            for i in 0..num_uniforms {
                let mut name_len = 0;
                let mut size = 0;
                let mut gl_type = 0;
                CheckGl!(gl::GetActiveUniform(program.id, i as u32, max_len_uniform, &mut name_len, &mut size, &mut gl_type, uniform_name_buffer.as_mut_ptr() as *mut i8))?;
                let name_bytes = &uniform_name_buffer[0..name_len as usize];
                let name_bytes_with_nul = &uniform_name_buffer[0..(name_len + 1) as usize];
                if let Ok(name) = std::str::from_utf8(name_bytes) {
                    let location = CheckGl!(gl::GetUniformLocation(program.id, name_bytes_with_nul.as_ptr() as _))?;
                    log::debug!("Found uniform {} (type: {}, size: {}, location: {})", name, gl_type, size, location);
                    program.uniforms.insert(name.to_owned(), UniformInfo { gl_type, size, location });
                } else {
                    log::warn!("Ignoring invalid uniform {:?}", name_bytes);
                }
            }
        }

        Ok(program)
    }

    /// Compile a program consisting of a vertex and fragment shader from source.
    pub fn from_source(vertex_shader: &str, fragment_shader: &str) -> GameResult<Program> {
        let vs = Shader::new(ShaderType::Vertex, vertex_shader)?;
        let fs = Shader::new(ShaderType::Fragment, fragment_shader)?;
        Program::new(&[vs, fs])
    }

    fn get_typed_uniform(&self, uniform: &str, expected_type: GLenum) -> GameResult<&UniformInfo> {
        let info = self.uniforms.get(uniform).ok_or_else(|| GameError::Graphics(format!("No such uniform: {}", uniform)))?;
        if info.gl_type != expected_type {
            return Err(GameError::Graphics("Invalid uniform type".to_string()));
        }
        Ok(info)
    }

    pub fn set_uniform_mat3(&self, uniform: &str, value: &Matrix3<f32>) -> GameResult<()> {
        let info = self.get_typed_uniform(uniform, gl::FLOAT_MAT3)?;
        unsafe {
            gl::UniformMatrix3fv(info.location, 1, gl::FALSE, value as *const _ as *const f32);
        }
        Ok(())
    }

    pub fn bind(&self) -> GameResult<()> {
        unsafe { CheckGl!(gl::UseProgram(self.id)) }
    }

    pub fn unbind() -> GameResult<()> {
        unsafe { CheckGl!(gl::UseProgram(0)) }
    }
}

/// Information about a uniform in a shader.
#[derive(Debug)]
pub struct UniformInfo {
    pub gl_type: GLenum,
    pub size: i32,
    pub location: i32,
}


impl Drop for Program {
    fn drop(&mut self) {
        // TODO: how to make sure this happens in a safe way?
        log::trace!("DeleteProgram({})", self.id);
        unsafe { gl::DeleteProgram(self.id); }
    }
}

#[derive(Debug)]
pub struct Buffer {
    id: u32,
}

impl Buffer {
    pub fn new() -> GameResult<Buffer> {
        let mut id = 0;
        unsafe {
            CheckGl!(gl::GenBuffers(1, &mut id))?;
            log::trace!("GenBuffers() = {}", id);
        }
        Ok(Buffer { id })
    }

    pub fn bind(target: GLenum, buffer: &Buffer) -> GameResult<()> {
        unsafe { CheckGl!(gl::BindBuffer(target, buffer.id)) }
    }

    pub fn unbind(target: GLenum) -> GameResult<()> {
        unsafe { CheckGl!(gl::BindBuffer(target, 0)) }
    }

    /// Create the buffer data storage and upload the given array.
    ///
    /// # Safety
    ///
    /// Highly unsafe. Make sure that `T` is a `repr(C)` type.
    pub unsafe fn data<T>(target: GLenum, data: &[T], usage: GLenum) -> GameResult<()> {
        let size = std::mem::size_of_val(data);
        if size > std::isize::MAX as usize {
            return Err(GameError::Graphics("Buffer too large".to_string()))
        }
        CheckGl!(gl::BufferData(target, size as isize, data.as_ptr() as *const _, usage))
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        log::trace!("DeleteBuffers() = {}", self.id);
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}


#[derive(Debug)]
pub struct VertexArray {
    id: u32,
}

impl VertexArray {
    pub fn new() -> GameResult<VertexArray> {
        let mut id = 0;
        unsafe {
            CheckGl!(gl::GenVertexArrays(1, &mut id))?;
            log::trace!("GenVertexArrays() = {}", id);
        }
        Ok(VertexArray { id })
    }

    pub fn bind(&self) -> GameResult<()> {
        unsafe { CheckGl!(gl::BindVertexArray(self.id)) }
    }

    pub fn unbind() -> GameResult<()> {
        unsafe { CheckGl!(gl::BindVertexArray(0)) }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        log::trace!("DeleteVertexArrays() = {}", self.id);
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}