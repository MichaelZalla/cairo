use std::{cell::RefMut, collections::HashMap};

use uuid::Uuid;

use cairo::{
    buffer::Buffer2D,
    color::{self, Color},
    device::{KeyboardState, MouseState},
    texture::map::TextureMap,
    time::TimingInfo,
};

use super::ui::{
    button::{do_button, ButtonOptions},
    checkbox::{do_checkbox, CheckboxOptions},
    context::{UIContext, UIID},
    dropdown::{do_dropdown, DropdownOptions},
    image::{do_image, ImageOptions},
    layout::{
        item::{ItemLayoutHorizontalAlignment, ItemLayoutOptions, ItemTextAlignment},
        UILayoutContext,
    },
    separator::{do_separator, SeparatorOptions},
    slider::{do_slider, NumberSliderOptions},
    text::{do_text, TextOptions},
    textbox::{do_textbox, TextboxOptions},
};

#[allow(clippy::too_many_arguments)]
pub fn draw_sample_panel_contents(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    panel_uuid: &Uuid,
    panel_id: &UIID,
    parent_buffer: &mut Buffer2D,
    mouse_state: &MouseState,
    keyboard_state: &KeyboardState,
    textboxes_model: &mut HashMap<String, String>,
    checkboxes_model: &mut HashMap<String, bool>,
    timing_info: &TimingInfo,
    wojak_texture: &mut TextureMap,
) {
    // Draw a bordered button.

    let button_options = ButtonOptions {
        label: "Bordered button".to_string(),
        with_border: true,
        ..Default::default()
    };

    let button_1_id = UIID {
        item: ctx.next_id(),
    };

    if do_button(ctx, layout, parent_buffer, mouse_state, &button_options).was_released {
        println!("You clicked a Button ({}).", button_1_id);
    }

    // Draw a borderless button.

    let button_2_id = UIID {
        item: ctx.next_id(),
    };

    if do_button(
        ctx,
        layout,
        parent_buffer,
        mouse_state,
        &ButtonOptions {
            label: "Borderless button".to_string(),
            with_border: false,
            ..button_options
        },
    )
    .was_released
    {
        println!("You clicked a Button ({}).", button_2_id);
    }

    // Draw a checkbox.

    let checkbox_options = CheckboxOptions {
        label: format!("Checkbox {}", panel_id.item).to_string(),
        ..Default::default()
    };

    let checkbox_model_key = format!("{}_checkbox", panel_uuid);

    checkboxes_model
        .entry(checkbox_model_key.clone())
        .or_default();

    let checkbox_model_entry = checkboxes_model.entry(checkbox_model_key.clone());

    let checkbox_id = UIID {
        item: ctx.next_id(),
    };

    if do_checkbox(
        ctx,
        layout,
        parent_buffer,
        mouse_state,
        &checkbox_options,
        checkbox_model_entry,
    )
    .was_released
    {
        let is_checked = checkboxes_model
            .entry(checkbox_model_key.clone())
            .or_default();

        println!(
            "The Checkbox ({}) is now {}!",
            checkbox_id,
            if *is_checked { "checked" } else { "unchecked" }
        );
    }

    // Draw a separator.

    do_separator(
        ctx,
        layout,
        &SeparatorOptions {
            ..Default::default()
        },
        parent_buffer,
    );

    // Draw some cached text labels.

    let text_options = TextOptions {
        text: format!("Welcome to Panel {}!", panel_id.item),
        color: color::WHITE,
        ..Default::default()
    };

    do_text(ctx, layout, parent_buffer, &text_options);

    do_text(
        ctx,
        layout,
        parent_buffer,
        &TextOptions {
            layout_options: ItemLayoutOptions {
                horizontal_alignment: ItemLayoutHorizontalAlignment::Center,
                ..Default::default()
            },
            text: format!("FPS: {:.*}", 0, timing_info.frames_per_second),
            color: color::RED,
            cache: false,
        },
    );

    // Draw a non-cached text label.

    do_text(
        ctx,
        layout,
        parent_buffer,
        &TextOptions {
            layout_options: ItemLayoutOptions {
                horizontal_alignment: ItemLayoutHorizontalAlignment::Right,
                ..Default::default()
            },
            text: format!("Uptime: {:.*}", 2, timing_info.uptime_seconds),
            cache: false,
            color: color::GREEN,
        },
    );

    // Draw a separator.

    do_separator(
        ctx,
        layout,
        &SeparatorOptions {
            ..Default::default()
        },
        parent_buffer,
    );

    // Draw a downscaled image.

    do_image(
        ctx,
        layout,
        wojak_texture,
        &ImageOptions {
            width: 256,
            height: 256,
            border: Some(Color::rgb(45, 45, 45)),
        },
        parent_buffer,
    );

    // Draw a separator.

    do_separator(
        ctx,
        layout,
        &SeparatorOptions {
            ..Default::default()
        },
        parent_buffer,
    );

    // Draw a textbox.

    let textbox_options = TextboxOptions {
        label: format!("Textbox {}", panel_id.item).to_string(),
        input_text_alignment: ItemTextAlignment::Left,
        ..Default::default()
    };

    let textbox_model_key = format!("{}_textbox", panel_uuid);

    textboxes_model
        .entry(textbox_model_key.clone())
        .or_default();

    let textbox_model_entry = textboxes_model.entry(textbox_model_key.clone());

    let textbox_id = UIID {
        item: ctx.next_id(),
    };

    if do_textbox(
        ctx,
        layout,
        parent_buffer,
        timing_info.uptime_seconds,
        keyboard_state,
        mouse_state,
        &textbox_options,
        textbox_model_entry,
    )
    .did_edit
    {
        println!("You edited a Textbox ({})!", textbox_id);
    }

    // Draw a number slider.

    let slider_options = NumberSliderOptions {
        label: format!("Slider {}", panel_id.item).to_string(),
        min: Some(-1.0 * panel_id.item as f32),
        max: Some(1.0 * panel_id.item as f32),
        ..Default::default()
    };

    let slider_model_key = format!("{}_slider", panel_uuid);

    textboxes_model.entry(slider_model_key.clone()).or_default();

    let slider_model_entry = textboxes_model.entry(slider_model_key.clone());

    let slider_id = UIID {
        item: ctx.next_id(),
    };

    if do_slider(
        ctx,
        layout,
        parent_buffer,
        mouse_state,
        &slider_options,
        slider_model_entry,
    )
    .did_edit
    {
        println!("You edited a NumberSlider ({})!", slider_id);
    }

    // Draw a dropdown menu.

    let dropdown_options = DropdownOptions {
        label: format!("Dropdown {}", panel_id.item).to_string(),
        items: vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
            "Item 4".to_string(),
            "Item 5".to_string(),
        ],
        ..Default::default()
    };

    let dropdown_model_key = format!("{}_dropdown", panel_uuid);

    textboxes_model
        .entry(dropdown_model_key.clone())
        .or_default();

    let dropdown_model_entry = textboxes_model.entry(dropdown_model_key.clone());

    if do_dropdown(
        ctx,
        layout,
        parent_buffer,
        mouse_state,
        &dropdown_options,
        dropdown_model_entry,
    )
    .did_edit
    {
        println!("You edited a Dropdown ({})!", slider_id);
    }
}
