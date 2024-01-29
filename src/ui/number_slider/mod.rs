use std::{
    collections::hash_map::Entry,
    sync::{RwLock, RwLockWriteGuard},
};

use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    color,
    device::MouseState,
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
    ui_context: &'static RwLock<UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &NumberSliderOptions,
    mut model_entry: Entry<'_, String, String>,
) -> DoNumberSliderResult {
    let mut ctx = ui_context.write().unwrap();

    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let label_texture = text_cache.get(&text_cache_key).unwrap();

    // Check whether a mouse event occurred inside this slider.

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, NUMBER_SLIDER_WIDTH);

    let (_is_down, _was_released) = get_mouse_result(
        &mut ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        NUMBER_SLIDER_WIDTH + NUMBER_SLIDER_LABEL_PADDING + label_texture.width,
        label_texture.height,
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

                                *o.get_mut() = adjusted.to_string();
                            }
                            Err(_) => {}
                        }

                        did_edit = true;
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
        &mut ctx,
        id,
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

fn draw_slider(
    ctx: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    font_info: &'static FontInfo,
    x: u32,
    y: u32,
    options: &NumberSliderOptions,
    label_texture: &Buffer2D<u8>,
    model: &mut Entry<'_, String, String>,
) {
    let slider_height = label_texture.height;

    let color = if ctx.is_focused(id) {
        color::RED
    } else if ctx.is_hovered(id) {
        color::WHITE
    } else {
        color::YELLOW
    };

    // Draw the slider borders.

    Graphics::rectangle(
        panel_buffer,
        x,
        y,
        NUMBER_SLIDER_WIDTH,
        slider_height,
        color,
    );

    let slider_top_left = (x, y);
    let slider_top_right = (x + NUMBER_SLIDER_WIDTH, y);

    // Draw the slider model value.

    match model {
        Entry::Occupied(o) => {
            let text = o.get();

            if text.len() > 0 {
                // Draw the text.

                let mut font_cache = font_cache_rwl.write().unwrap();

                let font = font_cache.load(font_info).unwrap();

                let (_label_width, _label_height, model_value_texture) =
                    Graphics::make_text_texture(font.as_ref(), text).unwrap();

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
                        color,
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
        color,
    };

    Graphics::blit_text_from_mask(label_texture, &op, panel_buffer, None)
}
