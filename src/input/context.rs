use std::collections::HashSet;

use glutin::event::VirtualKeyCode;
use glutin::event::MouseButton;

#[derive(Debug)]
pub(crate) struct MouseContext {
    pub position: glutin::dpi::PhysicalPosition<f64>,
    pub pressed_buttons: HashSet<MouseButton>,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl Default for MouseContext {
    fn default() -> Self {
        Self {
            position: glutin::dpi::PhysicalPosition { x: 0.0, y: 0.0 },
            pressed_buttons: HashSet::new(),
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct KeyboardContext {
    pub pressed_keys: HashSet<VirtualKeyCode>,
    pub unicode_text: String,
}
