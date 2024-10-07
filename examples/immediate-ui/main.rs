extern crate sdl2;

use sdl2::mouse::Cursor;

use std::{cell::RefCell, env, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    mem::linked_list::LinkedList,
    resource::{arena::Arena, handle::Handle},
    ui::{context::GLOBAL_UI_CONTEXT, panel::PanelRenderCallback, ui_box::tree::UIBoxTree},
};

use command::{Command, CommandBuffer};
use font::load_system_font;
use panel::{PanelInstance, SettingsPanel};
use settings::Settings;
use window::make_window_list;

mod command;
mod font;
mod panel;
mod settings;
mod window;

thread_local! {
    pub static SETTINGS: Settings = Default::default();
    pub static COMMAND_BUFFER: CommandBuffer = Default::default();
}

fn retain_cursor(cursor_kind: &MouseCursorKind, retained: &mut Option<Cursor>) {
    let cursor = mouse::cursor::set_cursor(cursor_kind).unwrap();

    retained.replace(cursor);
}

fn process_command(command: Command) -> Result<(), String> {
    if command.kind == "set_setting" {
        let (setting_key, new_value) = (&command.args[0], &command.args[1]);

        if setting_key == "clicked_count" {
            SETTINGS.with(|settings| {
                *settings.clicked_count.borrow_mut() = new_value.parse::<usize>().unwrap();
            });
        }
    }

    Ok(())
}

fn process_commands(
    pending_commands: &mut LinkedList<String>,
    executed_commands: &mut LinkedList<String>,
) -> Result<(), String> {
    while let Some(cmd) = pending_commands.pop_front() {
        let components: Vec<String> = cmd.split(' ').map(|s| s.to_string()).collect();

        if let Some((kind, args)) = components.split_first() {
            process_command(Command { kind, args })?;
        }

        executed_commands.push_back(cmd);
    }

    Ok(())
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

    // Builds a list of windows containing our UI.

    let settings_panel_arena_rc = Box::leak(Box::new(RefCell::new(Arena::<SettingsPanel>::new())));

    let settings_panel_render_callback: PanelRenderCallback = Rc::new(
        |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
            let mut settings_panel_arena = settings_panel_arena_rc.borrow_mut();

            if let Ok(entry) = settings_panel_arena.get_mut(panel_instance) {
                let panel = &mut entry.item;

                panel.render(tree).unwrap();
            }

            Ok(())
        },
    );

    let window_list_rc = {
        let mut settings_panel_arena = settings_panel_arena_rc.borrow_mut();
        let resolution = window_info.window_resolution;

        let list = make_window_list(
            &mut settings_panel_arena,
            settings_panel_render_callback,
            resolution,
        )?;

        Rc::new(RefCell::new(list))
    };

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
        let mut window_list = window_list_rc.borrow_mut();
        let mut retained_cursor = retained_cursor_rc.borrow_mut();

        // Check if our application window was just resized...

        if let Some(resolution) = new_resolution {
            // Resize our framebuffer to match the native window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);

            // Rebuild each window's UI tree(s), in response to the new (native
            // window) resolution.

            window_list.rebuild_ui_trees(resolution);
        }

        framebuffer.clear(None);

        GLOBAL_UI_CONTEXT.with(|ctx| {
            window_list.render(frame_index, &mut framebuffer).unwrap();

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
        // Processes any pending commands.

        COMMAND_BUFFER.with(|buffer| {
            let mut pending_commands = buffer.pending_commands.borrow_mut();
            let mut executed_commands = buffer.executed_commands.borrow_mut();

            process_commands(&mut pending_commands, &mut executed_commands).unwrap();
        });

        // Binds the latest user inputs (and time delta) to the global UI context.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            // Resets the cursor style.
            ctx.begin_frame();

            // Bind the latest user input events.
            ctx.set_user_inputs(keyboard_state, mouse_state, game_controller_state);

            // Binds the latest seconds-since-last-update.
            ctx.set_seconds_since_last_update(app.timing_info.seconds_since_last_update);
        });

        // Rebuilds the UI trees for each window in our window list.

        let current_window_info = app.window_info.borrow();

        let current_resolution = current_window_info.window_resolution;

        let mut window_list = window_list_rc.borrow_mut();

        window_list.rebuild_ui_trees(current_resolution);

        Ok(())
    };

    // Start the main loop...

    app.run(&mut update, &render)?;

    Ok(())
}
