use std::{
    collections::hash_map::Entry,
    f32::consts::PI,
    sync::{RwLock, RwLockWriteGuard},
};

use sdl2::keyboard::Keycode;

use crate::{
    buffer::Buffer2D,
    device::{keycode::get_alpha_numeric, KeyboardState, MouseState},
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
    get_mouse_result,
    layout::{ItemLayoutOptions, ItemTextAlignment},
    panel::PanelInfo,
};

static TEXTBOX_WIDTH: u32 = 200;
static TEXTBOX_LABEL_PADDING: u32 = 8;
static TEXTBOX_TEXT_PADDING: u32 = 4;
static TEXTBOX_CURSOR_PADDING: u32 = 2;

#[derive(Default, Debug)]
pub struct TextboxOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
    pub input_text_alignment: ItemTextAlignment,
}

#[derive(Default, Debug)]
pub struct DoTextboxResult {
    pub did_edit: bool,
}

pub fn do_textbox(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    uptime_seconds: f32,
    keyboard_state: &KeyboardState,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &TextboxOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoTextboxResult {
    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let label_texture = text_cache.get(&text_cache_key).unwrap();

    // Check whether a mouse event occurred inside this textbox.

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, TEXTBOX_WIDTH);

    let (_is_down, _was_released) = get_mouse_result(
        ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        TEXTBOX_WIDTH + TEXTBOX_LABEL_PADDING + label_texture.width,
        label_texture.height,
    );

    // Updates the state of our textbox model, if needed.

    let mut did_edit = false;

    match ctx.get_focus_target() {
        Some(target_id) => {
            if target_id == id {
                for code in &keyboard_state.keys_pressed {
                    match code {
                        Keycode::Backspace | Keycode::Delete { .. } => {
                            // Remove one character from the model value, if possible.

                            match &mut model_entry {
                                Entry::Occupied(o) => {
                                    (*o.get_mut()).pop();

                                    did_edit = true;
                                }
                                Entry::Vacant(_v) => {
                                    // Ignore this keypress.
                                }
                            }
                        }
                        _ => {
                            match get_alpha_numeric(code) {
                                Some(char) => {
                                    // Add this character to the model value (string).

                                    match &mut model_entry {
                                        Entry::Occupied(o) => {
                                            *o.get_mut() += char;

                                            did_edit = true;
                                        }
                                        Entry::Vacant(_v) => {
                                            // No model value exists at this entry.

                                            // Ignore this keypress.
                                        }
                                    }
                                }
                                None => {
                                    // Ignore this keypress.
                                }
                            }
                        }
                    }
                }
            }
        }
        None => (),
    }

    let result = DoTextboxResult { did_edit };

    // Render a textbox.

    draw_textbox(
        ctx,
        id,
        uptime_seconds,
        panel_buffer,
        font_cache_rwl,
        font_info,
        x,
        y,
        options,
        label_texture,
        &mut model_entry,
    );

    result
}

fn draw_textbox(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    uptime_second: f32,
    panel_buffer: &mut Buffer2D,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    font_info: &'static FontInfo,
    x: u32,
    y: u32,
    options: &TextboxOptions,
    label_texture: &Buffer2D<u8>,
    model: &mut Entry<'_, String, String>,
) {
    let textbox_height = label_texture.height;

    let theme = ctx.get_theme();

    let label_color = if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    // Draw the textbox borders.

    Graphics::rectangle(
        panel_buffer,
        x,
        y,
        TEXTBOX_WIDTH,
        textbox_height,
        theme.input_background,
        Some(theme.input_background),
    );

    let textbox_top_left = (x, y);
    let textbox_top_right = (x + TEXTBOX_WIDTH - 1, y);

    // Draw the textbox model value (text).

    match model {
        Entry::Occupied(o) => {
            let text = o.get();

            if text.len() > 0 {
                // Draw the text.

                let mut font_cache = font_cache_rwl.write().unwrap();

                let font = font_cache.load(font_info).unwrap();

                let (_label_width, _label_height, model_value_texture) =
                    Graphics::make_text_texture(font.as_ref(), text).unwrap();

                let max_width = TEXTBOX_WIDTH - TEXTBOX_LABEL_PADDING;

                let input_text_x = match options.input_text_alignment {
                    ItemTextAlignment::Left => textbox_top_left.0 + TEXTBOX_TEXT_PADDING,
                    ItemTextAlignment::Center => {
                        (TEXTBOX_WIDTH as f32 / 2.0 - model_value_texture.width as f32 / 2.0) as u32
                    }
                    ItemTextAlignment::Right => {
                        TEXTBOX_WIDTH - model_value_texture.width - TEXTBOX_LABEL_PADDING
                    }
                };

                Graphics::blit_text_from_mask(
                    &model_value_texture,
                    &TextOperation {
                        text,
                        x: input_text_x,
                        y: textbox_top_left.1 + 1,
                        color: theme.input_text,
                    },
                    panel_buffer,
                    Some(max_width),
                );

                // Draw the text cursor.

                let with_cursor = (uptime_second * 2.0 * PI).sin() > 0.0;

                if ctx.is_focused(id) && with_cursor {
                    Graphics::rectangle(
                        panel_buffer,
                        textbox_top_left.0
                            + TEXTBOX_TEXT_PADDING
                            + model_value_texture
                                .width
                                .min(max_width - TEXTBOX_CURSOR_PADDING)
                            + TEXTBOX_CURSOR_PADDING,
                        textbox_top_right.1 + 2,
                        2,
                        textbox_height - 2 - 2,
                        theme.input_cursor,
                        None,
                    );
                }
            }
        }
        Entry::Vacant(_v) => {
            // Do nothing
        }
    }

    // Draw the textbox label.

    let op = TextOperation {
        text: &options.label,
        x: textbox_top_right.0 + TEXTBOX_LABEL_PADDING,
        y: textbox_top_right.1,
        color: label_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, panel_buffer, None)
}
