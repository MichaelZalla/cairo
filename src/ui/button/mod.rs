use std::sync::RwLock;

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
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &ButtonOptions,
) -> DoButtonResult {
    let op = TextOperation {
        text: &options.label,
        x: 0,
        y: 0,
        color: color::YELLOW,
    };

    cache_text(font_cache_rwl, text_cache_rwl, font_info, &op);

    let text_cache_key = TextCacheKey {
        font_info,
        text: op.text.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    //

    let mut is_down: bool = false;
    let mut was_released: bool = false;

    // Check whether a mouse event occurred inside this button.

    let (mut mouse_x, mut mouse_y) = (mouse_state.position.0 as u32, mouse_state.position.1 as u32);

    let x = if options.align_right {
        panel_info.width - texture.width - options.x_offset
    } else {
        options.x_offset
    };

    let y = options.y_offset;

    if mouse_x >= panel_info.x && mouse_y >= panel_info.y {
        // Maps mouse_x and mouse_y into panel's local coordinates.

        mouse_x -= panel_info.x;
        mouse_y -= panel_info.y;

        if mouse_x as u32 >= x
            && mouse_x < x + texture.width
            && mouse_y >= y
            && mouse_y < y + texture.height
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
    draw_button(panel_buffer, x, y, texture, options, &result);

    DoButtonResult {
        is_down,
        was_released,
    }
}

fn draw_button(
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    texture: &Buffer2D<u8>,
    options: &ButtonOptions,
    result: &DoButtonResult,
) {
    // Draw the button's text label.

    Graphics::blit_u8_to_u32(texture, x, y, texture.width, texture.height, panel_buffer);

    // Draw the button's border.

    if options.with_border {
        Graphics::rectangle(
            panel_buffer,
            x,
            y,
            texture.width,
            texture.height,
            if result.is_down {
                color::GREEN
            } else {
                color::YELLOW
            },
        )
    }
}
