use std::sync::RwLock;

use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    color::{self},
    device::{MouseEventKind, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{text::TextOperation, Graphics},
};

use super::panel::PanelInfo;

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
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache: &'static RwLock<FontCache<'static>>,
    font_info: &FontInfo,
    options: &ButtonOptions,
) -> DoButtonResult {
    let op = TextOperation {
        text: &options.label,
        x: 0,
        y: 0,
        color: color::YELLOW,
    };

    let mut cache = font_cache.write().unwrap();

    let font = cache.load(font_info).unwrap();

    let (label_width, label_height, text_texture) =
        Graphics::make_text_texture(font.as_ref(), &op).unwrap();

    let mut is_down: bool = false;
    let mut was_released: bool = false;

    // Check whether a mouse event occurred inside this button.

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0 as u32, mouse_state.position.1 as u32);

    let x = if options.align_right {
        panel_info.width - label_width - options.x_offset
    } else {
        options.x_offset
    };

    let y = options.y_offset;

    if mouse_x >= panel_info.x && mouse_y >= panel_info.y {
        // Maps mouse_x and mouse_y into panel's local coordinates.

        mouse_x -= panel_info.x;
        mouse_y -= panel_info.y;

        if mouse_x as u32 >= x
            && mouse_x < x + label_width
            && mouse_y >= y
            && mouse_y < y + label_height
        {
            // Check whether LMB was pressed or released inside of this button.

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

    let result = DoButtonResult {
        is_down,
        was_released,
    };

    // Render an unpressed or pressed button.
    draw_button(
        panel_buffer,
        x,
        y,
        label_width,
        label_height,
        &text_texture,
        options,
        &result,
    );

    DoButtonResult {
        is_down,
        was_released,
    }
}

fn draw_button(
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    text_texture: &Buffer2D<u8>,
    options: &ButtonOptions,
    result: &DoButtonResult,
) {
    // Draw the button's text label.

    Graphics::blit_u8_to_u32(text_texture, x, y, width, height, panel_buffer);

    // Draw the button's border.

    if options.with_border {
        Graphics::rectangle(
            panel_buffer,
            x,
            y,
            width,
            height,
            if result.is_down {
                color::GREEN
            } else {
                color::YELLOW
            },
        )
    }
}
