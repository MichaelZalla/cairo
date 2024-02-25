extern crate sdl2;

use std::{cell::RefCell, collections::HashMap, env, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
    texture::map::{TextureMap, TextureMapStorageFormat},
    ui::{
        button::{do_button, ButtonOptions},
        checkbox::{do_checkbox, CheckboxOptions},
        context::UIContext,
        dropdown::{do_dropdown, DropdownOptions},
        image::{do_image, ImageOptions},
        layout::{
            item::{ItemLayoutHorizontalAlignment, ItemLayoutOptions, ItemTextAlignment},
            UILayoutContext, UILayoutDirection, UILayoutExtent, UILayoutOptions,
        },
        panel::{Panel, PanelInfo, PANEL_TITLE_BAR_HEIGHT},
        separator::{do_separator, SeparatorOptions},
        slider::{do_slider, NumberSliderOptions},
        text::{do_text, TextOptions},
        textbox::{do_textbox, TextboxOptions},
    },
};

fn main() -> Result<(), String> {
    let resolution = (1920, 1080);

    let mut window_info = AppWindowInfo {
        title: "examples/ui-panels".to_string(),
        window_width: resolution.0,
        window_height: resolution.1,
        canvas_width: resolution.0,
        canvas_height: resolution.1,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Load a system font

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example ui-panels /path/to/your-font.fon");

        return Ok(());
    }

    // Create a static text (texture) cache.

    let _text_cache: TextCache = Default::default();

    // Set up our app

    let mut framebuffer = Buffer2D::new(window_info.window_width, window_info.window_height, None);

    let default_font_info = FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    };

    let ui_context: &'static RefCell<UIContext> =
        Box::leak(Box::new(RefCell::new(UIContext::new(
            Box::leak(Box::new(RefCell::new(FontCache::new(
                app.context.ttf_context,
            )))),
            &default_font_info,
            Box::leak(Box::new(RefCell::new(_text_cache))),
        ))));

    let mut textboxes_model = HashMap::<String, String>::new();

    textboxes_model.insert("1_textbox".to_string(), "ABC 123".to_string());
    textboxes_model.insert("1_slider".to_string(), "0.0".to_string());
    textboxes_model.insert("1_dropdown".to_string(), "Item 1".to_string());

    textboxes_model.insert("2_textbox".to_string(), "o-blah-dee-o-blah-dah".to_string());
    textboxes_model.insert("2_slider".to_string(), "0.0".to_string());
    textboxes_model.insert("2_dropdown".to_string(), "Item 4".to_string());

    let mut checkboxes_model = HashMap::<String, bool>::new();

    let mut wojak_texture = TextureMap::new(
        "./examples/ui-panels/assets/wojak.png",
        TextureMapStorageFormat::Index8,
    );

    wojak_texture.load(rendering_context).unwrap();

    let layout_direction = RefCell::new(UILayoutDirection::TopToBottom);

    let render_rc = RefCell::new(
        |panel_info: &PanelInfo,
         panel_buffer: &mut Buffer2D,
         app: &mut App,
         keyboard_state: &KeyboardState,
         mouse_state: &MouseState|
         -> Result<(), String> {
            let mut ctx = ui_context.borrow_mut();

            // @NOTE(mzalla) Layout cursor is currently local to panel's own UI
            // buffer.
            let mut layout = UILayoutContext::new(
                layout_direction.borrow().clone(),
                UILayoutExtent {
                    left: 0,
                    right: panel_info.width,
                    top: PANEL_TITLE_BAR_HEIGHT,
                    bottom: panel_info.height,
                },
                UILayoutOptions { padding: 8, gap: 8 },
            );

            // Draw a bordered button.

            let button_options = ButtonOptions {
                label: format!("Bordered button").to_string(),
                with_border: true,
                ..Default::default()
            };

            if do_button(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                mouse_state,
                &button_options,
            )
            .was_released
            {
                println!("You clicked a Button.");
            }

            // Draw a borderless button.

            if do_button(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                mouse_state,
                &ButtonOptions {
                    label: format!("Borderless button").to_string(),
                    with_border: false,
                    ..button_options
                },
            )
            .was_released
            {
                println!("You clicked a Button.");
            }

            // Draw a checkbox.

            let checkbox_options = CheckboxOptions {
                label: format!("Checkbox {}", panel_info.id).to_string(),
                ..Default::default()
            };

            let checkbox_model_key = panel_info.id.to_string() + "_checkbox";

            checkboxes_model
                .entry(checkbox_model_key.clone())
                .or_default();

            let checkbox_model_entry = checkboxes_model.entry(checkbox_model_key.clone());

            if do_checkbox(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
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
                    "The Checkbox is now {}!",
                    if *is_checked { "checked" } else { "unchecked" }
                );
            }

            // Draw a separator.

            do_separator(
                &mut ctx,
                panel_info.id,
                &mut layout,
                &SeparatorOptions {
                    ..Default::default()
                },
                panel_buffer,
            );

            // Draw some cached text labels.

            let text_options = TextOptions {
                text: format!("Welcome to Panel {}!", panel_info.id),
                color: color::WHITE,
                ..Default::default()
            };

            do_text(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                &text_options,
            );

            do_text(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                &TextOptions {
                    layout_options: ItemLayoutOptions {
                        horizontal_alignment: ItemLayoutHorizontalAlignment::Center,
                        ..Default::default()
                    },
                    text: text_options.text.clone(),
                    color: color::RED,
                    ..text_options
                },
            );

            // Draw a non-cached text label.

            let uptime = app.timing_info.uptime_seconds;

            do_text(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                &TextOptions {
                    layout_options: ItemLayoutOptions {
                        horizontal_alignment: ItemLayoutHorizontalAlignment::Right,
                        ..Default::default()
                    },
                    text: format!("Uptime: {:.*}", 2, uptime),
                    cache: false,
                    color: color::GREEN,
                    ..text_options
                },
            );

            // Draw a separator.

            do_separator(
                &mut ctx,
                panel_info.id,
                &mut layout,
                &SeparatorOptions {
                    ..Default::default()
                },
                panel_buffer,
            );

            // Draw a filled rectangle.

            do_image(
                &mut ctx,
                panel_info.id,
                &mut layout,
                &mut wojak_texture,
                &ImageOptions {
                    width: 256,
                    height: 256,
                    border: Some(Color::rgb(45, 45, 45)),
                },
                panel_buffer,
            );

            // Draw a separator.

            do_separator(
                &mut ctx,
                panel_info.id,
                &mut layout,
                &SeparatorOptions {
                    ..Default::default()
                },
                panel_buffer,
            );

            // Draw a textbox.

            let textbox_options = TextboxOptions {
                label: format!("Textbox {}", panel_info.id).to_string(),
                input_text_alignment: ItemTextAlignment::Left,
                ..Default::default()
            };

            let textbox_model_key = panel_info.id.to_string() + "_textbox";

            textboxes_model
                .entry(textbox_model_key.clone())
                .or_default();

            let textbox_model_entry = textboxes_model.entry(textbox_model_key.clone());

            if do_textbox(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                app.timing_info.uptime_seconds,
                keyboard_state,
                mouse_state,
                &textbox_options,
                textbox_model_entry,
            )
            .did_edit
            {
                println!("You edited a Textbox!");
            }

            // Draw a number slider.

            let slider_options = NumberSliderOptions {
                label: format!("Slider {}", panel_info.id).to_string(),
                min: Some(-1.0 * panel_info.id as f32),
                max: Some(1.0 * panel_info.id as f32),
                ..Default::default()
            };

            let slider_model_key = panel_info.id.to_string() + "_slider";

            textboxes_model.entry(slider_model_key.clone()).or_default();

            let slider_model_entry = textboxes_model.entry(slider_model_key.clone());

            if do_slider(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                mouse_state,
                &slider_options,
                slider_model_entry,
            )
            .did_edit
            {
                println!("You edited a NumberSlider!");
            }

            // Draw a dropdown menu.

            let dropdown_options = DropdownOptions {
                label: format!("Dropdown {}", panel_info.id).to_string(),
                items: vec![
                    "Item 1".to_string(),
                    "Item 2".to_string(),
                    "Item 3".to_string(),
                    "Item 4".to_string(),
                    "Item 5".to_string(),
                ],
                ..Default::default()
            };

            let dropdown_model_key = panel_info.id.to_string() + "_dropdown";

            textboxes_model
                .entry(dropdown_model_key.clone())
                .or_default();

            let dropdown_model_entry = textboxes_model.entry(dropdown_model_key.clone());

            if do_dropdown(
                &mut ctx,
                panel_info.id,
                &mut layout,
                panel_buffer,
                mouse_state,
                &dropdown_options,
                dropdown_model_entry,
            )
            .did_edit
            {
                println!("You edited a Dropdown!");
            }

            Ok(())
        },
    );

    let render_rc_option = Some(&render_rc);

    let mut root_panel = Panel::new(
        PanelInfo {
            title: "Panel 0".to_string(),
            width: window_info.window_width,
            height: window_info.window_height,
            ..Default::default()
        },
        render_rc_option,
    );

    root_panel.split()?;

    let root_panel_rc = RefCell::new(root_panel);

    let current_mouse_state: RwLock<MouseState> = RwLock::new(Default::default());

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::L { .. } => {
                    let mut direction = layout_direction.borrow_mut();

                    match *direction {
                        UILayoutDirection::TopToBottom => {
                            *direction = UILayoutDirection::LeftToRight;
                        }
                        UILayoutDirection::LeftToRight => {
                            *direction = UILayoutDirection::TopToBottom;
                        }
                    }
                }
                _ => (),
            }
        }

        // Delegrate update actions to the root panel?

        let mut root_panel = root_panel_rc.borrow_mut();

        root_panel
            .update(app, keyboard_state, mouse_state, game_controller_state)
            .unwrap();

        // Delegate render call to the root panel

        root_panel
            .render(app, keyboard_state, mouse_state, ui_context)
            .unwrap();

        // Cache the mouse state (position) so that we can render a crosshair.

        current_mouse_state.write().unwrap().position = mouse_state.position;

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        let fill_value = color::WHITE.to_u32();

        // Clears pixel buffer
        framebuffer.clear(Some(fill_value));

        // Blit panel pixels (local space) onto global pixels

        let root = root_panel_rc.borrow();

        framebuffer.blit_from(root.info.x, root.info.y, &root.buffer);

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
