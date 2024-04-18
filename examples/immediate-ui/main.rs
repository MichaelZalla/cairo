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
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    texture::map::{TextureMap, TextureMapStorageFormat},
    ui::{
        context::{UIContext, UIID},
        layout::{
            UILayoutContext, UILayoutDirection, UILayoutExtent,
        },
        panel::{do_panel, PanelOptions, PanelTitlebarOptions},
    },
};

use uuid::Uuid;

mod draw_sample_panel_contents;

use draw_sample_panel_contents::draw_sample_panel_contents;

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
        TextureMapStorageFormat::Index8(0),
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
                                closable: true,
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
