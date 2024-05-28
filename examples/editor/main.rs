extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    animation::lerp,
    app::{resolution::RESOLUTION_1600_BY_900, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::cache::FontCache,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
        UISize, UISizeWithStrictness,
    },
};

pub mod editor;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/editor".to_string(),
        window_resolution: RESOLUTION_1600_BY_900,
        canvas_resolution: RESOLUTION_1600_BY_900,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example editor /path/to/your-font.fon");
        return Ok(());
    }

    let font_filepath = args[1].to_string();

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = ctx.font_info.borrow_mut();

            font_info.filepath = font_filepath;
            font_info.point_size = 14;
        }
    });

    // Set up our app

    let mut framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    // Main panel tree.

    let main_panel_tree = editor::panel::build_panel_tree()?;

    let main_panel_tree_rc = RefCell::new(main_panel_tree);

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;
        let seconds_since_last_update = app.timing_info.seconds_since_last_update;

        // Recreate the UI tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                ctx.clear_for_next_frame();

                let tree = &mut ctx.tree.borrow_mut();

                // println!("\nRebuilding tree...\n");

                let alpha_x = uptime.sin() / 2.0 + 0.5;
                let alpha_y = uptime.cos() / 2.0 + 0.5;

                ctx.fill_color(color::WHITE, || {
                    tree.push_parent(UIBox::new(
                        "Root__root".to_string(),
                        UIBoxFeatureMask::none()
                            | UIBoxFeatureFlag::DrawFill
                            | UIBoxFeatureFlag::DrawChildDividers,
                        UILayoutDirection::TopToBottom,
                        [
                            UISizeWithStrictness {
                                size: UISize::Pixels(lerp(
                                    window_info.window_resolution.width as f32 * 0.925,
                                    window_info.window_resolution.width as f32,
                                    alpha_x,
                                ) as u32),
                                strictness: 1.0,
                            },
                            UISizeWithStrictness {
                                size: UISize::Pixels(lerp(
                                    window_info.window_resolution.height as f32 * 0.925,
                                    window_info.window_resolution.height as f32,
                                    alpha_y,
                                ) as u32),
                                strictness: 1.0,
                            },
                        ],
                    ))
                })?;
            }

            ctx.fill_color(color::WHITE, || {
                ctx.border_color(color::BLACK, || {
                    editor::ui::build_main_menu_bar_ui(ctx)?;
                    editor::ui::build_toolbar_ui(ctx)
                })
            })?;

            let mut main_panel_tree = main_panel_tree_rc.borrow_mut();

            main_panel_tree.render(ctx)?;

            {
                let tree = &mut ctx.tree.borrow_mut();

                // `Current` is now back at the root...

                tree.do_user_inputs_pass(
                    seconds_since_last_update,
                    keyboard_state,
                    mouse_state,
                    game_controller_state,
                )?;

                tree.do_autolayout_pass()
            }
        })
    };

    let mut render = |frame_index: u32| -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let tree = &mut ctx.tree.borrow_mut();

            tree.render(frame_index, &mut framebuffer).unwrap();
        });

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
