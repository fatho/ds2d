use glutin::{dpi::PhysicalSize, WindowedContext};
use std::rc::Rc;

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

    /// Check whether an OpenGL error occurred recently.
    pub fn _check_errors(&mut self) -> crate::GameResult<()> {
        use crate::GameError;

        let mut error = unsafe { gl::GetError() };
        let first_error = error;
        // Just in case there are more errors queued up, try and drain the queue
        while error != gl::NO_ERROR {
            error = unsafe { gl::GetError() };
        }
        match first_error {
            gl::NO_ERROR => Ok(()),
            gl::INVALID_ENUM => Err(GameError::Graphics("GL_INVALID_ENUM".into())),
            gl::INVALID_VALUE => Err(GameError::Graphics("GL_INVALID_VALUE".into())),
            gl::INVALID_OPERATION => Err(GameError::Graphics("GL_INVALID_OPERATION".into())),
            gl::STACK_OVERFLOW => Err(GameError::Graphics("GL_STACK_OVERFLOW".into())),
            gl::STACK_UNDERFLOW => Err(GameError::Graphics("GL_STACK_UNDERFLOW".into())),
            gl::OUT_OF_MEMORY => Err(GameError::Graphics("GL_OUT_OF_MEMORY".into())),
            gl::INVALID_FRAMEBUFFER_OPERATION => Err(GameError::Graphics(
                "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
            )),
            _ => Err(GameError::Graphics(format!(
                "Unknown GL error {}",
                first_error
            ))),
        }
    }

    // pub fn create_shader(&mut self, shader_type: ShaderType) -> ShaderHandle {
    //     let gl_type = match shader_type {
    //         ShaderType::Vertex => gl::VERTEX_SHADER,
    //         ShaderType::Fragment => gl::FRAGMENT_SHADER,
    //     };
    //     //gl::create_shader(gl_type)
    // }
}

/// The supported kinds of shaders.
pub enum ShaderType {
    /// A vertex shader, for processing the vertices of a mesh.
    Vertex,
    /// A fragment shader, for processing the rasterized pixels of a mesh.
    Fragment,
}

pub struct ShaderHandle {
    inner: ShaderHandleImpl,
}

struct ShaderHandleImpl {}
