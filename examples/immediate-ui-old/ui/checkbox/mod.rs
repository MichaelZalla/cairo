use std::{cell::RefMut, collections::hash_map::Entry};

use cairo::{
    buffer::Buffer2D,
    device::mouse::MouseState,
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

static CHECKBOX_LABEL_PADDING: u32 = 8;

#[derive(Default, Debug)]
pub struct CheckboxOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
}

#[derive(Default, Debug)]
pub struct DoCheckboxResult {
    pub is_down: bool,
    pub was_released: bool,
    pub is_checked: bool,
}

pub fn do_checkbox(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    options: &CheckboxOptions,
    model_entry: Entry<'_, String, bool>,
) -> DoCheckboxResult {
    let id = UIID {
        item: ctx.next_id(),
    };

    {
        let mut font_cache = ctx.font_cache.borrow_mut();
        let mut text_cache = ctx.text_cache.borrow_mut();

        cache_text(
            &mut font_cache,
            &mut text_cache,
            &ctx.font_info,
            &options.label,
        );
    }

    let label_texture_width: u32;
    let label_texture_height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.borrow();

        let texture = text_cache.get(&text_cache_key).unwrap();

        label_texture_width = texture.0.width;
        label_texture_height = texture.0.height;
    }

    // Check whether a mouse event occurred inside this checkbox.

    let checkbox_size = label_texture_height;

    let (layout_offset_x, layout_offset_y) = options
        .layout_options
        .get_layout_offset(layout, checkbox_size);

    let checkbox_size = label_texture_height;

    let item_width = checkbox_size + CHECKBOX_LABEL_PADDING + label_texture_width;
    let item_height = label_texture_height;

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

    // Updates the state of our checkbox model, if needed.

    let mut is_checked = match &model_entry {
        Entry::Occupied(occupied_entry) => *(occupied_entry.get()),
        Entry::Vacant(_) => false,
    };

    if was_released {
        // Toggle the model values.

        is_checked = !is_checked;

        model_entry.and_modify(|value| {
            *value = is_checked;
        });
    }

    let result = DoCheckboxResult {
        is_down,
        was_released,
        is_checked,
    };

    // Render an unchecked or checked checkbox.

    layout.prepare_cursor(layout_offset_x + item_width, layout_offset_y + item_height);

    draw_checkbox(
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

    result
}

#[allow(clippy::too_many_arguments)]
fn draw_checkbox(
    ctx: &mut RefMut<'_, UIContext>,
    id: &UIID,
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    text_cache_key: &TextCacheKey,
    options: &CheckboxOptions,
    parent_buffer: &mut Buffer2D,
    result: &DoCheckboxResult,
) {
    let text_cache = ctx.text_cache.borrow();

    let texture = text_cache.get(text_cache_key).unwrap();

    let checkbox_size = texture.0.height;

    let theme = ctx.get_theme();

    let label_color = if result.is_down {
        theme.text_pressed
    } else if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    let cursor = layout.get_cursor();

    // Draw the checkbox borders.

    let (checkbox_x, checkbox_y) = (cursor.x + layout_offset_x, cursor.y + layout_offset_y);

    Graphics::rectangle(
        parent_buffer,
        checkbox_x,
        checkbox_y,
        checkbox_size,
        checkbox_size,
        Some(theme.checkbox_background.to_u32()),
        None,
    );

    let checkbox_top_left = (checkbox_x, checkbox_y);
    let checkbox_top_right = (checkbox_x + checkbox_size - 1, checkbox_y);
    let checkbox_bottom_left = (checkbox_x, checkbox_y + checkbox_size - 1);
    let checkbox_bottom_right = (
        checkbox_x + checkbox_size - 1,
        checkbox_y + checkbox_size - 1,
    );

    // Draw the checkbox check, if needed.

    if result.is_checked {
        Graphics::line(
            parent_buffer,
            checkbox_top_left.0 as i32,
            checkbox_top_left.1 as i32,
            checkbox_bottom_right.0 as i32,
            checkbox_bottom_right.1 as i32,
            theme.text.to_u32(),
        );
        Graphics::line(
            parent_buffer,
            checkbox_top_right.0 as i32,
            checkbox_top_right.1 as i32,
            checkbox_bottom_left.0 as i32,
            checkbox_bottom_left.1 as i32,
            theme.text.to_u32(),
        );
    }

    // Draw the checkbox label.

    let op = TextOperation {
        text: &options.label,
        x: checkbox_top_right.0 + CHECKBOX_LABEL_PADDING,
        y: checkbox_top_right.1,
        color: label_color,
    };

    Graphics::blit_text_from_mask(texture, &op, parent_buffer, None)
}
