use std::cell::RefMut;

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
    layout::{item::ItemLayoutOptions, UILayoutContext},
};

static BORDERED_BUTTON_LABEL_PADDING_VERTICAL: u32 = 4;
static BORDERED_BUTTON_LABEL_PADDING_HORIZONTAL: u32 = 8;

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
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    options: &ButtonOptions,
) -> DoButtonResult {
    let id = UIID {
        item: ctx.next_id(),
    };

    cache_text(
        ctx.font_cache,
        ctx.text_cache,
        &ctx.font_info,
        &options.label,
    );

    let label_texture_width: u32;
    let label_texture_height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.borrow();

        let texture = text_cache.get(&text_cache_key).unwrap();

        label_texture_width = texture.width;
        label_texture_height = texture.height;
    }

    let (layout_offset_x, layout_offset_y) = options
        .layout_options
        .get_layout_offset(layout, label_texture_width);

    let item_width = label_texture_width
        + if options.with_border {
            BORDERED_BUTTON_LABEL_PADDING_HORIZONTAL * 2
        } else {
            0
        };
    let item_height = label_texture_height
        + if options.with_border {
            BORDERED_BUTTON_LABEL_PADDING_VERTICAL * 2
        } else {
            0
        };

    // Check whether a mouse event occurred inside this button.

    let (is_down, was_released) = get_mouse_result(
        ctx,
        &id,
        layout,
        mouse_state,
        layout_offset_x,
        layout_offset_y,
        item_width,
        item_height,
    );

    let result = DoButtonResult {
        is_down,
        was_released,
    };

    // Render an unpressed or pressed button.

    layout.prepare_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

    draw_button(
        ctx,
        &id,
        layout,
        layout_offset_x,
        layout_offset_y,
        &text_cache_key,
        options,
        parent_buffer,
        &result,
    );

    layout.advance_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

    DoButtonResult {
        is_down,
        was_released,
    }
}

fn draw_button(
    ctx: &mut RefMut<'_, UIContext>,
    id: &UIID,
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    text_cache_key: &TextCacheKey,
    options: &ButtonOptions,
    parent_buffer: &mut Buffer2D,
    result: &DoButtonResult,
) {
    let theme = ctx.get_theme();

    let cursor = layout.get_cursor();

    let text_cache = ctx.text_cache.borrow();

    let texture = text_cache.get(text_cache_key).unwrap();

    if options.with_border {
        Graphics::rectangle(
            parent_buffer,
            cursor.x + layout_offset_x,
            cursor.y + layout_offset_y,
            texture.width + BORDERED_BUTTON_LABEL_PADDING_HORIZONTAL * 2,
            texture.height + BORDERED_BUTTON_LABEL_PADDING_VERTICAL * 2,
            Some(theme.button_background),
            None,
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
        x: cursor.x
            + layout_offset_x
            + if options.with_border {
                BORDERED_BUTTON_LABEL_PADDING_HORIZONTAL
            } else {
                0
            },
        y: cursor.y
            + layout_offset_y
            + if options.with_border {
                BORDERED_BUTTON_LABEL_PADDING_VERTICAL
            } else {
                0
            },
        color: text_color,
        text: &options.label,
    };

    Graphics::blit_text_from_mask(texture, &op, parent_buffer, None);
}
