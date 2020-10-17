use gl::types::GLenum;

use crate::CheckGl;

use super::BackendError;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlendMode {
    pub func_rgb: BlendFunc,
    pub func_alpha: BlendFunc,
    pub src_rgb: BlendParam,
    pub src_alpha: BlendParam,
    pub dst_rgb: BlendParam,
    pub dst_alpha: BlendParam,
}

impl BlendMode {
    pub const fn new(func: BlendFunc, src: BlendParam, dst: BlendParam) -> Self {
        Self {
            func_rgb: func,
            func_alpha: func,
            src_alpha: src,
            src_rgb: src,
            dst_alpha: dst,
            dst_rgb: dst,
        }
    }

    pub const fn alpha_blend() -> Self {
        Self {
            func_rgb: BlendFunc::Add,
            func_alpha: BlendFunc::Add,
            src_rgb: BlendParam::SrcAlpha,
            dst_rgb: BlendParam::OneMinusSrcAlpha,
            src_alpha: BlendParam::One,
            dst_alpha: BlendParam::Zero,
        }
    }

    pub const fn additive() -> Self {
        Self {
            func_rgb: BlendFunc::Add,
            func_alpha: BlendFunc::Add,
            src_rgb: BlendParam::One,
            dst_rgb: BlendParam::One,
            src_alpha: BlendParam::One,
            dst_alpha: BlendParam::One,
        }
    }

    pub const fn multiplicative() -> Self {
        Self {
            func_rgb: BlendFunc::Add,
            func_alpha: BlendFunc::Add,
            src_rgb: BlendParam::DstColor,
            dst_rgb: BlendParam::Zero,
            src_alpha: BlendParam::DstAlpha,
            dst_alpha: BlendParam::Zero,
        }
    }

    pub(crate) fn apply(self) -> Result<(), BackendError> {
        unsafe {
            CheckGl!(gl::BlendEquationSeparate(
                self.func_rgb.to_gl(),
                self.func_alpha.to_gl()
            ))?;
            CheckGl!(gl::BlendFuncSeparate(
                self.src_rgb.to_gl(),
                self.dst_rgb.to_gl(),
                self.src_alpha.to_gl(),
                self.dst_alpha.to_gl()
            ))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum BlendFunc {
    /// `result = source_factor * source_color + dest_factor * dest_color`
    Add,
    /// `result = source_factor * source_color - dest_factor * dest_color`
    Subtract,
    /// `result = dest_factor * dest_color - source_factor * source_color`
    ReverseSubtract,
}

impl BlendFunc {
    fn to_gl(self) -> GLenum {
        match self {
            BlendFunc::Add => gl::FUNC_ADD,
            BlendFunc::Subtract => gl::FUNC_SUBTRACT,
            BlendFunc::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum BlendParam {
    Zero,
    One,
    SrcAlpha,
    DstAlpha,
    OneMinusSrcAlpha,
    OneMinusDstAlpha,
    SrcColor,
    DstColor,
    OneMinusSrcColor,
    OneMinusDstColor,
}

impl BlendParam {
    fn to_gl(self) -> GLenum {
        match self {
            BlendParam::Zero => gl::ZERO,
            BlendParam::One => gl::ONE,
            BlendParam::SrcAlpha => gl::SRC_ALPHA,
            BlendParam::DstAlpha => gl::DST_ALPHA,
            BlendParam::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendParam::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendParam::SrcColor => gl::SRC_COLOR,
            BlendParam::DstColor => gl::DST_COLOR,
            BlendParam::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            BlendParam::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
        }
    }
}
