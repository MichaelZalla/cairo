use std::{collections::hash_map::Entry, sync::RwLockWriteGuard};

use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    device::MouseState,
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
    layout::{item::ItemLayoutOptions, UILayoutContext},
};

static NUMBER_SLIDER_WIDTH: u32 = 200;
static NUMBER_SLIDER_LABEL_PADDING: u32 = 8;
static NUMBER_SLIDER_TEXT_PADDING: u32 = 4;

#[derive(Default, Debug)]
pub struct NumberSliderOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

#[derive(Default, Debug)]
pub struct DoNumberSliderResult {
    pub did_edit: bool,
}

pub fn do_slider(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    layout: &mut UILayoutContext,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    options: &NumberSliderOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoNumberSliderResult {
    cache_text(
        ctx.font_cache,
        ctx.text_cache,
        ctx.font_info,
        &options.label,
    );

    let label_texture_width: u32;
    let label_texture_height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.read().unwrap();

        let label_texture = text_cache.get(&text_cache_key).unwrap();

        label_texture_width = label_texture.width;
        label_texture_height = label_texture.height;
    }

    // Check whether a mouse event occurred inside this slider.

    let (layout_offset_x, layout_offset_y) = options
        .layout_options
        .get_layout_offset(layout, NUMBER_SLIDER_WIDTH);

    let item_width = NUMBER_SLIDER_WIDTH + NUMBER_SLIDER_LABEL_PADDING + label_texture_width;
    let item_height = label_texture_height;

    let (_is_down, _was_released) = get_mouse_result(
        ctx,
        id,
        layout,
        mouse_state,
        layout_offset_x,
        layout_offset_y,
        item_width,
        item_height,
    );

    // Updates the state of our slider model, if needed.

    let mut did_edit = false;

    if ctx.is_focused(id) {
        match mouse_state.buttons_down.get(&MouseButton::Left) {
            Some(_) => {
                match &mut model_entry {
                    Entry::Occupied(o) => {
                        let x_motion = mouse_state.relative_motion.0;

                        let delta = x_motion as f32 / NUMBER_SLIDER_WIDTH as f32;

                        let parse_result = o.get().parse::<f32>();

                        let min = match options.min {
                            Some(min) => min,
                            None => f32::MIN,
                        };

                        let max = match options.max {
                            Some(max) => max,
                            None => f32::MAX,
                        };

                        let scaling_factor = max - min;

                        match parse_result {
                            Ok(value) => {
                                let adjusted = (value + delta * scaling_factor).clamp(min, max);

                                let adjusted_str = adjusted.to_string();

                                if *o.get() != adjusted_str {
                                    did_edit = true;

                                    *o.get_mut() = adjusted.to_string();
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Entry::Vacant(_v) => {
                        // Ignore this mouse-drag.
                    }
                }
            }
            None => {
                // Do nothing
            }
        }
    }

    // match ctx.get_focus_target() {
    //     Some(target_id) => {
    //         if target_id == id {
    //             for code in &keyboard_state.keys_pressed {
    //                 match code {
    //                     Keycode::Backspace | Keycode::Delete { .. } => {
    //                         // Remove one character from the model value, if possible.

    //                         match &mut model_entry {
    //                             Entry::Occupied(o) => {
    //                                 (*o.get_mut()).pop();

    //                                 did_edit = true;
    //                             }
    //                             Entry::Vacant(v) => {
    //                                 // Ignore this keypress.
    //                             }
    //                         }
    //                     }
    //                     _ => {
    //                         match get_alpha_numeric(code) {
    //                             Some(char) => {
    //                                 // Add this character to the model value (string).

    //                                 match &mut model_entry {
    //                                     Entry::Occupied(o) => {
    //                                         *o.get_mut() += char;

    //                                         did_edit = true;
    //                                     }
    //                                     Entry::Vacant(_v) => {
    //                                         // No model value exists at this entry.

    //                                         // Ignore this keypress.
    //                                     }
    //                                 }
    //                             }
    //                             None => {
    //                                 // Ignore this keypress.
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     None => (),
    // }

    let result = DoNumberSliderResult { did_edit };

    // Render a number slider.

    layout.prepare_cursor(item_width, item_height);

    draw_slider(
        ctx,
        id,
        layout,
        layout_offset_x,
        layout_offset_y,
        &text_cache_key,
        options,
        &mut model_entry,
        parent_buffer,
    );

    layout.advance_cursor(item_width, item_height);

    result
}

fn draw_slider(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    layout: &UILayoutContext,
    layout_offset_x: u32,
    layout_offset_y: u32,
    text_cache_key: &TextCacheKey,
    options: &NumberSliderOptions,
    model: &mut Entry<'_, String, String>,
    parent_buffer: &mut Buffer2D,
) {
    let cursor = layout.get_cursor();

    let text_cache = ctx.text_cache.read().unwrap();

    let label_texture = text_cache.get(&text_cache_key).unwrap();

    let slider_height = label_texture.height;

    let theme = ctx.get_theme();

    let text_color = if ctx.is_focused(id) {
        theme.text_focus
    } else if ctx.is_hovered(id) {
        theme.text_hover
    } else {
        theme.text
    };

    // Draw the slider borders.

    let slider_top_left = (cursor.x + layout_offset_x, cursor.y + layout_offset_y);
    let slider_top_right = (
        slider_top_left.0 + NUMBER_SLIDER_WIDTH - 1,
        slider_top_left.1,
    );

    Graphics::rectangle(
        parent_buffer,
        slider_top_left.0,
        slider_top_left.1,
        NUMBER_SLIDER_WIDTH,
        slider_height,
        theme.input_background,
        Some(theme.input_background),
    );

    // Draw the slider model value.

    match model {
        Entry::Occupied(o) => {
            let text = o.get();

            if text.len() > 0 {
                // Draw the slider value text (formatted).

                let text_parsed = text.parse::<f32>().unwrap();

                let text_formatted = format!("{:.*}", 2, text_parsed);

                let mut font_cache = ctx.font_cache.write().unwrap();

                let font = font_cache.load(ctx.font_info).unwrap();

                let (_label_width, _label_height, model_value_texture) =
                    Graphics::make_text_texture(font.as_ref(), &text_formatted).unwrap();

                let max_width = NUMBER_SLIDER_WIDTH - NUMBER_SLIDER_LABEL_PADDING;

                let input_text_x = slider_top_left.0
                    + (NUMBER_SLIDER_WIDTH as f32 / 2.0 - model_value_texture.width as f32 / 2.0)
                        as u32;

                let input_text_y = slider_top_left.1 + 1;

                Graphics::blit_text_from_mask(
                    &model_value_texture,
                    &TextOperation {
                        text,
                        x: input_text_x,
                        y: input_text_y,
                        color: theme.input_text,
                    },
                    parent_buffer,
                    Some(max_width),
                );
            }
        }
        Entry::Vacant(_v) => {
            // Do nothing
        }
    }

    // Draw the number slider label.

    let (label_x, label_y) = (
        slider_top_right.0 + NUMBER_SLIDER_LABEL_PADDING,
        slider_top_right.1,
    );

    let op = TextOperation {
        text: &options.label,
        x: label_x,
        y: label_y,
        color: text_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, parent_buffer, None)
}
