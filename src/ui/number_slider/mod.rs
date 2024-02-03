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
    layout::ItemLayoutOptions,
    panel::PanelInfo,
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

pub fn do_number_slider(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
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

    let width: u32;
    let height: u32;

    let text_cache_key = TextCacheKey {
        font_info: ctx.font_info.clone(),
        text: options.label.clone(),
    };

    {
        let text_cache = ctx.text_cache.read().unwrap();

        let label_texture = text_cache.get(&text_cache_key).unwrap();

        width = label_texture.width;
        height = label_texture.height;
    }

    // Check whether a mouse event occurred inside this slider.

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, NUMBER_SLIDER_WIDTH);

    let (_is_down, _was_released) = get_mouse_result(
        ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        NUMBER_SLIDER_WIDTH + NUMBER_SLIDER_LABEL_PADDING + width,
        height,
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

    let result = DoNumberSliderResult { did_edit };

    // Render a number slider.

    draw_slider(
        ctx,
        id,
        panel_buffer,
        x,
        y,
        &text_cache_key,
        options,
        &mut model_entry,
    );

    result
}

fn draw_slider(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    text_cache_key: &TextCacheKey,
    options: &NumberSliderOptions,
    model: &mut Entry<'_, String, String>,
) {
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

    Graphics::rectangle(
        panel_buffer,
        x,
        y,
        NUMBER_SLIDER_WIDTH,
        slider_height,
        theme.input_background,
        Some(theme.input_background),
    );

    let slider_top_left = (x, y);
    let slider_top_right = (x + NUMBER_SLIDER_WIDTH - 1, y);

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

                let input_text_x = (NUMBER_SLIDER_WIDTH as f32 / 2.0
                    - model_value_texture.width as f32 / 2.0)
                    as u32;

                Graphics::blit_text_from_mask(
                    &model_value_texture,
                    &TextOperation {
                        text,
                        x: input_text_x,
                        y: slider_top_left.1 + 1,
                        color: theme.input_text,
                    },
                    panel_buffer,
                    Some(max_width),
                );
            }
        }
        Entry::Vacant(_v) => {
            // Do nothing
        }
    }

    // Draw the number slider label.

    let op = TextOperation {
        text: &options.label,
        x: slider_top_right.0 + NUMBER_SLIDER_LABEL_PADDING,
        y: slider_top_right.1,
        color: text_color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, panel_buffer, None)
}
