extern crate sdl2;

use std::{cell::RefCell, env};

use sdl2::mouse::Cursor;

use cairo::{
    animation::lerp,
    app::{resolution::RESOLUTION_1600_BY_900, App, AppWindowInfo},
    buffer::Buffer2D,
    color::{self, Color},
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    font::cache::FontCache,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
        UISize, UISizeWithStrictness,
    },
};

pub mod editor;

static EDITOR_UI_FILL_COLOR: Color = Color::rgb(230, 230, 230);

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

    // Panel tree.

    let main_panel_tree = editor::panel::build_main_panel_tree()?;
    let main_panel_tree_rc = RefCell::new(main_panel_tree);

    // SDL will reset the cursor if we don't retain the result from
    // Cursor::set().

    let set_cursor_result_rc: RefCell<Option<Cursor>> = Default::default();

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;

        // Recreate the UI tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                ctx.clear_for_next_frame();

                // Bind the latest user input events.

                {
                    let mut input_events = ctx.input_events.borrow_mut();

                    input_events.keyboard = keyboard_state.clone();
                    input_events.mouse = mouse_state.clone();
                    input_events.game_controller = *game_controller_state;
                }

                // Bind delta time.

                {
                    let mut seconds_since_last_update = ctx.seconds_since_last_update.borrow_mut();

                    *seconds_since_last_update = app.timing_info.seconds_since_last_update;
                }

                let tree = &mut ctx.tree.borrow_mut();

                // println!("\nRebuilding tree...\n");

                let alpha_x = uptime.sin() / 2.0 + 0.5;
                let alpha_y = uptime.cos() / 2.0 + 0.5;

                ctx.fill_color(EDITOR_UI_FILL_COLOR, || {
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
                    ))?;

                    Ok(())
                })?;
            }

            ctx.fill_color(EDITOR_UI_FILL_COLOR, || {
                ctx.border_color(color::BLACK, || {
                    editor::ui::build_main_menu_bar_ui(ctx)?;
                    editor::ui::build_toolbar_ui(ctx)
                })
            })?;

            let mut main_panel_tree = main_panel_tree_rc.borrow_mut();

            main_panel_tree.render(ctx)?;

            {
                let tree = &mut ctx.tree.borrow_mut();

                tree.commit_frame()
            }
        })
    };

    let mut render = |frame_index: u32| -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                // Reset cursor for this frame.

                *ctx.cursor_kind.borrow_mut() = MouseCursorKind::Arrow;
            }

            let tree = &mut ctx.tree.borrow_mut();

            // Render the current frame's UI tree to `framebuffer`.

            tree.render_frame(frame_index, &mut framebuffer).unwrap();

            {
                let kind = &ctx.cursor_kind.borrow();

                *set_cursor_result_rc.borrow_mut() = Some(mouse::cursor::set_cursor(kind).unwrap());
            }
        });

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
