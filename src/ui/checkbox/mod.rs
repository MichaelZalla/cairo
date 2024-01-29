use std::{
    collections::hash_map::Entry,
    sync::{RwLock, RwLockWriteGuard},
};

use crate::{
    buffer::Buffer2D,
    color::{self},
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

static CHECKBOX_LABEL_PADDING: u32 = 8;

#[derive(Default, Debug)]
pub struct CheckboxOptions {
    pub layout_options: ItemLayoutOptions,
    pub label: String,
}

#[derive(Default, Debug)]
pub struct DoCheckboxResult {
    pub is_down: bool,
    pub was_released: bool,
    pub is_checked: bool,
}

pub fn do_checkbox(
    ui_context: &'static RwLock<UIContext>,
    id: UIID,
    panel_info: &PanelInfo,
    panel_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    text_cache_rwl: &'static RwLock<TextCache<'static>>,
    font_info: &'static FontInfo,
    options: &CheckboxOptions,
    model_entry: Entry<'_, String, bool>,
) -> DoCheckboxResult {
    let mut ctx = ui_context.write().unwrap();

    cache_text(font_cache_rwl, text_cache_rwl, font_info, &options.label);

    let text_cache_key = TextCacheKey {
        font_info,
        text: options.label.clone(),
    };

    let text_cache = text_cache_rwl.read().unwrap();

    let texture = text_cache.get(&text_cache_key).unwrap();

    // Check whether a mouse event occurred inside this checkbox.

    let checkbox_size = texture.height;

    let (x, y) = options
        .layout_options
        .get_top_left_within_parent(panel_info, checkbox_size);

    let checkbox_size = texture.height;

    let (is_down, was_released) = get_mouse_result(
        &mut ctx,
        id,
        panel_info,
        mouse_state,
        x,
        y,
        checkbox_size + CHECKBOX_LABEL_PADDING + texture.width,
        texture.height,
    );

    // Updates the state of our checkbox model, if needed.

    let mut is_checked = match &model_entry {
        Entry::Occupied(occupied_entry) => *(occupied_entry.get()),
        Entry::Vacant(_) => false,
    };

    if was_released {
        // Toggle the model values.

        is_checked = !is_checked;

        model_entry.and_modify(|value| {
            *value = is_checked;
        });
    }

    let result = DoCheckboxResult {
        is_down,
        was_released,
        is_checked,
    };

    // Render an unchecked or checked checkbox.

    draw_checkbox(&mut ctx, id, panel_buffer, x, y, options, texture, &result);

    result
}

fn draw_checkbox(
    ui_context: &mut RwLockWriteGuard<'_, UIContext>,
    id: UIID,
    panel_buffer: &mut Buffer2D,
    x: u32,
    y: u32,
    options: &CheckboxOptions,
    texture: &Buffer2D<u8>,
    result: &DoCheckboxResult,
) {
    let checkbox_size = texture.height;

    let is_focus_target = ui_context
        .get_focus_target()
        .is_some_and(|target_id| target_id == id);

    let is_hover_target = ui_context
        .get_hover_target()
        .is_some_and(|target_id| target_id == id);

    let color = if is_focus_target {
        color::RED
    } else if result.is_down {
        color::GREEN
    } else if is_hover_target {
        color::WHITE
    } else {
        color::YELLOW
    };

    // Draw the checkbox borders.

    Graphics::rectangle(panel_buffer, x, y, checkbox_size, checkbox_size, color);

    let checkbox_top_left = (x, y);
    let checkbox_top_right = (x + checkbox_size, y);
    let checkbox_bottom_left = (x, y + checkbox_size);
    let checkbox_bottom_right = (x + checkbox_size, y + checkbox_size);

    // Draw the checkbox check, if needed.

    if result.is_checked {
        Graphics::line(
            panel_buffer,
            checkbox_top_left.0 as i32,
            checkbox_top_left.1 as i32,
            checkbox_bottom_right.0 as i32,
            checkbox_bottom_right.1 as i32,
            color,
        );
        Graphics::line(
            panel_buffer,
            checkbox_top_right.0 as i32,
            checkbox_top_right.1 as i32,
            checkbox_bottom_left.0 as i32,
            checkbox_bottom_left.1 as i32,
            color,
        );
    }

    // Draw the checkbox label.

    let op = TextOperation {
        text: &options.label,
        x: checkbox_top_right.0 + CHECKBOX_LABEL_PADDING,
        y: checkbox_top_right.1,
        color,
    };

    Graphics::blit_text_from_mask(texture, &op, panel_buffer, None)
}
