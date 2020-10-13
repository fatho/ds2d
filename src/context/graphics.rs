use glow::HasContext;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct GraphicsContext {
    pub gl: glow::Context,
    pub can_debug: bool,
}

impl GraphicsContext {
    pub fn new(windowed_context: Rc<glutin::WindowedContext<glutin::PossiblyCurrent>>) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            })
        };

        let version = unsafe { gl.get_parameter_string(glow::VERSION) };
        let shader_version = unsafe { gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION) };
        log::info!("Using {} (shading language {})", version, shader_version);

        let num_extensions = unsafe { gl.get_parameter_i32(glow::NUM_EXTENSIONS) };
        let mut can_debug = false;
        for i in 0..num_extensions {
            let ext = unsafe { gl.get_parameter_indexed_string(glow::EXTENSIONS, i as u32) };
            if ext == "GL_KHR_debug" {
                can_debug = true;
            }
            log::debug!("Found extension: {}", ext);
        }

        Self { gl, can_debug }
    }

    pub fn init_debug(&mut self) {
        if self.can_debug {
            unsafe {
                self.gl.enable(glow::DEBUG_OUTPUT);
                self.gl.debug_message_callback(|source, msg_type, msg_id, msg_severity, msg_str| {
                    let source = match source {
                        glow::DEBUG_SOURCE_API => "api",
                        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "window",
                        glow::DEBUG_SOURCE_SHADER_COMPILER => "shader",
                        glow::DEBUG_SOURCE_THIRD_PARTY => "thirdparty",
                        glow::DEBUG_SOURCE_APPLICATION => "user",
                        _ => "other",
                    };
                    let msg_type = match msg_type {
                        glow::DEBUG_TYPE_ERROR => "error",
                        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated",
                        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
                        glow::DEBUG_TYPE_PORTABILITY => "portability",
                        glow::DEBUG_TYPE_PERFORMANCE => "performance",
                        glow::DEBUG_TYPE_MARKER => "marker",
                        glow::DEBUG_TYPE_PUSH_GROUP => "push group",
                        glow::DEBUG_TYPE_POP_GROUP => "pop group",
                        _ => "other",
                    };
                    let severity = match msg_severity {
                        glow::DEBUG_SEVERITY_HIGH => log::Level::Error,
                        glow::DEBUG_SEVERITY_MEDIUM => log::Level::Warn,
                        glow::DEBUG_SEVERITY_LOW => log::Level::Info,
                        glow::DEBUG_SEVERITY_NOTIFICATION => log::Level::Debug,
                        _ => log::Level::Info,
                    };
                    log::log!(target: "opengl", severity, "{} {} {}: {}", source, msg_type, msg_id, msg_str);
                });
            }
        }
    }

    /// Check whether an OpenGL error occurred recently.
    pub fn _check_errors(&mut self) -> crate::GameResult<()> {
        use crate::GameError;

        let mut error = unsafe { self.gl.get_error() };
        let first_error = error;
        // Just in case there are more errors queued up, try and drain the queue
        while error != glow::NO_ERROR {
            error = unsafe { self.gl.get_error() };
        }
        match first_error {
            glow::NO_ERROR => Ok(()),
            glow::INVALID_ENUM => Err(GameError::Graphics("GL_INVALID_ENUM".into())),
            glow::INVALID_VALUE => Err(GameError::Graphics("GL_INVALID_VALUE".into())),
            glow::INVALID_OPERATION => Err(GameError::Graphics("GL_INVALID_OPERATION".into())),
            glow::STACK_OVERFLOW => Err(GameError::Graphics("GL_STACK_OVERFLOW".into())),
            glow::STACK_UNDERFLOW => Err(GameError::Graphics("GL_STACK_UNDERFLOW".into())),
            glow::OUT_OF_MEMORY => Err(GameError::Graphics("GL_OUT_OF_MEMORY".into())),
            glow::INVALID_FRAMEBUFFER_OPERATION => Err(GameError::Graphics(
                "GL_INVALID_FRAMEBUFFER_OPERATION".into(),
            )),
            _ => Err(GameError::Graphics(format!(
                "Unknown GL error {}",
                first_error
            ))),
        }
    }
}
