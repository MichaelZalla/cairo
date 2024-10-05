extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_960_BY_540},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    ui::{context::GLOBAL_UI_CONTEXT, ui_box::tree::UIBoxTree},
};

use font::load_system_font;
use ui_tree::build_ui_tree;

mod font;
mod ui_tree;

fn main() -> Result<(), String> {
    // Main window info.

    let mut window_info = AppWindowInfo {
        title: "examples/immediate-ui".to_string(),
        window_resolution: RESOLUTION_960_BY_540,
        canvas_resolution: RESOLUTION_960_BY_540,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Validate command line arguments.

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example immediate-ui /path/to/your-font.fon");
        return Ok(());
    }

    // Load our system font.

    let system_font_path = args[1].to_string();

    load_system_font(&app, system_font_path);

    // Default render framebuffer.

    let framebuffer_rc = RefCell::new(Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    ));

    // Global UI tree.

    let ui_box_tree_rc = RefCell::new(UIBoxTree::default());

    // Callbacks.

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        // Binds the latest user inputs and timing info to the UI context.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            // Bind the latest user input events.

            {
                let mut input_events = ctx.input_events.borrow_mut();

                input_events.keyboard = keyboard_state.clone();
                input_events.mouse = mouse_state.clone();
                input_events.game_controller = *game_controller_state;
            }

            // Binds the latest seconds-since-last-update.

            {
                let mut seconds_since_last_update = ctx.seconds_since_last_update.borrow_mut();

                *seconds_since_last_update = app.timing_info.seconds_since_last_update;
            }
        });

        // Recreate the UI tree for this update.

        let mut result = Ok(());

        GLOBAL_UI_CONTEXT.with(|ctx| {
            result = ctx.fill_color(color::WHITE, || {
                ctx.border_color(color::BLACK, || {
                    let mut tree = ui_box_tree_rc.borrow_mut();

                    tree.clear();

                    build_ui_tree(ctx, &mut tree, &window_info)?;

                    tree.commit_frame()
                })
            });
        });

        result
    };

    let render = |frame_index: Option<u32>, _new_resolution| -> Result<Vec<u32>, String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        framebuffer.clear(None);

        {
            // Render our current UI tree into the framebuffer.

            let mut tree = ui_box_tree_rc.borrow_mut();

            tree.render_frame(frame_index.unwrap(), &mut framebuffer)?;
        }

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &render)?;

    Ok(())
}
