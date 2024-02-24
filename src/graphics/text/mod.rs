use std::{borrow::BorrowMut, cell::RefCell};

use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{
    buffer::Buffer2D,
    color::{self, Color},
    debug::message::DebugMessageBuffer,
    font::{cache::FontCache, FontInfo},
    texture::map::TextureBuffer,
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
        font_cache_rc: &'a RefCell<FontCache>,
        text_cache_rc: Option<&'a RefCell<TextCache<'a>>>,
        font_info: &'a FontInfo,
        op: &TextOperation,
    ) -> Result<(), String> {
        // Generate a texture for this text operation.

        let text_cache_key = TextCacheKey {
            font_info: font_info.clone(),
            text: op.text.clone(),
        };

        match text_cache_rc {
            Some(lock) => {
                cache_text(font_cache_rc, lock, font_info, op.text);

                let text_cache = lock.borrow_mut();

                let cached_texture = text_cache.get(&text_cache_key).unwrap();

                Graphics::blit_text_from_mask(&cached_texture, &op, dest_buffer, None);
            }
            None => {
                let mut font_cache = font_cache_rc.borrow_mut();

                let font = font_cache.load(font_info).unwrap();

                let (_label_width, _label_height, texture) =
                    Graphics::make_text_mask(font.as_ref(), &op.text).unwrap();

                Graphics::blit_text_from_mask(&texture, &op, dest_buffer, None);
            }
        }

        Ok(())
    }

    pub fn blit_text_from_mask(
        mask: &Buffer2D<u8>,
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

        let available_height = mask.height.min(dest_buffer.height - op.y);

        let available_width =
            mask.width
                .min(dest_buffer.width - op.x)
                .min(if max_width.is_some() {
                    max_width.unwrap()
                } else {
                    u32::MAX
                });

        for y_rel in 0..available_height {
            for x_rel in 0..available_width {
                let index = y_rel as usize * mask.width as usize + x_rel as usize;

                if mask.data[index] == 0 {
                    // Skips unrendered pixels in our text texture (mask).

                    continue;
                }

                dest_buffer.set(op.x + x_rel, op.y + y_rel, color_u32)
            }
        }
    }

    pub fn render_debug_messages<'a>(
        dest_buffer: &mut Buffer2D,
        font_cache: &'a RefCell<FontCache>,
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

    pub fn make_text_mask(font: &Font, text: &String) -> Result<(u32, u32, TextureBuffer), String> {
        // Generate a new text texture (mask).

        let surface = font
            .render(text)
            .solid(SDLColor::WHITE)
            .map_err(|e| e.to_string())?;

        // Read the pixel data from the rendered surface

        let text_surface_canvas = surface.into_canvas()?;
        let text_surface_canvas_size = text_surface_canvas.output_size()?;

        let width = text_surface_canvas_size.0;
        let height = text_surface_canvas_size.1;

        let bytes = text_surface_canvas.read_pixels(None, sdl2::pixels::PixelFormatEnum::Index8)?;

        let buffer = Buffer2D::from_data(width, height, bytes);

        Ok((width, height, buffer))
    }
}
