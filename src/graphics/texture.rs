use std::{path::Path, rc::Rc};

use crate::{CheckGl, Context};

use super::context::Texture;

/// A reference counted 2D texture that can be cheaply shared between several sprites.
#[derive(Debug)]
pub struct Texture2D {
    inner: Rc<Texture2DImpl>,
}

#[derive(Debug)]
struct Texture2DImpl {
    raw: Texture,
    width: u32,
    height: u32,
}

impl Texture2D {
    pub fn from_memory(ctx: &mut Context, data: &[u8]) -> Result<Self, super::GraphicsError> {
        let image = image::load_from_memory(data)?.into_rgba();
        Self::from_image(ctx, &image)
    }

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
        unsafe {
            CheckGl!(gl::GenerateMipmap(gl::TEXTURE_2D))?;
        }
        Texture::unbind(gl::TEXTURE_2D)?;

        Ok(Texture2D {
            inner: Rc::new(Texture2DImpl {
                raw,
                width: image.width(),
                height: image.height(),
            }),
        })
    }

    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }

    pub fn raw(&self) -> &Texture {
        &self.inner.raw
    }
}
