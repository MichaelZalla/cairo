use std::sync::{RwLock, RwLockWriteGuard};

use crate::{
    buffer::Buffer2D,
    color::{self},
    device::MouseState,
    font::{cache::FontCache, FontInfo},
    graphics::{
        text::{
            cache::{cache_text, TextCache, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
};

use super::{
    context::{UIContext, UIID},
    get_mouse_result,
    layout::ItemLayoutOptions,
    panel::PanelInfo,
};

#[derive(Default, Debug)]
pub struct ButtonOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
    pub with_border: bool,
}

#[derive(Default, Debug)]
pub struct DoButtonResult {
    pub is_down: bool,
    pub was_released: bool,
}

pub fn do_button(
    ui_context: &'static RwLock<UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &ButtonOptions,
) -> DoButtonResult {
    let mut ctx: RwLockWriteGuard<'_, UIContext> = ui_context.write().unwrap();

    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, texture.width);

    // Check whether a mouse event occurred inside this button.

    let (is_down, was_released) = get_mouse_result(
        &mut ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        texture.width,
        texture.height,
    );

    let result = DoButtonResult {
        is_down,
        was_released,
    };

    // Render an unpressed or pressed button.

    draw_button(&mut ctx, id, panel_buffer, x, y, texture, options, &result);

    DoButtonResult {
        is_down,
        was_released,
    }
}

fn draw_button(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    texture: &Buffer2D<u8>,
    options: &ButtonOptions,
    result: &DoButtonResult,
) {
    let color = if result.is_down {
        color::GREEN
    } else if ctx.is_hovered(id) {
        color::WHITE
    } else {
        color::YELLOW
    };

    // Draw the button's text label.

    let op = TextOperation {
        x,
        y,
        color,
        text: &options.label,
    };

    Graphics::blit_text_from_mask(texture, &op, panel_buffer, None);

    // Draw the button's border.

    if options.with_border {
        Graphics::rectangle(panel_buffer, x, y, texture.width, texture.height, color)
    }
}
