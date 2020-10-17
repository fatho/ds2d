use std::{path::Path, rc::Rc};

use crate::Context;

use super::context::Texture;

/// A reference counted 2D texture that can be cheaply shared between several sprites.
pub struct Texture2D {
    raw: Rc<Texture>,
}

impl Texture2D {
    pub fn from_file<P: AsRef<Path>>(
        ctx: &mut Context,
        filename: P,
    ) -> Result<Self, super::GraphicsError> {
        // Always load textures as RGBA for now
        // TODO: add support for other formats?
        log::trace!("Loading image {}", filename.as_ref().display());
        let image = image::open(filename)?.into_rgba();

        Self::from_image(ctx, &image)
    }

    pub fn from_image(
        _ctx: &mut Context,
        image: &image::RgbaImage,
    ) -> Result<Self, super::GraphicsError> {
        let raw = Texture::new()?;

        if image.width().max(image.height()) >= std::i32::MAX as u32 {
            return Err(super::GraphicsError::Backend(super::BackendError::TooLarge));
        }

        Texture::bind(gl::TEXTURE_2D, &raw)?;
        Texture::image2d_rgba(
            gl::TEXTURE_2D,
            image.width() as i32,
            image.height() as i32,
            &*image,
        )?;
        Texture::unbind(gl::TEXTURE_2D)?;

        Ok(Texture2D { raw: Rc::new(raw) })
    }

    pub fn raw(&self) -> &Texture {
        &self.raw
    }
}
