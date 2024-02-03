extern crate sdl2;

use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    sync::{RwLock, RwLockWriteGuard},
};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{text::cache::TextCache, Graphics},
    ui::{
        button::{do_button, ButtonOptions},
        checkbox::{do_checkbox, CheckboxOptions},
        context::{UIContext, UIID},
        dropdown::{do_dropdown, DropdownOptions},
        layout::{ItemLayoutHorizontalAlignment, ItemLayoutOptions, ItemTextAlignment},
        panel::{Panel, PanelInfo, PANEL_TITLE_BAR_HEIGHT},
        slider::{do_slider, NumberSliderOptions},
        text::{do_text, TextOptions},
        textbox::{do_textbox, TextboxOptions},
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ui-panels".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

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

    let ui_context: &'static RwLock<UIContext> = Box::leak(Box::new(RwLock::new(UIContext::new(
        Box::leak(Box::new(RwLock::new(FontCache::new(
            app.context.ttf_context,
        )))),
        Box::leak(Box::new(FontInfo {
            filepath: args[1].to_string(),
            point_size: 16,
        })),
        Box::leak(Box::new(RwLock::new(_text_cache))),
    ))));

    let mut textboxes_model = HashMap::<String, String>::new();

    textboxes_model.insert("1_textbox".to_string(), "ABC 123".to_string());
    textboxes_model.insert("1_slider".to_string(), "0".to_string());
    textboxes_model.insert("1_dropdown".to_string(), "Item 1".to_string());

    textboxes_model.insert("2_textbox".to_string(), "o-blah-dee-o-blah-dah".to_string());
    textboxes_model.insert("2_slider".to_string(), "0.5".to_string());
    textboxes_model.insert("2_dropdown".to_string(), "Item 4".to_string());

    let mut checkboxes_model = HashMap::<String, bool>::new();

    let render_rwl = RwLock::new(
        |panel_info: &PanelInfo,
         panel_buffer: &mut Buffer2D,
         app: &mut App,
         keyboard_state: &KeyboardState,
         mouse_state: &MouseState|
         -> Result<(), String> {
            let mut ctx: RwLockWriteGuard<'_, UIContext> = ui_context.write().unwrap();

            // Draw a bordered button.

            let button_options = ButtonOptions {
                layout_options: ItemLayoutOptions {
                    x_offset: 8,
                    y_offset: PANEL_TITLE_BAR_HEIGHT + 8,
                    ..Default::default()
                },
                label: format!("Bordered button").to_string(),
                with_border: true,
                ..Default::default()
            };

            let button_1_id = UIID {
                parent: panel_info.id,
                item: 1,
                index: 0,
            };

            if do_button(
                &mut ctx,
                button_1_id,
                panel_info,
                panel_buffer,
                mouse_state,
                &button_options,
            )
            .was_released
            {
                println!("You clicked a Button ({}).", button_1_id);
            }

            // Draw a borderless button.

            let button_2_id = UIID {
                parent: panel_info.id,
                item: 2,
                index: 0,
            };

            if do_button(
                &mut ctx,
                button_2_id,
                panel_info,
                panel_buffer,
                mouse_state,
                &ButtonOptions {
                    layout_options: ItemLayoutOptions {
                        y_offset: button_options.layout_options.y_offset + 24,
                        ..button_options.layout_options
                    },
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
                layout_options: ItemLayoutOptions {
                    y_offset: button_options.layout_options.y_offset + 48,
                    ..button_options.layout_options
                },
                label: format!("Checkbox {}", panel_info.id).to_string(),
                ..Default::default()
            };

            let checkbox_model_key = panel_info.id.to_string() + "_checkbox";

            checkboxes_model
                .entry(checkbox_model_key.clone())
                .or_default();

            let checkbox_model_entry = checkboxes_model.entry(checkbox_model_key.clone());

            let checkbox_id = UIID {
                parent: panel_info.id,
                item: 3,
                index: 0,
            };

            if do_checkbox(
                &mut ctx,
                checkbox_id,
                panel_info,
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
                    "The Checkbox ({}) is now {}!",
                    checkbox_id,
                    if *is_checked { "checked" } else { "unchecked" }
                );
            }

            // Draw some cached text labels.

            let text_options = TextOptions {
                layout_options: ItemLayoutOptions {
                    y_offset: checkbox_options.layout_options.y_offset + 24,
                    ..button_options.layout_options
                },
                text: format!("Welcome to Panel {}!", panel_info.id),
                color: color::WHITE,
                ..Default::default()
            };

            do_text(
                &mut ctx,
                UIID {
                    parent: panel_info.id,
                    item: 4,
                    index: 0,
                },
                panel_info,
                panel_buffer,
                &text_options,
            );

            do_text(
                &mut ctx,
                UIID {
                    parent: panel_info.id,
                    item: 5,
                    index: 0,
                },
                panel_info,
                panel_buffer,
                &TextOptions {
                    layout_options: ItemLayoutOptions {
                        y_offset: text_options.layout_options.y_offset + 24,
                        horizontal_alignment: ItemLayoutHorizontalAlignment::Center,
                        ..text_options.layout_options
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
                UIID {
                    parent: panel_info.id,
                    item: 6,
                    index: 0,
                },
                panel_info,
                panel_buffer,
                &TextOptions {
                    layout_options: ItemLayoutOptions {
                        y_offset: text_options.layout_options.y_offset + 48,
                        horizontal_alignment: ItemLayoutHorizontalAlignment::Right,
                        ..text_options.layout_options
                    },
                    text: format!("Uptime: {}", uptime.to_string()),
                    cache: false,
                    color: color::GREEN,
                    ..text_options
                },
            );

            // Draw a textbox.

            let textbox_options = TextboxOptions {
                layout_options: ItemLayoutOptions {
                    y_offset: text_options.layout_options.y_offset + 72,
                    ..text_options.layout_options
                },
                label: format!("Textbox {}", panel_info.id).to_string(),
                input_text_alignment: ItemTextAlignment::Left,
                ..Default::default()
            };

            let textbox_model_key = panel_info.id.to_string() + "_textbox";

            textboxes_model
                .entry(textbox_model_key.clone())
                .or_default();

            let textbox_model_entry = textboxes_model.entry(textbox_model_key.clone());

            let textbox_id = UIID {
                parent: panel_info.id,
                item: 7,
                index: 0,
            };

            if do_textbox(
                &mut ctx,
                textbox_id,
                panel_info,
                panel_buffer,
                app.timing_info.uptime_seconds,
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
                layout_options: ItemLayoutOptions {
                    y_offset: textbox_options.layout_options.y_offset + 24,
                    ..textbox_options.layout_options
                },
                label: format!("Slider {}", panel_info.id).to_string(),
                min: Some(-1.0 * panel_info.id as f32),
                max: Some(1.0 * panel_info.id as f32),
                ..Default::default()
            };

            let slider_model_key = panel_info.id.to_string() + "_slider";

            textboxes_model.entry(slider_model_key.clone()).or_default();

            let slider_model_entry = textboxes_model.entry(slider_model_key.clone());

            let slider_id = UIID {
                parent: panel_info.id,
                item: 8,
                index: 0,
            };

            if do_slider(
                &mut ctx,
                slider_id,
                panel_info,
                panel_buffer,
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
                layout_options: ItemLayoutOptions {
                    y_offset: slider_options.layout_options.y_offset + 24,
                    ..slider_options.layout_options
                },
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

            let dropdown_id = UIID {
                parent: panel_info.id,
                item: 9,
                index: 0,
            };

            if do_dropdown(
                &mut ctx,
                dropdown_id,
                panel_info,
                panel_buffer,
                mouse_state,
                &dropdown_options,
                dropdown_model_entry,
            )
            .did_edit
            {
                println!("You edited a Dropdown ({})!", slider_id);
            }

            // Draw a filled rectangle.

            Graphics::rectangle(
                panel_buffer,
                dropdown_options.layout_options.x_offset,
                dropdown_options.layout_options.y_offset + 24,
                64,
                64,
                color::WHITE,
                Some(color::BLUE),
            );

            Ok(())
        },
    );

    let render_rwl_option = Some(&render_rwl);

    let mut root_panel = Panel::new(
        PanelInfo {
            title: "Panel 0".to_string(),
            width: window_info.window_width,
            height: window_info.window_height,
            ..Default::default()
        },
        render_rwl_option,
    );

    root_panel.split()?;

    let root_panel_rc = RefCell::new(root_panel);

    let current_mouse_state: RwLock<MouseState> = RwLock::new(Default::default());

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
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
