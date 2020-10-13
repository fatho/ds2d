use std::rc::Rc;

use glutin::WindowedContext;

#[derive(Debug)]
pub(crate) struct WindowContext {
    pub windowed_context: Rc<WindowedContext<glutin::PossiblyCurrent>>,
}

impl WindowContext {
    pub fn new(windowed_context: Rc<glutin::WindowedContext<glutin::PossiblyCurrent>>) -> Self {
        Self { windowed_context }
    }
}
