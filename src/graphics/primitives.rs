use cgmath::Matrix3;

use super::BlendMode;

// pub struct Decomposed {

// }

#[derive(Debug, Copy, Clone)]
pub struct RenderState {
    pub transform: Matrix3<f32>,
    pub blend: Option<BlendMode>,
}
