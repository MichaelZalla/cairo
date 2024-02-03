use std::{borrow::BorrowMut, sync::RwLock};

use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{
    buffer::Buffer2D,
    color::{self, Color},
    debug::message::DebugMessageBuffer,
    font::{cache::FontCache, FontInfo},
    texture::TextureBuffer,
};

use self::cache::{cache_text, TextCache, TextCacheKey};

use super::Graphics;

pub mod cache;

#[derive(Clone)]
pub struct TextOperation<'a> {
    pub text: &'a String,
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

impl Graphics {
    pub fn text<'a>(
        dest_buffer: &mut Buffer2D,
        font_cache_rwl: &'a RwLock<FontCache>,
        text_cache_rwl: Option<&'a RwLock<TextCache<'a>>>,
        font_info: &'a FontInfo,
        op: &TextOperation,
    ) -> Result<(), String> {
        // Generate a texture for this text operation.

        let text_cache_key = TextCacheKey {
            font_info: font_info.clone(),
            text: op.text.clone(),
        };

        match text_cache_rwl {
            Some(lock) => {
                cache_text(font_cache_rwl, lock, font_info, op.text);

                let text_cache = lock.write().unwrap();

                let cached_texture = text_cache.get(&text_cache_key).unwrap();

                Graphics::blit_text_from_mask(&cached_texture, &op, dest_buffer, None);
            }
            None => {
                let mut font_cache = font_cache_rwl.write().unwrap();

                let font = font_cache.load(font_info).unwrap();

                let (_label_width, _label_height, texture) =
                    Graphics::make_text_texture(font.as_ref(), &op.text).unwrap();

                Graphics::blit_text_from_mask(&texture, &op, dest_buffer, None);
            }
        }

        Ok(())
    }

    pub fn blit_text_from_mask(
        texture: &Buffer2D<u8>,
        op: &TextOperation,
        dest_buffer: &mut Buffer2D<u32>,
        max_width: Option<u32>,
    ) {
        if op.x >= dest_buffer.width {
            return;
        }

        if op.y >= dest_buffer.height {
            return;
        }

        let color_u32 = op.color.to_u32();

        for y_rel in 0..texture.height.min(dest_buffer.height - op.y) {
            for x_rel in
                0..texture
                    .width
                    .min(dest_buffer.width - op.x)
                    .min(if max_width.is_some() {
                        max_width.unwrap()
                    } else {
                        u32::MAX
                    })
            {
                let index = (x_rel as usize + y_rel as usize * texture.width as usize) * 4;

                let a = texture.data[index + 3];

                if a == 0 {
                    // Skips unrendered pixels in our text texture (mask).

                    continue;
                }

                dest_buffer.set(op.x + x_rel, op.y + y_rel, color_u32)
            }
        }
    }

    pub fn render_debug_messages<'a>(
        dest_buffer: &mut Buffer2D,
        font_cache: &'a RwLock<FontCache>,
        font_info: &'a FontInfo,
        position: (u32, u32),
        padding_ems: f32,
        debug_messages: &mut DebugMessageBuffer,
    ) {
        let mut y_offset = position.1;

        for msg in debug_messages.borrow_mut() {
            let op = TextOperation {
                text: &msg,
                x: position.0,
                y: y_offset,
                color: color::WHITE,
            };

            Graphics::text(dest_buffer, font_cache, None, &font_info, &op).unwrap();

            y_offset += (font_info.point_size as f32 * padding_ems) as u32;
        }

        debug_messages.drain();
    }

    pub fn make_text_texture(
        font: &Font,
        text: &String,
    ) -> Result<(u32, u32, TextureBuffer), String> {
        // Generate a new text texture (mask).

        let color = color::WHITE;

        let surface = font
            .render(text)
            .blended(SDLColor::RGBA(color.r, color.g, color.b, color.a))
            .map_err(|e| e.to_string())?;

        // Read the pixel data from the rendered surface

        let text_surface_canvas = surface.into_canvas()?;

        let text_surface_canvas_size = text_surface_canvas.output_size()?;

        let width = text_surface_canvas_size.0;
        let height = text_surface_canvas_size.1;

        let bytes = text_surface_canvas.read_pixels(None, sdl2::pixels::PixelFormatEnum::RGBA32)?;

        let buffer = Buffer2D::from_data(width, height, bytes);

        Ok((width, height, buffer))
    }
}
