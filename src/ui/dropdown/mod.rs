use std::{
    collections::hash_map::Entry,
    sync::{RwLock, RwLockWriteGuard},
};

use crate::{
    buffer::Buffer2D,
    color,
    device::{MouseEventKind, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{
        text::{
            cache::{cache_text, TextCache, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
    vec::vec2::Vec2,
};

use super::{
    context::{UIContext, UIID},
    get_mouse_result,
    layout::ItemLayoutOptions,
    panel::PanelInfo,
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
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &DropdownOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoDropdownResult {
    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let label_texture = text_cache.get(&text_cache_key).unwrap();

    // Check whether a mouse event occurred inside this dropdown.

    let is_open = ctx.is_focused(id) && ctx.is_focus_target_open();

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, DROPDOWN_WIDTH);

    let dropdown_height = if is_open {
        label_texture.height
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
        label_texture.height
    };

    let (_is_down, was_released) = get_mouse_result(
        ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        DROPDOWN_WIDTH + DROPDOWN_LABEL_PADDING + label_texture.width,
        dropdown_height,
    );

    if was_released {
        if ctx.is_focused(id) {
            // Toggle the open vs. closed state of our menu.

            let is_open = ctx.is_focus_target_open();

            ctx.set_focus_target_open(!is_open);
        }
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
                match mouse_state.button_event {
                    Some(event) => {
                        match event.kind {
                            MouseEventKind::Down => {
                                let (mouse_x, mouse_y) = (
                                    mouse_state.position.0 - panel_info.x as i32,
                                    mouse_state.position.1 - panel_info.y as i32,
                                );

                                if mouse_x >= x as i32
                                    && mouse_x < (x + DROPDOWN_WIDTH) as i32
                                    && mouse_y > y as i32
                                    && mouse_y < (y + dropdown_height) as i32
                                {
                                    let relative_mouse_y =
                                        mouse_state.position.1 as u32 - panel_info.y;

                                    let mut target_item_index: i32 = -1;

                                    let mut current_y = y;

                                    while current_y < relative_mouse_y {
                                        target_item_index += 1;

                                        current_y +=
                                            label_texture.height + DROPDOWN_ITEM_VERTICAL_PADDING;
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
                    None => (),
                }
            }
        }
        Entry::Vacant(_v) => {
            // Ignore this click.
        }
    }

    let result = DoDropdownResult { did_edit };

    // Render a dropdown.

    draw_dropdown(
        ctx,
        id,
        panel_buffer,
        font_cache_rwl,
        font_info,
        x,
        y,
        is_open,
        dropdown_height,
        options,
        current_item,
        label_texture,
    );

    result
}

fn draw_dropdown(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    font_info: &'static FontInfo,
    x: u32,
    y: u32,
    is_open: bool,
    height: u32,
    options: &DropdownOptions,
    current_item: String,
    label_texture: &Buffer2D<u8>,
) {
    let theme = ctx.get_theme();

    let border_color = if ctx.is_focused(id) {
        theme.border_focus
    } else if ctx.is_hovered(id) {
        theme.border_hover
    } else {
        theme.border
    };

    let text_color = if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    // Draw the dropdown borders.

    Graphics::rectangle(
        panel_buffer,
        x,
        y,
        DROPDOWN_WIDTH,
        height,
        border_color,
        None,
    );

    // Draw the dropdown carat if needed.

    static CARAT_MARGIN_RIGHT: f32 = 5.0;
    static CARAT_WIDTH: f32 = 10.0;
    static CARAT_HEIGHT: f32 = CARAT_WIDTH / 2.0;

    let carat_top_left = Vec2 {
        x: (x + DROPDOWN_WIDTH - 1) as f32 - CARAT_MARGIN_RIGHT - CARAT_WIDTH,
        y: (y + (height / 2)) as f32 - CARAT_HEIGHT / 2.0,
        z: 0.0,
    };

    let carat_top_right = Vec2 {
        x: (x + DROPDOWN_WIDTH - 1) as f32 - CARAT_MARGIN_RIGHT,
        y: (y + (height / 2)) as f32 - CARAT_HEIGHT / 2.0,
        z: 0.0,
    };

    let mut carat_bottom_mid = carat_top_left + (carat_top_right - carat_top_left) / 2.0;
    carat_bottom_mid.y += CARAT_HEIGHT;

    let carat_points = [carat_top_left, carat_bottom_mid, carat_top_right];

    if !is_open {
        Graphics::poly_line(panel_buffer, &carat_points, theme.text);
    }

    let dropdown_top_left = (x, y);
    let dropdown_top_right = (x + DROPDOWN_WIDTH - 1, y);

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

        let mut font_cache = font_cache_rwl.write().unwrap();

        let font = font_cache.load(font_info).unwrap();

        let (_label_width, _label_height, model_value_texture) =
            Graphics::make_text_texture(font.as_ref(), &item).unwrap();

        let max_width = DROPDOWN_WIDTH - DROPDOWN_LABEL_PADDING;

        Graphics::blit_text_from_mask(
            &model_value_texture,
            &TextOperation {
                text: &item,
                x,
                y: y + 1,
                color: if is_open && *item == current_item {
                    color::WHITE
                } else {
                    text_color
                },
            },
            panel_buffer,
            Some(max_width),
        );

        y += model_value_texture.height + DROPDOWN_ITEM_VERTICAL_PADDING;
    }

    // Draw the textbox label.

    let op = TextOperation {
        text: &options.label,
        x: dropdown_top_right.0 + DROPDOWN_LABEL_PADDING,
        y: dropdown_top_right.1,
        color: text_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, panel_buffer, None)
}
