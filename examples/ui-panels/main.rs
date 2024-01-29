extern crate sdl2;

use std::{cell::RefCell, collections::HashMap, env, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
    ui::{
        button::{do_button, ButtonOptions},
        checkbox::{do_checkbox, CheckboxOptions},
        context::{UIContext, UIID},
        layout::{ItemLayoutHorizontalAlignment, ItemLayoutOptions},
        panel::{Panel, PanelInfo, PANEL_TITLE_BAR_HEIGHT},
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

    let font_info: &'static FontInfo = Box::leak(Box::new(FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    }));

    // Create a static font cache.

    let font_cache: &'static mut RwLock<FontCache<'static>> = Box::leak(Box::new(RwLock::new(
        FontCache::new(app.context.ttf_context),
    )));

    // Create a static text (texture) cache.

    let _text_cache: TextCache = Default::default();

    let text_cache: &'static mut RwLock<TextCache<'static>> =
        Box::leak(Box::new(RwLock::new(_text_cache)));

    // Set up our app

    let mut framebuffer = Buffer2D::new(window_info.window_width, window_info.window_height, None);

    let ui_context: &'static RwLock<UIContext> = Box::leak(Box::new(Default::default()));

    let mut textboxes_model = HashMap::<String, String>::new();

    textboxes_model.insert("1_textbox".to_string(), "ABC 123".to_string());
    textboxes_model.insert("2_textbox".to_string(), "o-blah-dee-o-blah-dah".to_string());

    let mut checkboxes_model = HashMap::<String, bool>::new();

    let render_rwl = RwLock::new(
        |info: &PanelInfo,
         buffer: &mut Buffer2D,
         app: &mut App,
         keyboard_state: &KeyboardState,
         mouse_state: &MouseState|
         -> Result<(), String> {
            // Clear the panel buffer for drawing.

            buffer.clear(None);

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
                parent: info.id,
                item: 1,
                index: 0,
            };

            if do_button(
                ui_context,
                button_1_id,
                info,
                buffer,
                mouse_state,
                font_cache,
                text_cache,
                font_info,
                &button_options,
            )
            .was_released
            {
                println!("You clicked a Button ({}).", button_1_id);
            }

            // Draw a borderless button.

            let button_2_id = UIID {
                parent: info.id,
                item: 2,
                index: 0,
            };

            if do_button(
                ui_context,
                button_2_id,
                info,
                buffer,
                mouse_state,
                font_cache,
                text_cache,
                font_info,
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
                label: format!("Checkbox {}", info.id).to_string(),
                ..Default::default()
            };

            let checkbox_model_key = info.id.to_string() + "_checkbox";

            checkboxes_model
                .entry(checkbox_model_key.clone())
                .or_default();

            let checkbox_model_entry = checkboxes_model.entry(checkbox_model_key.clone());

            let checkbox_id = UIID {
                parent: info.id,
                item: 3,
                index: 0,
            };

            if do_checkbox(
                ui_context,
                checkbox_id,
                info,
                buffer,
                mouse_state,
                font_cache,
                text_cache,
                font_info,
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
                text: format!("Welcome to Panel {}!", info.id),
                color: color::WHITE,
                ..Default::default()
            };

            do_text(
                ui_context,
                UIID {
                    parent: info.id,
                    item: 4,
                    index: 0,
                },
                info,
                buffer,
                font_cache,
                text_cache,
                font_info,
                &text_options,
            );

            do_text(
                ui_context,
                UIID {
                    parent: info.id,
                    item: 5,
                    index: 0,
                },
                info,
                buffer,
                font_cache,
                text_cache,
                font_info,
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
                ui_context,
                UIID {
                    parent: info.id,
                    item: 6,
                    index: 0,
                },
                info,
                buffer,
                font_cache,
                text_cache,
                font_info,
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
                label: format!("Textbox {}", info.id).to_string(),
                ..Default::default()
            };

            let textbox_model_key = info.id.to_string() + "_textbox";

            textboxes_model
                .entry(textbox_model_key.clone())
                .or_default();

            let textbox_model_entry = textboxes_model.entry(textbox_model_key.clone());

            let textbox_id = UIID {
                parent: info.id,
                item: 7,
                index: 0,
            };

            if do_textbox(
                ui_context,
                textbox_id,
                info,
                buffer,
                app.timing_info.uptime_seconds,
                keyboard_state,
                mouse_state,
                font_cache,
                text_cache,
                font_info,
                &textbox_options,
                textbox_model_entry,
            )
            .did_edit
            {
                println!("You edited a Textbox ({})!", textbox_id);
            }

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

    root_panel.split(0.5)?;

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
            .render(
                app,
                keyboard_state,
                mouse_state,
                ui_context,
                font_cache,
                text_cache,
                font_info,
            )
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
