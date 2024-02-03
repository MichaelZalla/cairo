use std::sync::RwLockWriteGuard;

use sdl2::mouse::MouseButton;

use crate::device::{MouseEventKind, MouseState};

use self::{
    context::{UIContext, UIID},
    layout::UILayoutContext,
};

pub mod button;
pub mod checkbox;
pub mod context;
pub mod dropdown;
pub mod layout;
pub mod panel;
pub mod separator;
pub mod slider;
pub mod text;
pub mod textbox;
pub mod theme;

pub fn get_mouse_result(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    layout: &UILayoutContext,
    mouse_state: &MouseState,
    layout_offset_x: u32,
    layout_offset_y: u32,
    item_width: u32,
    item_height: u32,
) -> (bool, bool) {
    let mut is_down: bool = false;
    let mut was_released: bool = false;

    // Maps mouse_x and mouse_y into panel's local coordinates.

    let local_mouse_x = mouse_state.position.0;
    let local_mouse_y = mouse_state.position.1;

    let cursor = layout.get_cursor();

    let item_top_left = (cursor.x + layout_offset_x, cursor.y + layout_offset_y);
    let item_bottom_right = (
        item_top_left.0 + item_width - 1,
        item_top_left.1 + item_height - 1,
    );

    let mouse_in_bounds = local_mouse_x >= item_top_left.0 as i32
        && local_mouse_x <= item_bottom_right.0 as i32
        && local_mouse_y >= item_top_left.1 as i32
        && local_mouse_y <= item_bottom_right.1 as i32;

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

    (is_down, was_released)
}
