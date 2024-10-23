use sdl2::{pixels::Color as SDLColor, ttf::Font};

use crate::{
    animation::lerp,
    buffer::Buffer2D,
    color::{self, Color},
    debug::message::DebugMessageBuffer,
    font::{cache::FontCache, FontInfo},
    texture::map::TextureBuffer,
};

use self::cache::{cache_text, TextCache, TextCacheKey, TextMask};

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
    pub fn text(
        target: &mut Buffer2D,
        font_cache: &mut FontCache,
        text_cache: Option<&mut TextCache>,
        font_info: &FontInfo,
        op: &TextOperation,
    ) -> Result<(), String> {
        // Generate a texture for this text operation.

        let text_cache_key = TextCacheKey {
            font_info: font_info.clone(),
            text: op.text.clone(),
        };

        match text_cache {
            Some(text_cache) => {
                cache_text(font_cache, text_cache, font_info, op.text);

                let cached_text_mask = text_cache.get(&text_cache_key).unwrap();

                Graphics::blit_text_from_mask(cached_text_mask, op, target, None);
            }
            None => {
                let font = font_cache.load(font_info).unwrap();

                let (_label_width, _label_height, text_mask) =
                    Graphics::make_text_mask(font.as_ref(), op.text).unwrap();

                println!("Generated text mask for text '{}' (uncached).", op.text);

                Graphics::blit_text_from_mask(&text_mask, op, target, None);
            }
        }

        Ok(())
    }

    pub fn blit_text_from_mask(
        mask: &TextMask,
        op: &TextOperation,
        target: &mut Buffer2D<u32>,
        max_width: Option<u32>,
    ) {
        if op.x >= target.width {
            return;
        }

        if op.y >= target.height {
            return;
        }

        let available_height = mask.0.height.min(target.height - op.y);

        let available_width =
            mask.0
                .width
                .min(target.width - op.x)
                .min(if let Some(width) = max_width {
                    width
                } else {
                    u32::MAX
                });

        for y_rel in 0..available_height {
            for x_rel in 0..available_width {
                let mask_pixel_index = y_rel as usize * mask.0.width as usize + x_rel as usize;

                let alpha = mask.0.data[mask_pixel_index];

                if alpha == 0.0 {
                    // Skips unrendered pixels in our text texture (mask).

                    continue;
                }

                let (x, y) = (op.x + x_rel, op.y + y_rel);

                let start = Color::from_u32(*target.get(x, y)).to_vec3();
                let end = op.color.to_vec3();

                let blended = lerp(start, end, alpha);
                let blended_u32 = Color::from_vec3(blended).to_u32();

                target.set(x, y, blended_u32)
            }
        }
    }

    pub fn render_debug_messages(
        target: &mut Buffer2D,
        font_cache: &mut FontCache,
        font_info: &FontInfo,
        position: (u32, u32),
        padding_ems: f32,
        mut debug_messages: &mut DebugMessageBuffer,
    ) {
        let mut y_offset = position.1;

        for msg in &mut debug_messages {
            let op = TextOperation {
                text: &msg,
                x: position.0,
                y: y_offset,
                color: color::WHITE,
            };

            Graphics::text(target, font_cache, None, font_info, &op).unwrap();

            y_offset += (font_info.point_size as f32 * padding_ems) as u32;
        }

        debug_messages.drain();
    }

    pub fn make_text_mask(font: &Font, text: &str) -> Result<(u32, u32, TextMask), String> {
        // Generate a new text texture (mask).

        let surface = font
            .render(text)
            .blended(SDLColor::WHITE)
            .map_err(|e| e.to_string())?;

        // Read the pixel data from the rendered surface

        let text_surface_canvas = surface.into_canvas()?;
        let text_surface_canvas_size = text_surface_canvas.output_size()?;

        let width = text_surface_canvas_size.0;
        let height = text_surface_canvas_size.1;
        let pixel_count = (width * height) as usize;

        static RGBA8888_CHANNEL_COUNT: usize = 4;

        let rgba8888 =
            text_surface_canvas.read_pixels(None, sdl2::pixels::PixelFormatEnum::RGBA8888)?;

        let mut alpha = vec![0.0; pixel_count];

        for (pixel_index, alpha_value) in alpha.iter_mut().enumerate() {
            let a = rgba8888[pixel_index * RGBA8888_CHANNEL_COUNT];

            *alpha_value = a as f32 / 255.0;
        }

        let buffer = Buffer2D::from_data(width, height, alpha);

        Ok((width, height, TextureBuffer(buffer)))
    }
}
