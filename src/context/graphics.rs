use glow::HasContext;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct GraphicsContext {
    pub gl: glow::Context,
}

impl GraphicsContext {
    pub fn new(windowed_context: Rc<glutin::WindowedContext<glutin::PossiblyCurrent>>) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            })
        };
        Self { gl }
    }

    pub fn check_errors(&mut self) -> crate::GameResult<()> {
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
