use std::sync::RwLockWriteGuard;

use sdl2::mouse::MouseButton;

use crate::device::{MouseEventKind, MouseState};

use self::{
    context::{UIContext, UIID},
    panel::PanelInfo,
};

pub mod button;
pub mod checkbox;
pub mod context;
pub mod dropdown;
pub mod layout;
pub mod number_slider;
pub mod panel;
pub mod text;
pub mod textbox;
pub mod theme;

pub fn get_mouse_result(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    mouse_state: &MouseState,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> (bool, bool) {
    let mut is_down: bool = false;
    let mut was_released: bool = false;

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0, mouse_state.position.1);

    // Maps mouse_x and mouse_y into panel's local coordinates.

    mouse_x -= panel_info.x as i32;
    mouse_y -= panel_info.y as i32;

    let mouse_in_bounds = mouse_x >= x as i32
        && mouse_x < (x + width) as i32
        && mouse_y >= y as i32
        && mouse_y < (y + height) as i32;

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
