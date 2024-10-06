extern crate sdl2;

use sdl2::mouse::Cursor;

use std::{cell::RefCell, env};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    ui::{context::GLOBAL_UI_CONTEXT, ui_box::tree::UIBoxTree},
};

use font::load_system_font;
use ui_tree::build_ui_tree;

mod font;
mod ui_tree;

fn retain_cursor(cursor_kind: &MouseCursorKind, retained: &mut Option<Cursor>) {
    let cursor = mouse::cursor::set_cursor(cursor_kind).unwrap();

    retained.replace(cursor);
}

fn main() -> Result<(), String> {
    // Validates command line arguments.

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example immediate-ui /path/to/your-font.fon");
        return Ok(());
    }

    // Describes our application's window.

    let mut window_info = AppWindowInfo {
        title: "examples/immediate-ui".to_string(),
        window_resolution: RESOLUTION_1280_BY_720,
        canvas_resolution: RESOLUTION_1280_BY_720,
        resizable: true,
        ..Default::default()
    };

    // Allocates a default framebuffer.

    let framebuffer_rc = RefCell::new(Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    ));

    // Allocates a single global UI tree.

    let ui_box_tree_rc = RefCell::new(UIBoxTree::default());

    // We'll need to remember the index of the last rendered frame.

    // @TODO Why not have the `App` track this?!
    let current_frame_index_rc = RefCell::new(0_u32);

    // We need to retain a reference to each `Cursor` that we set through SDL.
    // Without this reference, the SDL cursor is immediately dropped
    // (deallocated), and we won't see our custom cursor take effect.

    let retained_cursor_rc: RefCell<Option<Cursor>> = Default::default();

    // Primary function for rendering the UI tree to `framebuffer`; this
    // function is called when either (1) the main loop executes, or (2) the
    // user is actively resizing the main (native) application window.

    let render = |frame_index: Option<u32>,
                  new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        if let Some(index) = frame_index {
            // Cache the index of the last-rendered frame.

            let mut current_frame_index = current_frame_index_rc.borrow_mut();

            *current_frame_index = index;

            // Prune old UI cache entries (with respect to this frame's index).

            GLOBAL_UI_CONTEXT.with(|ctx| {
                ctx.prune_cache(index);
            });
        }

        let frame_index = *current_frame_index_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();
        let mut tree = ui_box_tree_rc.borrow_mut();
        let mut retained_cursor = retained_cursor_rc.borrow_mut();

        // Check if our application window was just resized...

        if let Some(resolution) = new_resolution {
            // Resize our framebuffer to match the native window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);

            // Rebuild our UI tree, in response to the new resolution.

            GLOBAL_UI_CONTEXT.with(|ctx| {
                tree.clear();

                build_ui_tree(ctx, &mut tree, resolution)?;

                tree.commit_frame()
            })?;
        }

        framebuffer.clear(None);

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                // Reset cursor for this frame.
                *ctx.cursor_kind.borrow_mut() = MouseCursorKind::Arrow;
            }

            {
                // Render our current UI tree into the framebuffer.

                tree.render_frame(frame_index, &mut framebuffer).unwrap();
            }

            {
                let cursor_kind = ctx.cursor_kind.borrow();

                retain_cursor(&cursor_kind, &mut retained_cursor);
            }
        });

        Ok(framebuffer.get_all().clone())
    };

    // Instantiate our app, using the rendering callback we defined above.

    let (app, _event_watch) = App::new(&mut window_info, &render);

    // Load the font indicated by the CLI argument(s).

    load_system_font(&app, args[1].to_string());

    // Define `update()` in the context of our app's main loop.

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        // Binds the latest user inputs (and time delta) to the global UI context.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            // Bind the latest user input events.

            ctx.set_user_inputs(keyboard_state, mouse_state, game_controller_state);

            // Binds the latest seconds-since-last-update.

            ctx.set_seconds_since_last_update(app.timing_info.seconds_since_last_update);
        });

        // Recreate the UI tree for this update.

        let window_info = app.window_info.borrow();

        let resolution = window_info.window_resolution;

        let mut result = Ok(());

        GLOBAL_UI_CONTEXT.with(|ctx| {
            result = ctx.fill_color(color::WHITE, || {
                ctx.border_color(color::BLACK, || {
                    let mut tree = ui_box_tree_rc.borrow_mut();

                    tree.clear();

                    build_ui_tree(ctx, &mut tree, resolution)?;

                    tree.commit_frame()
                })
            });
        });

        result
    };

    // Start the main loop...

    app.run(&mut update, &render)?;

    Ok(())
}
