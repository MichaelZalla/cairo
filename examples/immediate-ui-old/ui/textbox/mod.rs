use std::{cell::RefMut, collections::hash_map::Entry, f32::consts::PI};

use sdl2::keyboard::Keycode;

use cairo::{
    buffer::Buffer2D,
    device::{keycode::get_alpha_numeric, KeyboardState, MouseState},
    graphics::{
        text::{
            cache::{cache_text, TextCacheKey},
            TextOperation,
        },
        Graphics,
    },
};

use super::{
    context::{UIContext, UIID},
    get_mouse_result,
    layout::{
        item::{ItemLayoutOptions, ItemTextAlignment},
        UILayoutContext,
    },
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
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    uptime_seconds: f32,
    keyboard_state: &KeyboardState,
    mouse_state: &MouseState,
    options: &TextboxOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoTextboxResult {
    let id = UIID {
        item: ctx.next_id(),
    };

    cache_text(
        ctx.font_cache,
        ctx.text_cache,
        &ctx.font_info,
        &options.label,
    );

    let label_texture_width: u32;
    let label_texture_height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.borrow();

        let label_texture = text_cache.get(&text_cache_key).unwrap();

        label_texture_width = label_texture.width;
        label_texture_height = label_texture.height;
    }

    // Check whether a mouse event occurred inside this textbox.

    let (layout_offset_x, layout_offset_y) = options
        .layout_options
        .get_layout_offset(layout, TEXTBOX_WIDTH);

    let item_width = TEXTBOX_WIDTH + TEXTBOX_LABEL_PADDING + label_texture_width;
    let item_height = label_texture_height;

    let (_is_down, _was_released) = get_mouse_result(
        ctx,
        &id,
        layout,
        mouse_state,
        layout_offset_x,
        layout_offset_y,
        item_width,
        item_height,
    );

    // Updates the state of our textbox model, if needed.

    let mut did_edit = false;

    if let Some(target_id) = ctx.get_focus_target() {
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

    let result = DoTextboxResult { did_edit };

    // Render a textbox.

    layout.prepare_cursor(item_width, item_height);

    draw_textbox(
        ctx,
        &id,
        layout,
        layout_offset_x,
        layout_offset_y,
        &text_cache_key,
        options,
        &mut model_entry,
        uptime_seconds,
        parent_buffer,
    );

    layout.advance_cursor(item_width, item_height);

    result
}

fn draw_textbox(
    ctx: &mut RefMut<'_, UIContext>,
    id: &UIID,
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    text_cache_key: &TextCacheKey,
    options: &TextboxOptions,
    model: &mut Entry<'_, String, String>,
    uptime_second: f32,
    parent_buffer: &mut Buffer2D,
) {
    let cursor = layout.get_cursor();

    let text_cache = ctx.text_cache.borrow();

    let label_texture = text_cache.get(text_cache_key).unwrap();

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

    let (textbox_x, textbox_y) = (cursor.x + layout_offset_x, cursor.y + layout_offset_y);

    Graphics::rectangle(
        parent_buffer,
        textbox_x,
        textbox_y,
        TEXTBOX_WIDTH,
        textbox_height,
        Some(&theme.input_background),
        None,
    );

    let textbox_top_left = (textbox_x, textbox_y);
    let textbox_top_right = (textbox_x + TEXTBOX_WIDTH - 1, textbox_y);

    // Draw the textbox model value (text).

    match model {
        Entry::Occupied(o) => {
            let text = o.get();

            if !text.is_empty() {
                // Draw the input value text.

                let mut font_cache = ctx.font_cache.borrow_mut();

                let font = font_cache.load(&ctx.font_info).unwrap();

                let (_label_width, _label_height, model_value_texture) =
                    Graphics::make_text_mask(font.as_ref(), text).unwrap();

                let max_width = TEXTBOX_WIDTH - TEXTBOX_LABEL_PADDING;

                let input_text_x = textbox_top_left.0
                    + match options.input_text_alignment {
                        ItemTextAlignment::Left => TEXTBOX_TEXT_PADDING,
                        ItemTextAlignment::Center => {
                            (TEXTBOX_WIDTH as f32 / 2.0 - model_value_texture.0.width as f32 / 2.0)
                                as u32
                        }
                        ItemTextAlignment::Right => {
                            TEXTBOX_WIDTH - model_value_texture.0.width - TEXTBOX_LABEL_PADDING
                        }
                    };

                let input_text_y = textbox_top_left.1 + 1;

                Graphics::blit_text_from_mask(
                    &model_value_texture.0,
                    &TextOperation {
                        text,
                        x: input_text_x,
                        y: input_text_y,
                        color: theme.input_text,
                    },
                    parent_buffer,
                    Some(max_width),
                );

                // Draw the text cursor.

                let with_cursor = (uptime_second * 2.0 * PI).sin() > 0.0;

                if ctx.is_focused(id) && with_cursor {
                    let blinking_text_cursor_x = textbox_top_left.0
                        + TEXTBOX_TEXT_PADDING
                        + model_value_texture
                            .0
                            .width
                            .min(max_width - TEXTBOX_CURSOR_PADDING)
                        + TEXTBOX_CURSOR_PADDING;

                    let blinking_text_cursor_y = textbox_top_right.1 + 2;

                    Graphics::rectangle(
                        parent_buffer,
                        blinking_text_cursor_x,
                        blinking_text_cursor_y,
                        2,
                        textbox_height - 2 - 2,
                        None,
                        Some(&theme.input_cursor),
                    );
                }
            }
        }
        Entry::Vacant(_v) => {
            // Do nothing
        }
    }

    // Draw the textbox label.

    let (label_x, label_y) = (
        textbox_top_right.0 + TEXTBOX_LABEL_PADDING,
        textbox_top_right.1,
    );

    let op = TextOperation {
        text: &options.label,
        x: label_x,
        y: label_y,
        color: label_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, parent_buffer, None)
}
