use std::borrow::BorrowMut;

use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{
    buffer::Buffer2D,
    color::{self, Color},
    debug::message::DebugMessageBuffer,
    font::{cache::FontCache, FontInfo},
    texture::TextureBuffer,
};

use super::Graphics;

#[derive(Clone)]
pub struct TextOperation<'a> {
    pub text: &'a String,
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

impl Graphics {
    pub fn text(dest_buffer: &mut Buffer2D, font: &Font, op: &TextOperation) -> Result<(), String> {
        // Generate a texture for this text operation.

        let (width, height, src_buffer) = Graphics::make_text_texture(font, op).unwrap();

        // Copy the rendered pixels to this buffer, at location (op.x, op.y).

        Graphics::blit_u8_to_u32(&src_buffer, op.x, op.y, width, height, dest_buffer);

        Ok(())
    }

    pub fn blit_u8_to_u32(
        src_buffer: &Buffer2D<u8>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        dest_buffer: &mut Buffer2D<u32>,
    ) {
        if x >= dest_buffer.width {
            return;
        }

        if y >= dest_buffer.height {
            return;
        }

        for y_rel in 0..height.min(dest_buffer.height - y) {
            for x_rel in 0..width.min(dest_buffer.width - x) {
                let index = (x_rel as usize + y_rel as usize * width as usize) * 4;

                let a = src_buffer.data[index + 3];

                if a == 0 {
                    continue;
                }

                let value = Color {
                    r: src_buffer.data[index],
                    g: src_buffer.data[index + 1],
                    b: src_buffer.data[index + 2],
                    a,
                }
                .to_u32();

                dest_buffer.set(x + x_rel, y + y_rel, value)
            }
        }
    }

    pub fn render_debug_messages(
        buffer: &mut Buffer2D,
        font_cache: &mut FontCache,
        font_info: &FontInfo,
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

            Graphics::text_using_font_cache(buffer, font_cache, &font_info, &op).unwrap();

            y_offset += (font_info.point_size as f32 * padding_ems) as u32;
        }

        debug_messages.drain();
    }

    pub fn make_text_texture(
        font: &Font,
        op: &TextOperation,
    ) -> Result<(u32, u32, TextureBuffer), String> {
        // Generate a new text rendering (surface)

        let surface = font
            .render(op.text)
            .blended(SDLColor::RGBA(
                op.color.r, op.color.g, op.color.b, op.color.a,
            ))
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

    fn text_using_font_cache(
        buffer: &mut Buffer2D,
        font_cache: &mut FontCache,
        font_info: &FontInfo,
        text_operation: &TextOperation,
    ) -> Result<(), String> {
        match (*font_cache).load(font_info) {
            Ok(font) => Graphics::text(buffer, &font, text_operation),
            Err(e) => return Err(e.to_string()),
        }
    }
}
