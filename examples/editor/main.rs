extern crate sdl2;

use std::{cell::RefCell, env, rc::Rc};

use sdl2::mouse::{Cursor, MouseButton};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1920_BY_1080},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseEventKind, MouseState},
    },
    font::cache::FontCache,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        panel::PanelInstanceData,
        ui_box::{
            tree::{UIBoxTree, UIBoxTreeRenderCallback},
            utils::text_box,
            UIBox,
        },
        window::{Window, WindowList, WindowOptions, DEFAULT_WINDOW_FILL_COLOR},
    },
};

use editor::panel::{build_floating_window_panel_tree, EditorPanelMetadataMap, EditorPanelType};

pub mod editor;

fn main() -> Result<(), String> {
    let title = format!("Cairo Engine (build {})", env!("GIT_COMMIT_SHORT_HASH")).to_string();

    // Main window info.

    let mut window_info = AppWindowInfo {
        title,
        window_resolution: RESOLUTION_1920_BY_1080,
        canvas_resolution: RESOLUTION_1920_BY_1080,
        resizable: true,
        ..Default::default()
    };

    // Default render framebuffer.

    let framebuffer_rc = RefCell::new(Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    ));

    let current_frame_index_rc = RefCell::new(0_u32);

    // Panel metadata (with rendering callbacks).

    let render_main_window_header: UIBoxTreeRenderCallback =
        Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                ctx.fill_color(DEFAULT_WINDOW_FILL_COLOR, || {
                    ctx.border_color(color::BLACK, || {
                        editor::ui::build_main_menu_bar_ui(ctx, tree)?;
                        editor::ui::build_toolbar_ui(ctx, tree)
                    })
                })
            })?;

            Ok(())
        });

    let panel_metadata_map = EditorPanelMetadataMap {
        outline: PanelInstanceData {
            panel_type: EditorPanelType::Outline,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "Outline".to_string()))?;

                Ok(())
            })),
        },
        viewport3d: PanelInstanceData {
            panel_type: EditorPanelType::Viewport3D,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "Viewport3D".to_string()))?;

                Ok(())
            })),
        },
        asset_browser: PanelInstanceData {
            panel_type: EditorPanelType::AssetBrowser,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "AssetBrowser".to_string()))?;

                Ok(())
            })),
        },
        console: PanelInstanceData {
            panel_type: EditorPanelType::Console,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "Console".to_string()))?;

                Ok(())
            })),
        },
        inspector: PanelInstanceData {
            panel_type: EditorPanelType::Inspector,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "Inspector".to_string()))?;

                Ok(())
            })),
        },
        file_system: PanelInstanceData {
            panel_type: EditorPanelType::FileSystem,
            render_callback: Some(Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
                tree.push(text_box(String::new(), "FileSystem".to_string()))?;

                Ok(())
            })),
        },
    };

    // Initial main window.

    let window_list = {
        let mut list: WindowList<EditorPanelType> = Default::default();

        let main_window_id = "main_window".to_string();

        let main_window_panel_tree =
            editor::panel::build_main_window_panel_tree(&main_window_id, &panel_metadata_map)
                .unwrap();

        let main_window = Window::new(
            main_window_id,
            WindowOptions {
                docked: true,
                size: (
                    window_info.window_resolution.width,
                    window_info.window_resolution.height,
                ),
                ..Default::default()
            },
            Some(render_main_window_header),
            main_window_panel_tree,
        );

        list.push_back(main_window);

        for i in 0..15 {
            let floating_window_id = format!("floating_window_{}", i);

            let floating_window_panel_tree =
                build_floating_window_panel_tree(&floating_window_id, &panel_metadata_map.console)?;

            let floating_window = Window::new(
                floating_window_id,
                WindowOptions {
                    docked: false,
                    with_titlebar: true,
                    size: (236, 178),
                    position: (100 + i * 36, 100 + i * 36),
                },
                None,
                floating_window_panel_tree,
            );

            list.push_back(floating_window);
        }

        list
    };

    let window_list_rc = Rc::new(RefCell::new(window_list));

    // SDL will reset the cursor if we don't retain the result from
    // Cursor::set().

    let set_cursor_result_rc: RefCell<Option<Cursor>> = Default::default();

    // Primary function for rendering the UI tree to `framebuffer`; this
    // function is called when either (1) the main loop executes, or (2) the
    // user is actively resizing the main application window.

    let render_window_list_to_framebuffer = |_frame_index: Option<u32>,
                                             new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        let frame_index = current_frame_index_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        let mut window_list = (*window_list_rc).borrow_mut();

        if let Some(resolution) = &new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);

            // Rebuild the UI tree based on the new window (root) resolution.

            render_window_list(&mut window_list, resolution);
        }

        framebuffer.clear(Some(color::BLACK.to_u32()));

        // Check if our application window was just resized.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            {
                // Reset cursor for this frame.

                *ctx.cursor_kind.borrow_mut() = MouseCursorKind::Arrow;
            }

            for window in window_list.iter() {
                let base_ui_tree = &mut window.ui_trees.base.borrow_mut();

                // Render the current frame's UI tree to `framebuffer`.

                base_ui_tree
                    .render_frame(*frame_index, &mut framebuffer)
                    .unwrap();
            }

            {
                let kind = &ctx.cursor_kind.borrow();

                *set_cursor_result_rc.borrow_mut() = Some(mouse::cursor::set_cursor(kind).unwrap());
            }
        });

        Ok(framebuffer.get_all().clone())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_window_list_to_framebuffer);

    // Set the global font info, based on the font filepath that was passed to
    // our program.

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = ctx.font_info.borrow_mut();

            let args: Vec<String> = env::args().collect();

            if args.len() < 2 {
                panic!("Usage: cargo run --example editor /path/to/your-font.fon");
            }

            let font_filepath = args[1].to_string();

            font_info.filepath = font_filepath;
            font_info.point_size = 14;
        }
    });

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
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

        let resolution = &(*app.window_info).borrow().window_resolution;

        let mut window_list = window_list_rc.borrow_mut();

        render_window_list(&mut window_list, resolution);

        Ok(())
    };

    let render = |frame_index: Option<u32>,
                  new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        if let Some(index) = frame_index {
            let mut current_frame_index = current_frame_index_rc.borrow_mut();

            *current_frame_index = index;

            // Prune old entries from our UI cache.

            GLOBAL_UI_CONTEXT.with(|ctx| {
                let mut cache = ctx.cache.borrow_mut();

                cache.retain(|_key, ui_box: &mut UIBox| ui_box.last_read_at_frame == index);
            });
        }

        render_window_list_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn render_window_list(window_list: &mut WindowList<EditorPanelType>, resolution: &Resolution) {
    let mut focused_window = None;

    {
        let mut cursor = window_list.cursor_mut();

        while let Some(window) = cursor.peek_prev() {
            let mut did_focus = false;

            // Check if we should capture the current mouse event for this
            // window, exclusively.

            GLOBAL_UI_CONTEXT.with(|ctx| {
                let mouse = &ctx.input_events.borrow().mouse;

                if focused_window.is_none()
                    && window.active
                    && window
                        .extent
                        .contains(mouse.position.0 as u32, mouse.position.1 as u32)
                {
                    if let Some(event) = mouse.button_event {
                        if matches!(
                            (event.button, event.kind),
                            (MouseButton::Left, MouseEventKind::Down)
                        ) {
                            did_focus = true;
                        }
                    }
                }
            });

            GLOBAL_UI_CONTEXT.with(|ctx| {
                // Rebuild the UI tree based on the latest user inputs.
                window.render_ui_trees(ctx, resolution).unwrap();
            });

            if did_focus && cursor.index() != Some(1) {
                // Take the focused window out of the window list (temporarily).
                focused_window.replace(cursor.remove_prev().unwrap());

                GLOBAL_UI_CONTEXT.with(|ctx| {
                    // Steal the mouse event used to focus the window.
                    let mut input_events = ctx.input_events.borrow_mut();

                    input_events.mouse.button_event.take();
                });
            }

            // Advance the window cursor.
            cursor.move_prev();
        }
    }

    if let Some(window) = focused_window {
        // Re-insert the focused window at the end of the window list.
        window_list.push_back(window);
    }

    window_list.retain(|window| window.active);
}
