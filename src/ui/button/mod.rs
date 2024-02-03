use std::sync::RwLockWriteGuard;

use crate::{
    buffer::Buffer2D,
    device::MouseState,
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
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    options: &ButtonOptions,
) -> DoButtonResult {
    cache_text(
        ctx.font_cache,
        ctx.text_cache,
        ctx.font_info,
        &options.label,
    );

    let width: u32;
    let height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.read().unwrap();

        let texture = text_cache.get(&text_cache_key).unwrap();

        width = texture.width;
        height = texture.height;
    }

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, width);

    // Check whether a mouse event occurred inside this button.

    let (is_down, was_released) =
        get_mouse_result(ctx, id, panel_info, mouse_state, x, y, width, height);

    let result = DoButtonResult {
        is_down,
        was_released,
    };

    // Render an unpressed or pressed button.

    draw_button(
        ctx,
        id,
        panel_buffer,
        x,
        y,
        &text_cache_key,
        options,
        &result,
    );

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
    text_cache_key: &TextCacheKey,
    options: &ButtonOptions,
    result: &DoButtonResult,
) {
    let theme = ctx.get_theme();

    let text_cache = ctx.text_cache.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    if options.with_border {
        Graphics::rectangle(
            panel_buffer,
            x,
            y,
            texture.width,
            texture.height,
            theme.button_background,
            Some(theme.button_background),
        )
    }

    // Draw the button's text label.

    let text_color = if result.is_down {
        theme.text_pressed
    } else if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    let op = TextOperation {
        x,
        y,
        color: text_color,
        text: &options.label,
    };

    Graphics::blit_text_from_mask(texture, &op, panel_buffer, None);
}
