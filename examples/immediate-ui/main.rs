extern crate sdl2;

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    env,
};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{resolution::RESOLUTIONS_16X9, App, AppWindowInfo},
    buffer::{
        framebuffer::{Framebuffer, FramebufferAttachmentKind},
        Buffer2D,
    },
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    texture::map::{TextureMap, TextureMapStorageFormat},
    time::TimingInfo,
    ui::{
        button::{do_button, ButtonOptions},
        checkbox::{do_checkbox, CheckboxOptions},
        context::{UIContext, UIID},
        dropdown::{do_dropdown, DropdownOptions},
        image::{do_image, ImageOptions},
        layout::{
            item::{ItemLayoutHorizontalAlignment, ItemLayoutOptions, ItemTextAlignment},
            UILayoutContext, UILayoutDirection, UILayoutExtent,
        },
        panel::{do_panel, PanelOptions, PanelTitlebarOptions},
        separator::{do_separator, SeparatorOptions},
        slider::{do_slider, NumberSliderOptions},
        text::{do_text, TextOptions},
        textbox::{do_textbox, TextboxOptions},
    },
};

use uuid::Uuid;

fn main() -> Result<(), String> {
    let current_resolution_index: usize = 6;

    let resolution = RESOLUTIONS_16X9[current_resolution_index];

    let mut window_info = AppWindowInfo {
        title: "examples/immediate-ui".to_string(),
        window_resolution: resolution,
        canvas_resolution: resolution,
        ..Default::default()
    };

    // Initialize an app.

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Load a system font

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example immediate-ui /path/to/your-font.fon");

        return Ok(());
    }

    let default_font_info = FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    };

    // Initialize framebuffer with attachments

    let framebuffer = Framebuffer::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
    );

    let framebuffer_rc = RefCell::new(framebuffer);

    framebuffer_rc
        .borrow_mut()
        .create_attachment(FramebufferAttachmentKind::Color, None, None);

    // Global UI context

    let global_ui_context: &'static RefCell<UIContext> =
        Box::leak(Box::new(RefCell::new(UIContext::new(
            Box::leak(Box::new(RefCell::new(FontCache::new(
                app.context.ttf_context,
            )))),
            &default_font_info,
            Box::leak(Box::new(RefCell::new(Default::default()))),
        ))));

    // UI panel extents

    let root_id = UIID {
        item: global_ui_context.borrow_mut().next_id(),
    };

    let root_extent = UILayoutExtent {
        left: 0,
        right: framebuffer_rc.borrow().width - 1,
        top: 0,
        bottom: framebuffer_rc.borrow().height - 1,
    };

    let left_panel_uuid = Uuid::new_v4();
    let left_panel_parent = root_id.item;
    let left_panel_extent = UILayoutExtent {
        right: root_extent.right / 2,
        ..root_extent
    };

    let right_panel_uuid = Uuid::new_v4();
    let right_panel_parent = root_id.item;
    let right_panel_extent = UILayoutExtent {
        left: root_extent.right / 2,
        ..root_extent
    };

    let mut panels_model = HashMap::<(Uuid, u32), UILayoutExtent>::new();

    panels_model.insert((left_panel_uuid, left_panel_parent), left_panel_extent);
    panels_model.insert((right_panel_uuid, right_panel_parent), right_panel_extent);

    let mut textboxes_model = HashMap::<String, String>::new();

    textboxes_model.insert(
        format!("{}_textbox", left_panel_uuid).to_string(),
        "ABC 123".to_string(),
    );
    textboxes_model.insert(
        format!("{}_slider", left_panel_uuid).to_string(),
        "0.0".to_string(),
    );
    textboxes_model.insert(
        format!("{}_dropdown", left_panel_uuid).to_string(),
        "Item 1".to_string(),
    );

    textboxes_model.insert(
        format!("{}_textbox", right_panel_uuid).to_string(),
        "o-blah-dee-o-blah-dah".to_string(),
    );
    textboxes_model.insert(
        format!("{}_slider", right_panel_uuid).to_string(),
        "0.0".to_string(),
    );
    textboxes_model.insert(
        format!("{}_dropdown", right_panel_uuid).to_string(),
        "Item 4".to_string(),
    );

    let mut checkboxes_model = HashMap::<String, bool>::new();

    let mut wojak_texture = TextureMap::new(
        "./examples/immediate-ui/assets/wojak.png",
        TextureMapStorageFormat::Index8,
    );

    wojak_texture.load(rendering_context).unwrap();

    let layout_direction = UILayoutDirection::TopToBottom;

    let layout_direction_rc = RefCell::new(layout_direction);

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        match framebuffer_rc.borrow_mut().attachments.color.as_mut() {
            Some(rc) => {
                let mut color_buffer = rc.borrow_mut();

                color_buffer.clear(None);

                let mut ctx = global_ui_context.borrow_mut();

                ctx.reset_id_counter(root_id.item + 1);

                // Process global inputs.

                {
                    for keycode in &keyboard_state.keys_pressed {
                        match keycode {
                            Keycode::L { .. } => {
                                let mut layout_direction = layout_direction_rc.borrow_mut();
                                
                                *layout_direction = match *layout_direction {
                                    UILayoutDirection::LeftToRight => UILayoutDirection::TopToBottom,
                                    UILayoutDirection::TopToBottom => UILayoutDirection::LeftToRight,
                                }
                            }
                            _ => ()
                        }
                    }
                }

                // Draw all panels.

                let mut active_panel_uuid: Option<Uuid> = None;
                let mut active_panel_resize_request: Option<(i32, i32)> = None;

                panels_model.retain(
                    |(panel_uuid, _), panel_extent: &mut UILayoutExtent| -> bool {
                        let panel_options = PanelOptions {
                            titlebar_options: Some(PanelTitlebarOptions {
                                title: format!("Panel {}", panel_uuid).to_string(),
                            }),
                            resizable: if *panel_uuid == left_panel_uuid {
                                true
                            } else {
                                false
                            },
                            ..Default::default()
                        };

                        let mut panel_layout = UILayoutContext::new(
                            UILayoutDirection::TopToBottom,
                            *panel_extent,
                            Default::default(),
                        );

                        let do_panel_result = do_panel(
                            &mut ctx,
                            &panel_uuid,
                            &mut panel_layout,
                            &mut color_buffer,
                            &panel_options,
                            mouse_state,
                            keyboard_state,
                            game_controller_state,
                            &mut |ctx: &mut RefMut<'_, UIContext>,
                                layout: &mut UILayoutContext,
                                panel_uuid: &Uuid,
                                panel_id: &UIID,
                                parent_buffer: &mut Buffer2D,
                                mouse_state: &MouseState,
                                keyboard_state: &KeyboardState,
                                _game_controller_state: &GameControllerState| {
                                
                                layout.direction = *layout_direction_rc.borrow();

                                draw_sample_panel_contents(
                                    ctx,
                                    layout,
                                    panel_uuid,
                                    panel_id,
                                    parent_buffer,
                                    mouse_state,
                                    keyboard_state,
                                    &mut textboxes_model,
                                    &mut checkboxes_model,
                                    &app.timing_info,
                                    &mut wojak_texture,
                                );
                            },
                        );

                        if do_panel_result.should_close {
                            println!("Closing Panel ({})...", panel_uuid);

                            active_panel_uuid = Some(*panel_uuid);

                            return false;
                        }

                        if do_panel_result.requested_resize.0 != 0
                            || do_panel_result.requested_resize.1 != 0
                        {
                            let (delta_x, delta_y) = do_panel_result.requested_resize;

                            println!("Resizing Panel {}: {}, {}", panel_uuid, delta_x, delta_y);

                            active_panel_uuid = Some(*panel_uuid);
                            active_panel_resize_request = Some(do_panel_result.requested_resize);
                        }

                        true
                    },
                );

                match active_panel_uuid {
                    Some(active_uuid) => {
                        match active_panel_resize_request {
                            Some(_resize_request) => {
                                // Resize request scenario.

                                static MIN_PANEL_WIDTH: u32 = 150;

                                let mouse_x_relative_to_root =
                                    mouse_state.position.0 - root_extent.left as i32;

                                for ((uuid, _parent), extent) in panels_model.iter_mut() {
                                    if *uuid == active_uuid {
                                        extent.right = mouse_x_relative_to_root
                                            .min((root_extent.right - MIN_PANEL_WIDTH) as i32)
                                            .max(MIN_PANEL_WIDTH as i32)
                                            as u32;
                                    } else {
                                        extent.left = mouse_x_relative_to_root
                                            .min((root_extent.right - MIN_PANEL_WIDTH) as i32)
                                            .max(MIN_PANEL_WIDTH as i32)
                                            as u32;
                                    }
                                }
                            }
                            None => {
                                // Close request scenario.

                                // Update the sibling panel's extent.

                                for ((uuid, _parent), extent) in panels_model.iter_mut() {
                                    if *uuid != active_uuid {
                                        extent.left = root_extent.left;
                                        extent.right = root_extent.right;
                                    }
                                }
                            }
                        }
                    }
                    None => (),
                }
            }
            None => (),
        }

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        match framebuffer_rc.borrow_mut().attachments.color.as_ref() {
            Some(rc) => {
                let color_buffer = rc.borrow();

                Ok(color_buffer.get_all().clone())
            }
            None => Err("Framebuffer has no color attachment!".to_string()),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}

fn draw_sample_panel_contents(
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
        label: format!("Bordered button").to_string(),
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
            label: format!("Borderless button").to_string(),
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
            ..text_options
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
            ..text_options
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
