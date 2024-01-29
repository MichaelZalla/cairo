use std::{
    collections::hash_map::Entry,
    sync::{RwLock, RwLockWriteGuard},
};

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

use super::{
    context::{UIContext, UIID},
    panel::PanelInfo,
};

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
    ui_context: &'static RwLock<UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &CheckboxOptions,
    model: Entry<'_, String, bool>,
) -> DoCheckboxResult {
    let mut ctx = ui_context.write().unwrap();

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

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0, mouse_state.position.1);

    let checkbox_size = texture.height;

    let x = if options.align_right {
        panel_info.width - checkbox_size - options.x_offset
    } else {
        options.x_offset
    };

    let y = options.y_offset;

    // Maps mouse_x and mouse_y into panel's local coordinates.

    mouse_x -= panel_info.x as i32;
    mouse_y -= panel_info.y as i32;

    let mouse_in_bounds = mouse_x >= x as i32
        && mouse_x < (x + texture.width) as i32
        && mouse_y >= y as i32
        && mouse_y < (y + texture.height) as i32;

    match (ctx.get_hover_target(), mouse_in_bounds) {
        (Some(target_id), true) => {
            if target_id != id {
                // Mouse is positioned inside of this checkbox (making it the
                // current hover target).

                ctx.set_hover_target(Some(id))
            }
        }
        (None, true) => ctx.set_hover_target(Some(id)),
        (Some(target_id), false) => {
            // Yield the hover target to some other UI item.

            if target_id == id {
                ctx.set_hover_target(None)
            }
        }
        (None, false) => (),
    }

    match mouse_state.button_event {
        Some(event) => match event.button {
            MouseButton::Left => match (event.kind, mouse_in_bounds) {
                (MouseEventKind::Up, true) => {
                    // Check whether LMB was just released inside of this
                    // checkbox.

                    was_released = true;
                }
                (MouseEventKind::Down, true) => {
                    // Check whether LMB was just pressed inside of this
                    // checkbox.

                    match ctx.get_focus_target() {
                        Some(target_id) => {
                            if target_id != id {
                                ctx.set_focus_target(Some(id))
                            }
                        }
                        None => ctx.set_focus_target(Some(id)),
                    }
                }
                (MouseEventKind::Up, false) => {}
                (MouseEventKind::Down, false) => match ctx.get_focus_target() {
                    Some(target_id) => {
                        if target_id == id {
                            ctx.set_focus_target(None)
                        }
                    }
                    None => (),
                },
            },
            _ => (),
        },
        None => (),
    }

    // Check whether LMB is down inside of this checkbox.

    match (
        mouse_state.buttons_down.get(&MouseButton::Left),
        mouse_in_bounds,
    ) {
        (Some(_), true) => {
            is_down = true;
        }
        _ => (),
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
    draw_checkbox(ctx, id, panel_buffer, x, y, options, texture, &result);

    result
}

fn draw_checkbox(
    ui_context: RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    options: &CheckboxOptions,
    texture: &Buffer2D<u8>,
    result: &DoCheckboxResult,
) {
    let checkbox_size = texture.height;

    let is_focus_target = ui_context
        .get_focus_target()
        .is_some_and(|target_id| target_id == id);

    let is_hover_target = ui_context
        .get_hover_target()
        .is_some_and(|target_id| target_id == id);

    let color = if is_focus_target {
        color::RED
    } else if result.is_down {
        color::GREEN
    } else if is_hover_target {
        color::WHITE
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
