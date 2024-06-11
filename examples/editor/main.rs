extern crate sdl2;

use std::{cell::RefCell, env, rc::Rc};

use uuid::Uuid;

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
    resource::handle::Handle,
    scene::{
        context::{utils::make_cube_scene, SceneContext},
        resources::SceneResources,
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        panel::PanelInstanceData,
        ui_box::{
            tree::{UIBoxTree, UIBoxTreeRenderCallback},
            UIBox,
        },
        window::{Window, WindowList, WindowOptions},
    },
};

use editor::panel::{
    build_floating_window_panel_tree, EditorPanelRenderCallbacks, PanelInstance,
    EDITOR_PANEL_ARENAS,
};

pub mod editor;

thread_local! {
    pub static EDITOR_SCENE_CONTEXT: SceneContext = Default::default();
}

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

    let mut scene_resources_rc: Option<Rc<RefCell<SceneResources>>> = None;

    EDITOR_SCENE_CONTEXT.with(|sc| {
        let cube_scene_context =
            make_cube_scene(framebuffer_rc.borrow().width_over_height).unwrap();

        let resources = RefCell::into_inner(Rc::try_unwrap(cube_scene_context.resources).unwrap());
        let scenes = RefCell::into_inner(cube_scene_context.scenes);

        *sc.resources.borrow_mut() = resources;
        *sc.scenes.borrow_mut() = scenes;

        scene_resources_rc.replace(sc.resources.clone());
    });

    // Panel rendering callbacks.

    let render_main_window_header: UIBoxTreeRenderCallback =
        Rc::new(|tree: &mut UIBoxTree| -> Result<(), String> {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                editor::ui::build_main_menu_bar_ui(ctx, tree)?;
                editor::ui::build_toolbar_ui(ctx, tree)
            })?;

            Ok(())
        });

    let panel_metadata_map = EditorPanelRenderCallbacks {
        outline: Rc::new(
            |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                EDITOR_PANEL_ARENAS.with(|arenas| {
                    let mut outline_arena = arenas.outline.borrow_mut();

                    if let Ok(entry) = outline_arena.get_mut(panel_instance) {
                        let panel = &mut entry.item;

                        panel.render(tree).unwrap();
                    }
                });

                Ok(())
            },
        ),
        viewport_3d: (
            Rc::new(
                |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                    EDITOR_PANEL_ARENAS.with(|arenas| {
                        let mut viewport_3d_arena = arenas.viewport_3d.borrow_mut();

                        if let Ok(entry) = viewport_3d_arena.get_mut(panel_instance) {
                            let panel = &mut entry.item;

                            panel.render(tree).unwrap();
                        }
                    });

                    Ok(())
                },
            ),
            Rc::new(
                |panel_instance: &Handle,
                 extent: &ScreenExtent,
                 target: &mut Buffer2D|
                 -> Result<(), String> {
                    EDITOR_PANEL_ARENAS.with(|arenas| {
                        let mut viewport_3d_arena = arenas.viewport_3d.borrow_mut();

                        if let Ok(entry) = viewport_3d_arena.get_mut(panel_instance) {
                            let panel = &mut entry.item;

                            panel.custom_render_callback(extent, target).unwrap();
                        }
                    });

                    Ok(())
                },
            ),
        ),
        asset_browser: Rc::new(
            |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                EDITOR_PANEL_ARENAS.with(|arenas| {
                    let mut asset_browser_arena = arenas.asset_browser.borrow_mut();

                    if let Ok(entry) = asset_browser_arena.get_mut(panel_instance) {
                        let panel = &mut entry.item;

                        panel.render(tree).unwrap();
                    }
                });

                Ok(())
            },
        ),
        console: Rc::new(
            |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                EDITOR_PANEL_ARENAS.with(|arenas| {
                    let mut console_arena = arenas.console.borrow_mut();

                    if let Ok(entry) = console_arena.get_mut(panel_instance) {
                        let panel = &mut entry.item;

                        panel.render(tree).unwrap();
                    }
                });

                Ok(())
            },
        ),
        inspector: Rc::new(
            |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                EDITOR_PANEL_ARENAS.with(|arenas| {
                    let mut inspector_arena = arenas.inspector.borrow_mut();

                    if let Ok(entry) = inspector_arena.get_mut(panel_instance) {
                        let panel = &mut entry.item;

                        panel.render(tree).unwrap();
                    }
                });

                Ok(())
            },
        ),
        file_system: Rc::new(
            |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
                EDITOR_PANEL_ARENAS.with(|arenas| {
                    let mut file_system_arena = arenas.file_system.borrow_mut();

                    if let Ok(entry) = file_system_arena.get_mut(panel_instance) {
                        let panel = &mut entry.item;

                        panel.render(tree).unwrap();
                    }
                });

                Ok(())
            },
        ),
    };

    // Renderer

    let renderer = SoftwareRenderer::new(
        Default::default(),
        scene_resources_rc.unwrap(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let renderer_rc: Rc<RefCell<SoftwareRenderer>> = Rc::new(RefCell::new(renderer));

    // Initial main window.

    let window_list = {
        let mut list: WindowList = Default::default();

        let main_window_id = "main_window".to_string();

        EDITOR_SCENE_CONTEXT.with(|sc| {
            let resource_arenas = sc.resources.borrow();

            EDITOR_PANEL_ARENAS.with(|panel_arenas| {
                let main_window_panel_tree = editor::panel::build_main_window_panel_tree(
                    &main_window_id,
                    &resource_arenas,
                    panel_arenas,
                    &panel_metadata_map,
                    &renderer_rc,
                )
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
            });
        });

        for i in 0..2 {
            EDITOR_PANEL_ARENAS.with(|arenas| {
                let mut outline_arena = arenas.outline.borrow_mut();
                let mut console_arena = arenas.console.borrow_mut();
                let mut inspector_arena = arenas.inspector.borrow_mut();

                let (panel_id, panel_instance, render_callback) = if i == 0 {
                    (
                        format!("Outline {}", i),
                        outline_arena.insert(Uuid::new_v4(), Default::default()),
                        panel_metadata_map.outline.clone(),
                    )
                } else if i == 1 {
                    (
                        format!("Console {}", i),
                        console_arena.insert(Uuid::new_v4(), Default::default()),
                        panel_metadata_map.console.clone(),
                    )
                } else {
                    (
                        format!("Inspector {}", i),
                        inspector_arena.insert(Uuid::new_v4(), Default::default()),
                        panel_metadata_map.inspector.clone(),
                    )
                };

                let floating_window_panel_tree = build_floating_window_panel_tree(
                    &panel_id,
                    PanelInstanceData {
                        panel_instance,
                        render: Some(render_callback),
                        custom_render_callback: None,
                    },
                )
                .unwrap();

                let floating_window = Window::new(
                    panel_id,
                    WindowOptions {
                        docked: false,
                        with_titlebar: true,
                        size: (300, 225),
                        position: (100 + i * 100, 100 + i * 100),
                    },
                    None,
                    floating_window_panel_tree,
                );

                list.push_back(floating_window);
            });
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

fn render_window_list(window_list: &mut WindowList, resolution: &Resolution) {
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
