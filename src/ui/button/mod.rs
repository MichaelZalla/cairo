use std::sync::{RwLock, RwLockWriteGuard};

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
pub struct ButtonOptions {
    pub x_offset: u32,
    pub y_offset: u32,
    pub label: String,
    pub align_right: bool,
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
    let mut ctx = ui_context.write().unwrap();

    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    //

    let mut is_down: bool = false;
    let mut was_released: bool = false;

    // Check whether a mouse event occurred inside this button.

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0, mouse_state.position.1);

    let x = if options.align_right {
        panel_info.width - texture.width - options.x_offset
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
                // Mouse is positioned inside of this button (making it the
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
                    // button.

                    was_released = true;
                }
                (MouseEventKind::Down, true) => {
                    // Check whether LMB was just pressed inside of this
                    // button.

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

    // Check whether LMB is down inside of this button.

    match (
        mouse_state.buttons_down.get(&MouseButton::Left),
        mouse_in_bounds,
    ) {
        (Some(_), true) => {
            is_down = true;
        }
        _ => (),
    }

    let result = DoButtonResult {
        is_down,
        was_released,
    };

    // Render an unpressed or pressed button.
    draw_button(ctx, id, panel_buffer, x, y, texture, options, &result);

    DoButtonResult {
        is_down,
        was_released,
    }
}

fn draw_button(
    ui_context: RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    texture: &Buffer2D<u8>,
    options: &ButtonOptions,
    result: &DoButtonResult,
) {
    let is_hover_target = ui_context
        .get_hover_target()
        .is_some_and(|target_id| target_id == id);

    let color = if result.is_down {
        color::GREEN
    } else if is_hover_target {
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

    Graphics::blit_text_from_mask(texture, &op, panel_buffer);

    // Draw the button's border.

    if options.with_border {
        Graphics::rectangle(panel_buffer, x, y, texture.width, texture.height, color)
    }
}
