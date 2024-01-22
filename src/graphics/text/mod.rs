use std::borrow::BorrowMut;

use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{
    color::{self, Color},
    debug::message::DebugMessageBuffer,
    font::{cache::FontCache, FontInfo},
    texture::TextureBuffer,
};

use super::{pixelbuffer::PixelBuffer, Graphics};

#[derive(Clone)]
pub struct TextOperation<'a> {
    pub text: &'a String,
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

impl Graphics {
    pub fn text(buffer: &mut PixelBuffer, font: &Font, op: &TextOperation) -> Result<(), String> {
        // Generate a texture for this text operation.

        let (width, height, bytes) = Graphics::make_text_texture(buffer, font, op).unwrap();

        // Copy the rendered pixels to this buffer, at location (op.x, op.y).

        let start_x = op.x;
        let start_y = op.y;

        for y in 0..height {
            for x in 0..width {
                let index = (x as usize + y as usize * width as usize) * 4;

                let a = bytes[index + 3];

                if a == 0 {
                    continue;
                }

                let value = Color {
                    r: bytes[index],
                    g: bytes[index + 1],
                    b: bytes[index + 2],
                    a,
                }
                .to_u32();

                buffer.set(start_x + x, start_y + y, value)
            }
        }

        Ok(())
    }

    pub fn render_debug_messages(
        buffer: &mut PixelBuffer,
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

    fn make_text_texture(
        buffer: &mut PixelBuffer,
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

        let text_canvas_width = text_surface_canvas_size.0;
        let text_canvas_height = text_surface_canvas_size.1;

        let text_surface_pixels =
            text_surface_canvas.read_pixels(None, sdl2::pixels::PixelFormatEnum::RGBA32)?;

        Ok((text_canvas_width, text_canvas_height, text_surface_pixels))
    }

    fn text_using_font_cache(
        buffer: &mut PixelBuffer,
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
