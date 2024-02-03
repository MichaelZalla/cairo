use std::sync::RwLockWriteGuard;

use crate::{
    buffer::Buffer2D,
    color::Color,
    graphics::{
        text::{
            cache::{cache_text, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
};

use super::{
    context::{UIContext, UIID},
    layout::item::ItemLayoutOptions,
    panel::PanelInfo,
};

#[derive(Debug)]
pub struct TextOptions {
    pub layout_options: ItemLayoutOptions,
    pub text: String,
    pub cache: bool,
    pub color: Color,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            layout_options: Default::default(),
            text: Default::default(),
            cache: true,
            color: Default::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct DoTextResult {}

pub fn do_text(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    _id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    options: &TextOptions,
) -> DoTextResult {
    match options.cache {
        true => {
            cache_text(ctx.font_cache, ctx.text_cache, ctx.font_info, &options.text);

            let text_cache_key = TextCacheKey {
                font_info: ctx.font_info.clone(),
                text: options.text.clone(),
            };

            let text_cache = ctx.text_cache.read().unwrap();

            let texture_ref = text_cache.get(&text_cache_key).unwrap();

            let (x, y) = options
                .layout_options
                .get_top_left_within_parent(panel_info, texture_ref.width);

            draw_text(panel_buffer, x, y, texture_ref, options);
        }
        false => {
            let mut font_cache = ctx.font_cache.write().unwrap();

            let font = font_cache.load(ctx.font_info).unwrap();

            let (_label_width, _label_height, texture) =
                Graphics::make_text_texture(font.as_ref(), &options.text).unwrap();

            let (x, y) = options
                .layout_options
                .get_top_left_within_parent(panel_info, texture.width);

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

    Graphics::blit_text_from_mask(texture, &op, panel_buffer, None);
}
