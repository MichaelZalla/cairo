use std::{collections::hash_map::Entry, sync::RwLock};

use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    color::{self},
    device::{MouseEventKind, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{
        text::{
            cache::{cache_text, TextCache, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
};

use super::{context::UIContext, panel::PanelInfo};

#[derive(Default, Debug)]
pub struct CheckboxOptions {
    pub x_offset: u32,
    pub y_offset: u32,
    pub label: String,
    pub align_right: bool,
}

#[derive(Default, Debug)]
pub struct DoCheckboxResult {
    pub is_down: bool,
    pub was_released: bool,
    pub is_checked: bool,
}

pub fn do_checkbox(
    _ui_context: &'static UIContext,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &CheckboxOptions,
    model: Entry<'_, String, bool>,
) -> DoCheckboxResult {
    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    let mut is_down: bool = false;
    let mut was_released: bool = false;

    // Check whether a mouse event occurred inside this checkbox.

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0 as u32, mouse_state.position.1 as u32);

    let checkbox_size = texture.height;

    let x = if options.align_right {
        panel_info.width - checkbox_size - options.x_offset
    } else {
        options.x_offset
    };

    let y = options.y_offset;

    if mouse_x >= panel_info.x && mouse_y >= panel_info.y {
        // Maps mouse_x and mouse_y into panel's local coordinates.

        mouse_x -= panel_info.x;
        mouse_y -= panel_info.y;

        if mouse_x as u32 >= x
            && mouse_x < x + checkbox_size
            && mouse_y >= y
            && mouse_y < y + checkbox_size
        {
            // Check whether LMB was pressed or released inside of this checkbox.

            match mouse_state.buttons_down.get(&MouseButton::Left) {
                Some(_) => {
                    is_down = true;
                }
                None => (),
            }

            match mouse_state.button_event {
                Some(event) => match event.button {
                    MouseButton::Left => match event.kind {
                        MouseEventKind::Up => {
                            was_released = true;
                        }
                        _ => (),
                    },
                    _ => (),
                },
                None => (),
            }
        }
    }

    let mut is_checked = match &model {
        Entry::Occupied(occupied_entry) => *(occupied_entry.get()),
        Entry::Vacant(_) => false,
    };

    if was_released {
        // Toggle our checkbox (model).

        is_checked = !is_checked;

        model.and_modify(|value| {
            *value = is_checked;
        });
    }

    let result = DoCheckboxResult {
        is_down,
        was_released,
        is_checked,
    };

    // Render an unchecked or checked checkbox.
    draw_checkbox(panel_buffer, x, y, options, texture, &result);

    result
}

fn draw_checkbox(
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    options: &CheckboxOptions,
    texture: &Buffer2D<u8>,
    result: &DoCheckboxResult,
) {
    let checkbox_size = texture.height;

    let color = if result.is_down {
        color::GREEN
    } else {
        color::YELLOW
    };

    // Draw the checkbox borders.

    Graphics::rectangle(panel_buffer, x, y, checkbox_size, checkbox_size, color);

    let checkbox_top_left = (x, y);
    let checkbox_top_right = (x + checkbox_size, y);
    let checkbox_bottom_left = (x, y + checkbox_size);
    let checkbox_bottom_right = (x + checkbox_size, y + checkbox_size);

    // Draw the checkbox check, if needed.

    if result.is_checked {
        Graphics::line(
            panel_buffer,
            checkbox_top_left.0 as i32,
            checkbox_top_left.1 as i32,
            checkbox_bottom_right.0 as i32,
            checkbox_bottom_right.1 as i32,
            color,
        );
        Graphics::line(
            panel_buffer,
            checkbox_top_right.0 as i32,
            checkbox_top_right.1 as i32,
            checkbox_bottom_left.0 as i32,
            checkbox_bottom_left.1 as i32,
            color,
        );
    }

    // Draw the checkbox label.

    let op = TextOperation {
        text: &options.label,
        x: checkbox_top_right.0 + 4,
        y: checkbox_top_right.1,
        color,
    };

    Graphics::blit_text_from_mask(texture, &op, panel_buffer)
}
