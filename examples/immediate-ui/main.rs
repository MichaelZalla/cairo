extern crate sdl2;

use std::{cell::RefCell, env, rc::Rc};

use sdl2::{
    keyboard::{Keycode, Mod},
    mouse::Cursor,
};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTIONS_16X9, RESOLUTION_1280_BY_720},
        window::AppWindowingMode,
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    resource::{arena::Arena, handle::Handle},
    ui::{
        context::GLOBAL_UI_CONTEXT, panel::PanelRenderCallback, ui_box::tree::UIBoxTree,
        window::list::WindowList,
    },
};

use command::{process_commands, CommandBuffer};
use panels::{
    render_options_panel::RenderOptionsPanel, settings_panel::SettingsPanel,
    shader_options_panel::ShaderOptionsPanel, PanelInstance,
};
use settings::Settings;
use window::make_window_list;

mod checkbox;
mod command;
mod panels;
mod radio;
mod settings;
mod stack;
mod window;

thread_local! {
    pub static SETTINGS: RefCell<Settings> = Default::default();
    pub static COMMAND_BUFFER: CommandBuffer = Default::default();
}

static DEFAULT_WINDOW_RESOLUTION: Resolution = RESOLUTION_1280_BY_720;

fn retain_cursor(cursor_kind: &MouseCursorKind, retained: &mut Option<Cursor>) {
    let cursor = mouse::cursor::set_cursor(cursor_kind).unwrap();

    retained.replace(cursor);
}

