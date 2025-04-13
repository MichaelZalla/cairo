extern crate sdl2;

use std::{cell::RefCell, env, rc::Rc};

use current_platform::CURRENT_PLATFORM;

use sdl2::mouse::Cursor;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1920_BY_1080},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, MouseState},
    },
    font::cache::FontCache,
    resource::handle::Handle,
    scene::{
        context::{utils::make_cube_scene, SceneContext},
        graph::SceneGraph,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        panel::PanelInstanceData,
        ui_box::tree::{UIBoxTree, UIBoxTreeRenderCallback},
        window::{list::WindowList, Window, WindowOptions},
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

fn viewport_3d_panel_custom_render_callback(
    panel_instance: &Option<Handle>,
    extent: &ScreenExtent,
    target: &mut Buffer2D,
) -> Result<(), String> {
    EDITOR_PANEL_ARENAS.with(|arenas| {
        let mut viewport_3d_arena = arenas.viewport_3d.borrow_mut();

        if let Ok(entry) = viewport_3d_arena.get_mut(&panel_instance.unwrap()) {
            let panel = &mut entry.item;

            panel.custom_render_callback(extent, target).unwrap();
        }
    });

    Ok(())
}

fn main() -> Result<(), String> {
    let title = format!(
        "Cairo Engine - {} (build {})",
        CURRENT_PLATFORM,
        env!("GIT_COMMIT_SHORT_HASH")
    )
    .to_string();

    // Main window info.

    let mut window_info = AppWindowInfo {
        title,
        window_resolution: RESOLUTION_1920_BY_1080,
        canvas_resolution: RESOLUTION_1920_BY_1080,
        relative_mouse_mode: false,
        resizable: true,
        ..Default::default()
    };

    // Default render framebuffer.

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = RefCell::new(framebuffer);

    let current_frame_index_rc = RefCell::new(0_u32);

    let (scene, shader_context) = EDITOR_SCENE_CONTEXT.with(
        |scene_context| -> Result<(SceneGraph, ShaderContext), String> {
            let resources = &scene_context.resources;

            let mut camera_arena = resources.camera.borrow_mut();
            let mut environment_arena = resources.environment.borrow_mut();
            let mut ambient_light_arena = resources.ambient_light.borrow_mut();
            let mut directional_light_arena = resources.directional_light.borrow_mut();
            let mut mesh_arena = resources.mesh.borrow_mut();
            let mut material_arena = resources.material.borrow_mut();
            let mut entity_arena = resources.entity.borrow_mut();

            make_cube_scene(
                &mut camera_arena,
                camera_aspect_ratio,
                &mut environment_arena,
                &mut ambient_light_arena,
                &mut directional_light_arena,
                &mut mesh_arena,
                &mut material_arena,
                &mut entity_arena,
            )
        },
    )?;

    EDITOR_SCENE_CONTEXT.with(|scene_context| {
        scene_context.scenes.borrow_mut().push(scene);
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

    let editor_panel_render_callbacks = EditorPanelRenderCallbacks {
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
            viewport_3d_panel_custom_render_callback,
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

    let scene_resources_rc =
        EDITOR_SCENE_CONTEXT.with(|scene_context| scene_context.resources.clone());

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    let renderer = SoftwareRenderer::new(
        shader_context_rc,
        scene_resources_rc,
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
            let resources = &sc.resources;

            EDITOR_PANEL_ARENAS.with(|panel_arenas| {
                let main_window_panel_tree = editor::panel::build_main_window_panel_tree(
                    &main_window_id,
                    resources,
                    panel_arenas,
                    &editor_panel_render_callbacks,
                    &renderer_rc,
                )
                .unwrap();

                let main_window = Window::new(
                    main_window_id,
                    "Main window".to_string(),
                    WindowOptions::docked(window_info.window_resolution),
                    Some(render_main_window_header),
                    main_window_panel_tree,
                );

                list.0.push_back(main_window);
            });
        });

        for i in 0..2 {
            EDITOR_PANEL_ARENAS.with(|arenas| {
                let mut outline_arena = arenas.outline.borrow_mut();
                let mut console_arena = arenas.console.borrow_mut();
                let mut inspector_arena = arenas.inspector.borrow_mut();

                let (panel_id, panel_title, panel_instance, render_callback) = if i == 0 {
                    (
                        format!("Outline {}", i),
                        "Outline".to_string(),
                        outline_arena.insert(Default::default()),
                        editor_panel_render_callbacks.outline.clone(),
                    )
                } else if i == 1 {
                    (
                        format!("Console {}", i),
                        "Console".to_string(),
                        console_arena.insert(Default::default()),
                        editor_panel_render_callbacks.console.clone(),
                    )
                } else {
                    (
                        format!("Inspector {}", i),
                        "Inspector".to_string(),
                        inspector_arena.insert(Default::default()),
                        editor_panel_render_callbacks.inspector.clone(),
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
                    panel_title,
                    WindowOptions {
                        docked: false,
                        with_titlebar: true,
                        size: (300, 225),
                        position: (100 + i * 100, 100 + i * 100),
                    },
                    None,
                    floating_window_panel_tree,
                );

                list.0.push_back(floating_window);
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

    let render = |frame_index: Option<u32>,
                  new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let mut current_frame_index = current_frame_index_rc.borrow_mut();

        if let Some(index) = frame_index {
            *current_frame_index = index;

            // Prune old entries from our UI cache.

            GLOBAL_UI_CONTEXT.with(|ctx| {
                ctx.prune_cache(index);
            });
        }

        let mut framebuffer = framebuffer_rc.borrow_mut();

        let mut window_list = (*window_list_rc).borrow_mut();

        if let Some(resolution) = new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);

            // Rebuild the UI tree based on the new window (root) resolution.

            window_list.rebuild_ui_trees(resolution);
        }

        framebuffer.clear(None);

        // Check if our application window was just resized.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            window_list
                .render(*current_frame_index, &mut framebuffer)
                .unwrap();

            {
                let kind = &ctx.cursor_kind.borrow();

                *set_cursor_result_rc.borrow_mut() = Some(mouse::cursor::set_cursor(kind).unwrap());
            }
        });

        framebuffer.copy_to(canvas);

        Ok(())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render);

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
            font_info.point_size = 12;
        }
    });

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        GLOBAL_UI_CONTEXT.with(|ctx| {
            // Resets the cursor style.
            ctx.begin_frame();

            // Bind the latest user input events.
            ctx.set_user_inputs(keyboard_state, mouse_state, game_controller_state);

            // Binds the latest timing info.
            ctx.set_timing_info(&app.timing_info);
        });

        let resolution = (*app.window_info).borrow().window_resolution;

        let mut window_list = window_list_rc.borrow_mut();

        window_list.rebuild_ui_trees(resolution);

        Ok(())
    };

    app.run(&mut update, &render)?;

    Ok(())
}
