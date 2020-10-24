use std::{path::Path, rc::Rc};

use cgmath::Vector2;

use crate::{CheckGl, Context};

use super::{context::Texture, Color, Rect};

/// A reference counted 2D texture that can be cheaply shared between several sprites.
#[derive(Debug, Clone)]
pub struct Texture2D {
    inner: Rc<Texture2DImpl>,
}

#[derive(Debug)]
struct Texture2DImpl {
    raw: Texture,
    width: u32,
    height: u32,
}

impl PartialEq for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner) || self.inner.raw.id() == other.inner.raw.id()
    }
}

impl Eq for Texture2D {}

impl Texture2D {
    pub fn solid(
        ctx: &mut Context,
        width: u32,
        height: u32,
        color: Color,
    ) -> Result<Self, super::GraphicsError> {
        let mut data = Vec::new();
        for _ in 0..(width * height) {
            data.extend(&color.to_rgba_u8());
        }
        let image = image::RgbaImage::from_raw(width, height, data)
            .expect("Image bounds should correspond to buffer size");
        Self::from_image(ctx, &image)
    }

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

        unsafe {
            Texture::bind(gl::TEXTURE_2D, &raw)?;
            Texture::image2d_rgba(
                gl::TEXTURE_2D,
                image.width() as i32,
                image.height() as i32,
                &*image,
            )?;
            CheckGl!(gl::GenerateMipmap(gl::TEXTURE_2D))?;
            Texture::unbind(gl::TEXTURE_2D)?;
        }

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

#[derive(Debug, Clone)]
pub struct TextureView2D {
    pub texture: Texture2D,
    pub source: Rect<f32>,
}

impl TextureView2D {
    pub fn new(texture: Texture2D, source: Rect<f32>) -> Self {
        Self { texture, source }
    }

    pub fn size(&self) -> Vector2<f32> {
        let uv_size = self.source.size();
        Vector2 {
            x: self.texture.width() as f32 * uv_size.x,
            y: self.texture.height() as f32 * uv_size.y,
        }
    }
}

impl From<Texture2D> for TextureView2D {
    fn from(texture: Texture2D) -> Self {
        Self {
            texture,
            source: Rect::unit_square(),
        }
    }
}
