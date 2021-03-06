use cgmath::Vector2;

use crate::Context;

use super::{
    context::Texture, Color, Drawable, GraphicsError, Quad, Rect, Texture2D, TextureView2D,
};

pub struct Rasterizer {
    renderer: super::BatchRender,
    cache_texture: Texture2D,
    cache: rusttype::gpu_cache::Cache<'static>,
    font_id_next: usize,
}

impl Rasterizer {
    pub fn new(ctx: &mut Context) -> Result<Self, GraphicsError> {
        // TODO: make dimensions of cache configurable
        let width = 1024;
        let height = width;
        let cache_texture = Texture2D::solid(ctx, width, height, Color::BLACK)?;
        let renderer = super::BatchRender::new(ctx)?;

        Ok(Self {
            renderer,
            cache_texture,
            cache: rusttype::gpu_cache::Cache::builder()
                .dimensions(width as u32, height as u32)
                .build(),
            font_id_next: 0,
        })
    }

    pub fn create_font(&mut self, data: Vec<u8>) -> Result<Font, GraphicsError> {
        let inner = rusttype::Font::try_from_vec(data).ok_or(GraphicsError::InvalidFont)?;
        let font_id = self.font_id_next;
        self.font_id_next += 1;
        Ok(Font { inner, font_id })
    }

    pub fn rasterize<'r, 't>(&'r mut self, text: &'t TextBuffer) -> RasterizedText<'r, 't> {
        RasterizedText {
            rasterizer: self,
            text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    inner: rusttype::Font<'static>,
    font_id: usize,
}

impl Font {
    fn layout_single_line<'a, 'b>(
        &'a self,
        scale: rusttype::Scale,
        start: rusttype::Point<f32>,
        text: &'b str,
    ) -> SingleLineLayout<'a, 'static, 'b> {
        SingleLineLayout {
            font: &self.inner,
            chars: text.chars(),
            caret: 0.0,
            scale,
            start,
            last_glyph: None,
        }
    }
}

pub struct TextBuffer {
    glyphs: Vec<StyledGlyph>,
}

struct StyledGlyph {
    font_id: usize,
    color: Color,
    glyph: rusttype::PositionedGlyph<'static>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self { glyphs: Vec::new() }
    }

    pub fn singleton<S: AsRef<str>>(style: &Style, position: Vector2<f32>, text: S) -> Self {
        let mut buf = Self::new();
        buf.add(style, position, text);
        buf
    }

    pub fn add<S: AsRef<str>>(&mut self, style: &Style, position: Vector2<f32>, text: S) {
        let scale = rusttype::Scale::uniform(style.size);
        let start = rusttype::point(position.x, position.y);
        for glyph in style.font.layout_single_line(scale, start, text.as_ref()) {
            let prepared = StyledGlyph {
                font_id: style.font.font_id,
                color: style.color,
                glyph,
            };
            self.glyphs.push(prepared);
        }
    }
}

pub struct Style {
    pub font: Font,
    pub size: f32,
    pub color: Color,
}

pub struct RasterizedText<'r, 't> {
    rasterizer: &'r mut Rasterizer,
    text: &'t TextBuffer,
}

impl<'r, 't> Drawable for RasterizedText<'r, 't> {
    fn draw(&mut self, ctx: &mut Context, state: super::RenderState) -> crate::GameResult<()> {
        unsafe {
            Texture::bind(gl::TEXTURE_2D, &self.rasterizer.cache_texture.raw())?;
        }

        for glyph in &self.text.glyphs {
            self.rasterizer
                .cache
                .queue_glyph(glyph.font_id, glyph.glyph.clone());
        }
        self.rasterizer
            .cache
            .cache_queued(|region, data| {
                // A bit wasteful, but then we can reuse the existing basic pipeline.
                // On the other hand, this is already future-proofed for colorful emoji
                // rendering (if ever supported by rusttype).
                let mut rgba_data: Vec<u8> = Vec::new();
                for alpha in data {
                    rgba_data.extend(&[255, 255, 255, *alpha]);
                }
                // Only we have access to the texture, so it is safe to modify
                unsafe {
                    let size = region.max - region.min;
                    if let Err(err) = Texture::subimage2d_rgba(
                        gl::TEXTURE_2D,
                        0,
                        region.min.x as i32,
                        region.min.y as i32,
                        size.x as i32,
                        size.y as i32,
                        &rgba_data,
                    ) {
                        log::error!("Failed to upload glyph: {}", err);
                    }
                }
            })
            .map_err(|_| GraphicsError::InsufficientGlyphCache)?;

        unsafe {
            Texture::unbind(gl::TEXTURE_2D)?;
        }

        for glyph in &self.text.glyphs {
            if let Ok(Some((tex_coords, screen_coords))) =
                self.rasterizer.cache.rect_for(glyph.font_id, &glyph.glyph)
            {
                let tex = TextureView2D {
                    texture: self.rasterizer.cache_texture.clone(),
                    source: tex_coords.into(),
                };
                let pos_size: Rect<i32> = screen_coords.into();
                self.rasterizer.renderer.draw_quad(
                    Quad::textured(tex)
                        .with_position(pos_size.top_left.map(|x| x as f32))
                        .with_size(pos_size.size().map(|x| x as f32))
                        .with_tint(glyph.color),
                )
            }
        }

        self.rasterizer.renderer.draw(ctx, state)
    }
}

#[derive(Clone)]
struct SingleLineLayout<'a, 'font, 's> {
    font: &'a rusttype::Font<'font>,
    chars: core::str::Chars<'s>,
    caret: f32,
    scale: rusttype::Scale,
    start: rusttype::Point<f32>,
    last_glyph: Option<rusttype::GlyphId>,
}

impl<'a, 'font, 's> Iterator for SingleLineLayout<'a, 'font, 's> {
    type Item = rusttype::PositionedGlyph<'font>;

    fn next(&mut self) -> Option<rusttype::PositionedGlyph<'font>> {
        self.chars.next().map(|c| {
            let g = self.font.glyph(c).scaled(self.scale);
            if let Some(last) = self.last_glyph {
                self.caret += self.font.pair_kerning(self.scale, last, g.id());
            }
            let w = g.h_metrics().advance_width;
            let g = g.positioned(rusttype::point(self.start.x + self.caret, self.start.y));
            self.caret += w;
            self.last_glyph = Some(g.id());
            g
        })
    }
}
