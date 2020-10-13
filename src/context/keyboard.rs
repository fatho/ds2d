use std::collections::HashSet;

use glutin::event::VirtualKeyCode;

#[derive(Debug, Default)]
pub(crate) struct KeyboardContext {
    pub pressed_keys: HashSet<VirtualKeyCode>,
    pub unicode_text: String,
}
