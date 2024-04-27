use std::{cell::RefMut, collections::hash_map::Entry};

use crate::{
    buffer::Buffer2D,
    color,
    device::{MouseEventKind, MouseState},
    graphics::{
        text::{
            cache::{cache_text, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
    vec::vec2::Vec2,
};

use super::{
    context::{UIContext, UIID},
    get_mouse_result,
    layout::{item::ItemLayoutOptions, UILayoutContext},
};

static DROPDOWN_WIDTH: u32 = 200;
static DROPDOWN_LABEL_PADDING: u32 = 8;
static DROPDOWN_ITEM_HORIZONTAL_PADDING: u32 = 4;
static DROPDOWN_ITEM_VERTICAL_PADDING: u32 = 4;

#[derive(Default, Debug)]
pub struct DropdownOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
    pub items: Vec<String>,
}

#[derive(Default, Debug)]
pub struct DoDropdownResult {
    pub did_edit: bool,
}

pub fn do_dropdown(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    options: &DropdownOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoDropdownResult {
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

        let label_texture = text_cache.get(&text_cache_key).unwrap();

        label_texture_width = label_texture.width;
        label_texture_height = label_texture.height;
    }

    // Check whether a mouse event occurred inside this dropdown.

    let is_open = ctx.is_focused(&id) && ctx.is_focus_target_open();

    let (layout_offset_x, layout_offset_y) = options
        .layout_options
        .get_layout_offset(layout, DROPDOWN_WIDTH);

    let item_width = DROPDOWN_WIDTH + DROPDOWN_LABEL_PADDING + label_texture_width;

    let item_height = if is_open {
        label_texture_height
            * if is_open {
                options.items.len() as u32
            } else {
                1
            }
            + if is_open {
                DROPDOWN_ITEM_VERTICAL_PADDING * options.items.len() as u32 - 1
            } else {
                0
            }
    } else {
        label_texture_height
    };

    let (_is_down, was_released) = get_mouse_result(
        ctx,
        &id,
        layout,
        mouse_state,
        layout_offset_x,
        layout_offset_y,
        item_width,
        item_height,
    );

    if was_released && ctx.is_focused(&id) {
        // Toggle the open vs. closed state of our menu.

        let is_open = ctx.is_focus_target_open();

        ctx.set_focus_target_open(!is_open);
    }

    // Updates the state of our textbox model, if needed.

    let mut did_edit = false;
    let mut current_item: String = "".to_string();

    match &mut model_entry {
        Entry::Occupied(o) => {
            {
                current_item = o.get().clone();
            }

            // Check if we've selected an option from the open menu.

            if is_open {
                if let Some(event) = mouse_state.button_event {
                    match event.kind {
                        MouseEventKind::Down => {
                            let cursor = layout.get_cursor();

                            let (mouse_x, mouse_y) = (
                                mouse_state.position.0 - cursor.x as i32,
                                mouse_state.position.1 - cursor.y as i32,
                            );

                            if mouse_x >= layout_offset_x as i32
                                && mouse_x < (layout_offset_x + DROPDOWN_WIDTH) as i32
                                && mouse_y > layout_offset_y as i32
                                && mouse_y < (layout_offset_y + item_height) as i32
                            {
                                let relative_mouse_y = mouse_state.position.1 as u32 - cursor.y;

                                let mut target_item_index: i32 = -1;

                                let mut current_y = layout_offset_y;

                                while current_y < relative_mouse_y {
                                    target_item_index += 1;

                                    current_y +=
                                        label_texture_height + DROPDOWN_ITEM_VERTICAL_PADDING;
                                }

                                let target_item = &options.items[target_item_index as usize];

                                if *target_item != current_item {
                                    did_edit = true;

                                    *o.get_mut() = (*target_item).clone();
                                }
                            }
                        }
                        MouseEventKind::Up => {
                            // Do nothing
                        }
                    }
                }
            }
        }
        Entry::Vacant(_v) => {
            // Ignore this click.
        }
    }

    let result = DoDropdownResult { did_edit };

    // Render a dropdown.

    layout.prepare_cursor(item_width, item_height);

    draw_dropdown(
        ctx,
        &id,
        layout,
        layout_offset_x,
        layout_offset_y,
        &text_cache_key,
        is_open,
        item_height,
        options,
        current_item,
        parent_buffer,
    );

    layout.advance_cursor(item_width, item_height);

    result
}

fn draw_dropdown(
    ctx: &mut RefMut<'_, UIContext>,
    id: &UIID,
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    text_cache_key: &TextCacheKey,
    is_open: bool,
    item_height: u32,
    options: &DropdownOptions,
    current_item: String,
    parent_buffer: &mut Buffer2D,
) {
    let theme = ctx.get_theme();

    let cursor = layout.get_cursor();

    let text_cache = ctx.text_cache.borrow();

    let label_texture = text_cache.get(text_cache_key).unwrap();

    let label_color = if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    // Draw the dropdown borders.

    let (dropdown_x, dropdown_y) = (cursor.x + layout_offset_x, cursor.y + layout_offset_y);

    Graphics::rectangle(
        parent_buffer,
        dropdown_x,
        dropdown_y,
        DROPDOWN_WIDTH,
        item_height,
        Some(theme.dropdown_background),
        None,
    );

    // Draw the dropdown carat if needed.

    static CARAT_MARGIN_RIGHT: f32 = 5.0;
    static CARAT_WIDTH: f32 = 10.0;
    static CARAT_HEIGHT: f32 = CARAT_WIDTH / 2.0;

    let carat_top_left = Vec2 {
        x: (dropdown_x + DROPDOWN_WIDTH - 1) as f32 - CARAT_MARGIN_RIGHT - CARAT_WIDTH,
        y: (dropdown_y + (item_height / 2)) as f32 - CARAT_HEIGHT / 2.0,
        z: 0.0,
    };

    let carat_top_right = Vec2 {
        x: (dropdown_x + DROPDOWN_WIDTH - 1) as f32 - CARAT_MARGIN_RIGHT,
        y: (dropdown_y + (item_height / 2)) as f32 - CARAT_HEIGHT / 2.0,
        z: 0.0,
    };

    let mut carat_bottom_mid = carat_top_left + (carat_top_right - carat_top_left) / 2.0;
    carat_bottom_mid.y += CARAT_HEIGHT;

    let carat_points = [carat_top_left, carat_bottom_mid, carat_top_right];

    if !is_open {
        Graphics::poly_line(parent_buffer, &carat_points, theme.text);
    }

    let dropdown_top_left = (dropdown_x, dropdown_y);
    let dropdown_top_right = (dropdown_x + DROPDOWN_WIDTH - 1, dropdown_y);

    // Draw the dropdown model value (text), or the open menu items.

    let (x, mut y) = (
        dropdown_top_left.0 + DROPDOWN_ITEM_HORIZONTAL_PADDING,
        dropdown_top_left.1,
    );

    for item in &options.items {
        // Ignore other items if the dropdown is closed.

        if !is_open && *item != current_item {
            continue;
        }

        // Draw the item text.

        let mut font_cache = ctx.font_cache.borrow_mut();

        let font = font_cache.load(&ctx.font_info).unwrap();

        let (_label_width, _label_height, model_value_texture) =
            Graphics::make_text_mask(font.as_ref(), item).unwrap();

        let max_width = DROPDOWN_WIDTH - DROPDOWN_LABEL_PADDING;

        if is_open && *item == current_item {
            Graphics::rectangle(
                parent_buffer,
                x - DROPDOWN_ITEM_HORIZONTAL_PADDING / 2 + 1,
                y + 1,
                max_width + DROPDOWN_ITEM_HORIZONTAL_PADDING / 2 * 2 - 2,
                model_value_texture.0.height,
                Some(color::BLUE),
                None,
            );
        }

        Graphics::blit_text_from_mask(
            &model_value_texture.0,
            &TextOperation {
                text: item,
                x,
                y: y + 1,
                color: if is_open && *item == current_item {
                    color::WHITE
                } else {
                    label_color
                },
            },
            parent_buffer,
            Some(max_width),
        );

        y += model_value_texture.0.height + DROPDOWN_ITEM_VERTICAL_PADDING;
    }

    // Draw the textbox label.

    let op = TextOperation {
        text: &options.label,
        x: dropdown_top_right.0 + DROPDOWN_LABEL_PADDING,
        y: dropdown_top_right.1,
        color: label_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, parent_buffer, None)
}
