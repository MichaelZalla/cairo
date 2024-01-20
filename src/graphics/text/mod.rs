use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{color::Color, texture::TextureBuffer};

use super::Graphics;

#[derive(Clone)]
pub struct TextOperation<'a> {
    pub text: &'a String,
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

impl Graphics {
    pub fn text(&mut self, font: &Font, op: &TextOperation) -> Result<(), String> {
        // Generate a texture for this text operation.

        let (width, height, bytes) = self.make_text_texture(font, op).unwrap();

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

                self.buffer.set_pixel(
                    start_x + x,
                    start_y + y,
                    Color {
                        r: bytes[index],
                        g: bytes[index + 1],
                        b: bytes[index + 2],
                        a,
                    },
                )
            }
        }

        Ok(())
    }

    fn make_text_texture(
        &mut self,
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
}
