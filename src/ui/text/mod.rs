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
    layout::{item::ItemLayoutOptions, UILayoutContext},
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
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    options: &TextOptions,
) -> DoTextResult {
    let item_width: u32;
    let item_height: u32;
    let layout_offset_x: u32;
    let layout_offset_y: u32;

    match options.cache {
        true => {
            cache_text(ctx.font_cache, ctx.text_cache, ctx.font_info, &options.text);

            let text_cache_key = TextCacheKey {
                font_info: ctx.font_info.clone(),
                text: options.text.clone(),
            };

            let text_cache = ctx.text_cache.read().unwrap();

            let texture_ref = text_cache.get(&text_cache_key).unwrap();

            (layout_offset_x, layout_offset_y) = options
                .layout_options
                .get_layout_offset(layout, texture_ref.width);

            item_width = texture_ref.width;
            item_height = texture_ref.height;

            layout.prepare_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

            draw_text(
                layout,
                layout_offset_x,
                layout_offset_y,
                texture_ref,
                options,
                parent_buffer,
            );
        }
        false => {
            let mut font_cache = ctx.font_cache.write().unwrap();

            let font = font_cache.load(ctx.font_info).unwrap();

            let (_label_width, _label_height, texture) =
                Graphics::make_text_texture(font.as_ref(), &options.text).unwrap();

            (layout_offset_x, layout_offset_y) = options
                .layout_options
                .get_layout_offset(layout, texture.width);

            item_width = texture.width;
            item_height = texture.height;

            layout.prepare_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

            draw_text(
                layout,
                layout_offset_x,
                layout_offset_y,
                &texture,
                options,
                parent_buffer,
            );
        }
    }

    layout.advance_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

    DoTextResult {}
}

fn draw_text(
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    texture: &Buffer2D<u8>,
    options: &TextOptions,
    parent_buffer: &mut Buffer2D,
) {
    let cursor = layout.get_cursor();

    let color = options.color;

    // Draw the button's text label.

    let op = TextOperation {
        x: cursor.x + layout_offset_x,
        y: cursor.y + layout_offset_y,
        color,
        text: &options.text,
    };

    Graphics::blit_text_from_mask(texture, &op, parent_buffer, None);
}