fn resize_framebuffer(
    resolution: Resolution,
    framebuffer: &mut Framebuffer,
    window_list: &mut WindowList,
) {
    // Resize our framebuffer to match the native window's new resolution.

    framebuffer.resize(resolution.width, resolution.height, true);

    // Rebuild each window's UI tree(s), in response to the new (native
    // window) resolution.

    window_list.rebuild_ui_trees(resolution);
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
        window_resolution: DEFAULT_WINDOW_RESOLUTION,
        canvas_resolution: DEFAULT_WINDOW_RESOLUTION,
        resizable: true,
        ..Default::default()
    };

    SETTINGS.with(|settings_rc| {
        let mut settings = settings_rc.borrow_mut();

        settings.resolution = RESOLUTIONS_16X9
            .iter()
            .position(|r| *r == DEFAULT_WINDOW_RESOLUTION)
            .unwrap();

        settings.hdr = true;
    });

    // Allocates a default framebuffer.

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

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

    let render_options_panel_arena_rc =
        Box::leak(Box::new(RefCell::new(Arena::<RenderOptionsPanel>::new())));

    let render_options_panel_render_callback: PanelRenderCallback = Rc::new(
        |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
            let mut render_options_panel_arena = render_options_panel_arena_rc.borrow_mut();

            if let Ok(entry) = render_options_panel_arena.get_mut(panel_instance) {
                let panel = &mut entry.item;

                panel.render(tree).unwrap();
            }

            Ok(())
        },
    );

    let shader_options_panel_arena_rc =
        Box::leak(Box::new(RefCell::new(Arena::<ShaderOptionsPanel>::new())));

    let shader_options_panel_render_callback: PanelRenderCallback = Rc::new(
        |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
            let mut shader_options_panel_arena = shader_options_panel_arena_rc.borrow_mut();

            if let Ok(entry) = shader_options_panel_arena.get_mut(panel_instance) {
                let panel = &mut entry.item;

                panel.render(tree).unwrap();
            }

            Ok(())
        },
    );

    let window_list_rc = {
        let mut settings_panel_arena = settings_panel_arena_rc.borrow_mut();
        let mut render_options_panel_arena = render_options_panel_arena_rc.borrow_mut();
        let mut shader_options_panel_arena = shader_options_panel_arena_rc.borrow_mut();

        let resolution = window_info.window_resolution;

        let list = make_window_list(
            &mut settings_panel_arena,
            settings_panel_render_callback,
            &mut render_options_panel_arena,
            render_options_panel_render_callback,
            &mut shader_options_panel_arena,
            shader_options_panel_render_callback,
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
            resize_framebuffer(resolution, &mut framebuffer, &mut window_list);
        }

        let mut color_buffer = framebuffer.attachments.color.as_mut().unwrap().borrow_mut();

        GLOBAL_UI_CONTEXT.with(|ctx| {
            window_list.render(frame_index, &mut color_buffer).unwrap();

            {
                let cursor_kind = ctx.cursor_kind.borrow();

                retain_cursor(&cursor_kind, &mut retained_cursor);
            }
        });

        Ok(color_buffer.get_all().clone())
    };

    // Instantiate our app, using the rendering callback we defined above.

    let (app, _event_watch) = App::new(&mut window_info, &render);

    // Load the font indicated by the CLI argument(s).

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.load_font(&app, args[1].to_string(), 12);
    });

    // Define `update()` in the context of our app's main loop.

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        // Check if the app's native window has been resized.

        {
            let window_info = app.window_info.borrow();
            let mut framebuffer = framebuffer_rc.borrow_mut();

            if window_info.window_resolution.width != framebuffer.width
                || window_info.window_resolution.height != framebuffer.height
            {
                // Resize our framebuffer to match the new window resolution.

                let mut canvas = app.context.rendering_context.canvas.borrow_mut();
                let window = canvas.window_mut();
                let mut window_list = window_list_rc.borrow_mut();

                resize_framebuffer(
                    Resolution::new(window.size()),
                    &mut framebuffer,
                    &mut window_list,
                );
            }
        }

        // Processes any pending commands.

        COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
            let new_resolution: Option<Resolution>;
            let new_windowing_mode: Option<AppWindowingMode>;

            {
                let mut pending_commands = buffer.pending_commands.borrow_mut();
                let mut executed_commands = buffer.executed_commands.borrow_mut();

                // Extract keyboard shortcut commands.

                keyboard_state
                    .keys_pressed
                    .retain(|(keycode, modifiers)| match keycode {
                        #[cfg(debug_assertions)]
                        Keycode::F7 => {
                            GLOBAL_UI_CONTEXT.with(|ctx| {
                                let mut debug_options = ctx.debug.borrow_mut();

                                debug_options.draw_box_boundaries =
                                    !debug_options.draw_box_boundaries;
                            });

                            false
                        }
                        Keycode::Z => {
                            if modifiers.contains(Mod::LCTRLMOD)
                                || modifiers.contains(Mod::RCTRLMOD)
                            {
                                if let Some(executed_command) = executed_commands.pop_back() {
                                    let (new_pending_command, is_undo) = {
                                        if modifiers.contains(Mod::LSHIFTMOD)
                                            | modifiers.contains(Mod::RSHIFTMOD)
                                        {
                                            (
                                                format!(
                                                    "{} {}",
                                                    executed_command.kind,
                                                    executed_command.args.join(" ")
                                                ),
                                                false,
                                            )
                                        } else if let Some(prev_value) = executed_command.prev_value
                                        {
                                            (
                                                format!(
                                                    "{} {} {}",
                                                    executed_command.kind,
                                                    executed_command.args[0],
                                                    prev_value
                                                )
                                                .to_string(),
                                                true,
                                            )
                                        } else {
                                            panic!()
                                        }
                                    };

                                    pending_commands.push_back((new_pending_command, is_undo));
                                }

                                false
                            } else {
                                true
                            }
                        }
                        Keycode::V => {
                            if modifiers.contains(Mod::LCTRLMOD)
                                || modifiers.contains(Mod::RCTRLMOD)
                            {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let vsync = current_settings.vsync;

                                    let cmd_str = format!(
                                        "set_setting vsync {}",
                                        if vsync { "false" } else { "true " }
                                    )
                                    .to_string();

                                    pending_commands.push_back((cmd_str, false));
                                });

                                false
                            } else {
                                true
                            }
                        }
                        Keycode::H => {
                            if modifiers.contains(Mod::LCTRLMOD)
                                || modifiers.contains(Mod::RCTRLMOD)
                            {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let hdr = current_settings.hdr;

                                    let cmd_str = format!(
                                        "set_setting hdr {}",
                                        if hdr { "false" } else { "true " }
                                    )
                                    .to_string();

                                    pending_commands.push_back((cmd_str, false));
                                });

                                false
                            } else {
                                true
                            }
                        }
                        Keycode::B => {
                            if modifiers.contains(Mod::LCTRLMOD)
                                || modifiers.contains(Mod::RCTRLMOD)
                            {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let bloom = current_settings.bloom;

                                    let cmd_str = format!(
                                        "set_setting bloom {}",
                                        if bloom { "false" } else { "true " }
                                    )
                                    .to_string();

                                    pending_commands.push_back((cmd_str, false));
                                });

                                false
                            } else {
                                true
                            }
                        }
                        _ => true,
                    });

                (new_resolution, new_windowing_mode) =
                    process_commands(&mut pending_commands, &mut executed_commands).unwrap();
            }

            let mut framebuffer = framebuffer_rc.borrow_mut();
            let mut window_list = window_list_rc.borrow_mut();

            if let Some(resolution) = new_resolution {
                resize_framebuffer(resolution, &mut framebuffer, &mut window_list);

                app.resize_window(resolution)
            } else {
                if let Some(mode) = new_windowing_mode {
                    app.set_windowing_mode(mode)?;

                    let mut canvas = app.context.rendering_context.canvas.borrow_mut();
                    let window = canvas.window_mut();

                    resize_framebuffer(
                        Resolution::new(window.size()),
                        &mut framebuffer,
                        &mut window_list,
                    );
                }

                Ok(())
            }
        })?;

        // Binds the latest user inputs (and time delta) to the global UI context.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            // Resets the cursor style.
            ctx.begin_frame();

            // Bind the latest user input events.
            ctx.set_user_inputs(keyboard_state, mouse_state, game_controller_state);

            // Binds the latest timing info.
            ctx.set_timing_info(&app.timing_info);
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
