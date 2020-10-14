//! Implementation of the graphics stack.
//! A lot of this assumes the presence of the global and all-encompassing GL context.

use glutin::{dpi::PhysicalSize, WindowedContext};
use std::rc::Rc;

use crate::{GameError, GameResult};

#[derive(Debug)]
pub(crate) struct GraphicsContext {
    pub windowed_context: Rc<WindowedContext<glutin::PossiblyCurrent>>,
    pub screen_size: PhysicalSize<u32>,
    pub scale_factor: f64,
    pub can_debug: bool,
}

impl GraphicsContext {
    pub fn new(windowed_context: Rc<WindowedContext<glutin::PossiblyCurrent>>) -> Self {
        {
            let windowed_context = windowed_context.clone();
            gl::load_with(move |s| windowed_context.get_proc_address(s) as *const _);
        }

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
        }
    }

    pub fn init_debug(&mut self) {
        if self.can_debug {
            unsafe {
                gl::Enable(gl::DEBUG_OUTPUT);

                use gl::types::{GLchar, GLenum, GLsizei, GLuint};
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

    pub fn get_error(&mut self, context: &str) -> GameResult<()> {
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
        Err(GameError::Graphics(format!("{}: {}", context, description)))
    }

    /// Create a shader by compiling source code.
    pub fn create_shader(&mut self, shader_type: ShaderType, source: &str) -> GameResult<Shader> {
        let gl_type = match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        };
        let shader = unsafe { gl::CreateShader(gl_type) };
        if shader == 0 {
            self.get_error("CreateShader")?;
            return Err(GameError::Graphics("Could not create shader but no error reported".into()));
        }
        log::trace!("CreateShader() = {}", shader);
        let shader = Shader { id: shader };

        unsafe {
            // Load and compile source code
            gl::ShaderSource(shader.id, 1, &(source.as_ptr() as _), &(source.len() as _));
            self.get_error("ShaderSource")?;
            gl::CompileShader(shader.id);
            self.get_error("CompileShader")?;
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

    /// Create a program by linking individual shaders.
    pub fn create_program(&mut self, shaders: &[Shader]) -> GameResult<Program> {
        let program = unsafe { gl::CreateProgram() };
        if program == 0 {
            self.get_error("CreateProgram")?;
            return Err(GameError::Graphics("Could not create program but no error reported".into()));
        }
        log::trace!("CreateProgram() = {}", program);
        let program = Program { id: program };

        unsafe {
            // Link all the shaders
            for shader in shaders {
                gl::AttachShader(program.id, shader.id);
                self.get_error("AttachShader")?;
            }
            gl::LinkProgram(program.id);
            self.get_error("LinkProgram")?;
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
        }

        Ok(program)
    }
}

/// The supported kinds of shaders.
#[derive(Debug)]
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
}

impl Drop for Program {
    fn drop(&mut self) {
        // TODO: how to make sure this happens in a safe way?
        log::trace!("DeleteProgram({})", self.id);
        unsafe { gl::DeleteProgram(self.id); }
    }
}

// TODO: implement useful shader

/// Default vertex shader that will be used for rendering textured meshes in 2D.
pub const DEFAULT_VERTEX_SHADER: &str = r"#version 330 core
layout (location = 0) in vec2 pos;

void main()
{
    gl_Position = vec4(pos.x, pos.y,0.0, 1.0);
}";

/// Default fragment shader that will be used for rendering textured meshes in 2D.
pub const DEFAULT_FRAGMENT_SHADER: &str = r"#version 330 core
out vec4 FragColor;

void main()
{
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}";
