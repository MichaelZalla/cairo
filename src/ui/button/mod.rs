use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    color::{self},
    device::{MouseEventKind, MouseState},
    graphics::Graphics,
};

use super::panel::PanelInfo;

pub fn do_button(
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> bool {
    let mut is_button_down: bool = false;
    let mut was_button_released: bool = false;

    // Check whether a mouse event occurred inside this button.

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0 as u32, mouse_state.position.1 as u32);

    if mouse_x >= panel_info.x && mouse_y >= panel_info.y {
        mouse_x -= panel_info.x;
        mouse_y -= panel_info.y;

        if mouse_x as u32 >= x && mouse_x < x + width && mouse_y >= y && mouse_y < height {
            // Check whether LMB was pressed or released inside of this button.

            match mouse_state.buttons_down.get(&MouseButton::Left) {
                Some(_) => {
                    is_button_down = true;
                }
                None => (),
            }

            match mouse_state.button_event {
                Some(event) => match event.button {
                    MouseButton::Left => match event.kind {
                        MouseEventKind::Up => {
                            was_button_released = true;
                        }
                        _ => (),
                    },
                    _ => (),
                },
                None => (),
            }
        }
    }

    // Render an unpressed or pressed button.
    draw_button(
        panel_buffer,
        x,
        y,
        width,
        height,
        is_button_down,
        was_button_released,
    );

    was_button_released
}

fn draw_button(
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    was_pressed: bool,
    _was_released: bool,
) {
    Graphics::rectangle(
        panel_buffer,
        x,
        y,
        width,
        height,
        if was_pressed {
            color::GREEN
        } else {
            color::YELLOW
        },
    )
}
