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
}
