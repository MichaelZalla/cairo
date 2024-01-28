use std::sync::RwLock;

use crate::{
    buffer::Buffer2D,
    color::Color,
    font::{cache::FontCache, FontInfo},
    graphics::{
        text::{
            cache::{cache_text, TextCache, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
};

use super::panel::PanelInfo;

#[derive(Debug)]
pub struct TextOptions {
    pub x_offset: u32,
    pub y_offset: u32,
    pub text: String,
    pub cache: bool,
    pub color: Color,
    pub align_right: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            x_offset: Default::default(),
            y_offset: Default::default(),
            text: Default::default(),
            cache: true,
            color: Default::default(),
            align_right: Default::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct DoTextResult {}

pub fn do_text(
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &TextOptions,
) -> DoTextResult {
    let get_x_y = |texture: &Buffer2D<u8>| {
        let x = if options.align_right {
            panel_info.width - texture.width - options.x_offset
        } else {
            options.x_offset
        };

        let y = options.y_offset;

        (x, y)
    };

    match options.cache {
        true => {
            cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.text);

            let text_cache_key = TextCacheKey {
                font_info,
                text: options.text.clone(),
            };

            let text_cache = text_cache_rwl.read().unwrap();

            let texture_ref = text_cache.get(&text_cache_key).unwrap();

            let (x, y) = get_x_y(texture_ref);

            draw_text(panel_buffer, x, y, texture_ref, options);
        }
        false => {
            let mut font_cache = font_cache_rwl.write().unwrap();

            let font = font_cache.load(font_info).unwrap();

            let (_label_width, _label_height, texture) =
                Graphics::make_text_texture(font.as_ref(), &options.text).unwrap();

            let (x, y) = get_x_y(&texture);

            draw_text(panel_buffer, x, y, &texture, options);
        }
    }

    DoTextResult {}
}

fn draw_text(
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    texture: &Buffer2D<u8>,
    options: &TextOptions,
) {
    let color = options.color;

    // Draw the button's text label.

    let op = TextOperation {
        x,
        y,
        color,
        text: &options.text,
    };

    Graphics::blit_text_from_mask(texture, &op, panel_buffer);
}
