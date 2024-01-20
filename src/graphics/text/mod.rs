use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::color::Color;

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

        // Copy the rendered pixels to the graphics buffer, with padding

        for y in 0..text_canvas_height {
            for x in 0..text_canvas_width {
                let text_surface_pixels_index =
                    (x as usize + y as usize * text_canvas_width as usize) * 4;

                let a = text_surface_pixels[text_surface_pixels_index + 3];

                if a != 0 {
                    self.buffer.set_pixel(
                        op.x + x,
                        op.y + y,
                        Color {
                            r: text_surface_pixels[text_surface_pixels_index],
                            g: text_surface_pixels[text_surface_pixels_index + 1],
                            b: text_surface_pixels[text_surface_pixels_index + 2],
                            a,
                        },
                    )
                }
            }
        }

        Ok(())
    }
}
