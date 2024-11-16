extern crate sdl2;

use std::{cell::RefCell, env, f32::consts::PI, path::Path, rc::Rc};

use sdl2::{keyboard::Keycode, mouse::Cursor};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTIONS_16X9, RESOLUTION_1600_BY_900},
        window::AppWindowingMode,
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{self, cursor::MouseCursorKind, MouseState},
    },
    effect::Effect,
    effects::{
        dilation_effect::DilationEffect, grayscale_effect::GrayscaleEffect,
        invert_effect::InvertEffect, kernel_effect::KernelEffect,
    },
    matrix::Mat4,
    render::options::RenderPassFlag,
    resource::handle::Handle,
    scene::{
        context::SceneContext,
        graph::SceneGraph,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        debug_shaders::{
            albedo_fragment_shader::AlbedoFragmentShader,
            depth_fragment_shader::DepthFragmentShader,
            normal_fragment_shader::NormalFragmentShader,
            uv_test_fragment_shader::UvTestFragmentShader,
        },
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::map::{TextureMap, TextureMapStorageFormat},
    transform::quaternion::Quaternion,
    ui::{context::GLOBAL_UI_CONTEXT, ui_box::tree::UIBoxTree, window::list::WindowList},
    vec::vec3,
};

use command::{process_commands, CommandBuffer};
use panels::{PanelArenas, PanelInstance, PanelRenderCallbacks};
use scene::make_scene;
use settings::Settings;
use window::make_window_list;

mod command;
mod panels;
mod scene;
mod settings;
mod window;

thread_local! {
    pub static SETTINGS: RefCell<Settings> = Default::default();
    pub static SCENE_CONTEXT: SceneContext = Default::default();
    pub static COMMAND_BUFFER: CommandBuffer = Default::default();
}

static DEFAULT_WINDOW_RESOLUTION: Resolution = RESOLUTION_1600_BY_900;

fn retain_cursor(cursor_kind: &MouseCursorKind, retained: &mut Option<Cursor>) {
    let cursor = mouse::cursor::set_cursor(cursor_kind).unwrap();

    retained.replace(cursor);
}

fn resize_framebuffer(
    resolution: Resolution,
    framebuffer_rc: &Rc<RefCell<Framebuffer>>,
    renderer: &mut SoftwareRenderer,
    window_list: &mut WindowList,
) {
    {
        // Resize our framebuffer to match the native window's new resolution.

        let mut framebuffer = framebuffer_rc.borrow_mut();

        framebuffer.resize(resolution.width, resolution.height, true);

        SCENE_CONTEXT.with(|ctx| {
            let resources = &ctx.resources;

            let mut camera_arena = resources.camera.borrow_mut();

            for entry in camera_arena.entries.iter_mut().flatten() {
                let camera = &mut entry.item;

                if camera
                    .set_aspect_ratio(framebuffer.width_over_height)
                    .is_ok()
                {
                    let mut shader_context = renderer.shader_context.borrow_mut();

                    camera.update_shader_context(&mut shader_context);
                }
            }
        })
    }

    // Re-binds the (resized) framebuffer.

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

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

    // Allocates a default framebuffer.

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Load a test texture.

    let uv_test_gradient_texture_handle = SCENE_CONTEXT.with(|ctx| -> Result<Handle, String> {
        let resources = &ctx.resources;

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let mut image_arena = ctx.image_arena.borrow_mut();

            image_arena.replace(resources.texture_u8.clone());
        });

        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        let texture: TextureMap<u8> = TextureMap::new(
            "./assets/textures/uv-test-gradient.png",
            TextureMapStorageFormat::RGB24,
        );

        Ok(texture_u8_arena.insert(texture))
    })?;

    // Makes a 3D scene and a preconfigured shader context.

    let (scene, shader_context) = SCENE_CONTEXT.with(
        |scene_context| -> Result<(SceneGraph, ShaderContext), String> {
            let resources = &scene_context.resources;

            let mut camera_arena = resources.camera.borrow_mut();
            let mut environment_arena = resources.environment.borrow_mut();
            let mut ambient_light_arena = resources.ambient_light.borrow_mut();
            let mut directional_light_arena = resources.directional_light.borrow_mut();
            let mut mesh_arena = resources.mesh.borrow_mut();
            let mut material_arena = resources.material.borrow_mut();
            let mut entity_arena = resources.entity.borrow_mut();
            let mut point_light_arena = resources.point_light.borrow_mut();
            let mut spot_light_arena = resources.spot_light.borrow_mut();
            let mut skybox_arena = resources.skybox.borrow_mut();

            make_scene(
                &mut camera_arena,
                camera_aspect_ratio,
                &mut environment_arena,
                &mut ambient_light_arena,
                &mut directional_light_arena,
                &mut mesh_arena,
                &mut material_arena,
                &mut entity_arena,
                &mut point_light_arena,
                &mut spot_light_arena,
                &mut skybox_arena,
            )
        },
    )?;

    SCENE_CONTEXT.with(|scene_context| -> Result<(), String> {
        if let Some(skybox_handle) = scene
            .root
            .find(|node| *node.get_type() == SceneNodeType::Skybox)?
        {
            let resources = &scene_context.resources;

            let mut skybox_arena = resources.skybox.borrow_mut();

            if let Ok(entry) = skybox_arena.get_mut(&skybox_handle) {
                let skybox = &mut entry.item;

                let mut texture_vec2_arena = resources.texture_vec2.borrow_mut();
                let mut cubemap_vec3_arena = resources.cubemap_vec3.borrow_mut();

                let hdr_path = Path::new("./examples/ibl/assets/thatch_chapel_4k.hdr");

                skybox.load_hdr(&mut texture_vec2_arena, &mut cubemap_vec3_arena, hdr_path);
            }
        }

        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);

        Ok(())
    })?;

    // Initializes a shader context.

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Initializes a software renderer (pipeline).

    let mut renderer = {
        let scene_resources = SCENE_CONTEXT.with(|ctx| ctx.resources.clone());

        SoftwareRenderer::new(
            shader_context_rc.clone(),
            scene_resources,
            DEFAULT_VERTEX_SHADER,
            DEFAULT_FRAGMENT_SHADER,
            Default::default(),
        )
    };

    SETTINGS.with(|settings_rc| {
        let mut settings = settings_rc.borrow_mut();

        settings.resolution = RESOLUTIONS_16X9
            .iter()
            .position(|r| *r == window_info.window_resolution)
            .unwrap();

        settings.render_options = renderer.options;
        settings.shader_options = renderer.shader_options;
    });

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Builds a list of windows containing our UI.

    let panel_arenas: PanelArenas = Default::default();

    let panel_render_callbacks = PanelRenderCallbacks {
        settings: Rc::new(panel_render_callback!(panel_arenas, settings)),
        render_options: Rc::new(panel_render_callback!(panel_arenas, render_options)),
        shader_options: Rc::new(panel_render_callback!(panel_arenas, shader_options)),
        rasterization_options: Rc::new(panel_render_callback!(panel_arenas, rasterization_options)),
        camera_attributes: Rc::new(panel_render_callback!(panel_arenas, camera_attributes)),
        scene_graph: Rc::new(panel_render_callback!(panel_arenas, scene_graph)),
    };

    let window_list = {
        SCENE_CONTEXT.with(|ctx| -> Result<WindowList, String> {
            make_window_list(
                ctx,
                &panel_arenas,
                panel_render_callbacks,
                &uv_test_gradient_texture_handle,
            )
        })
    }?;

    let window_list_rc = Rc::new(RefCell::new(window_list));

    // We'll need to remember the index of the last rendered frame.

    // @TODO Why not have the `App` track this?!
    let current_frame_index_rc = RefCell::new(0_u32);

    // We need to retain a reference to each `Cursor` that we set through SDL.
    // Without this reference, the SDL cursor is immediately dropped
    // (deallocated), and we won't see our custom cursor take effect.

    let retained_cursor_rc: RefCell<Option<Cursor>> = Default::default();

    // Create several screen-space post-processing effects.

    #[allow(unused)]
    let outline_effect = DilationEffect::new(color::BLUE, color::BLACK, Some(2));
    let outline_effect_rc = RefCell::new(outline_effect);

    #[allow(unused)]
    let grayscale_effect = GrayscaleEffect {};
    let grayscale_effect_rc = RefCell::new(grayscale_effect);

    #[allow(unused)]
    let invert_effect = InvertEffect {};
    let invert_effect_rc = RefCell::new(invert_effect);

    #[allow(unused)]
    let sharpen_kernel_effect = KernelEffect::new([2, 2, 2, 2, -15, 2, 2, 2, 2], None);
    let sharpen_kernel_effect_rc = RefCell::new(sharpen_kernel_effect);

    #[allow(unused)]
    let blur_kernel_effect = KernelEffect::new([1, 2, 1, 2, 4, 2, 1, 2, 1], Some(5));
    let blur_kernel_effect_rc = RefCell::new(blur_kernel_effect);

    #[allow(unused)]
    let edge_detection_kernel_effect = KernelEffect::new([1, 1, 1, 1, -8, 1, 1, 1, 1], None);
    let edge_detection_kernel_effect_rc = RefCell::new(edge_detection_kernel_effect);

    // Primary function for rendering the UI tree to `framebuffer`; this
    // function is called when either (1) the main loop executes, or (2) the
    // user is actively resizing the main (native) application window.

    let render_to_window_canvas = |frame_index: Option<u32>,
                                   new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
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

        let mut window_list = window_list_rc.borrow_mut();

        // Check if our application window was just resized...

        if let Some(resolution) = new_resolution {
            let mut renderer = renderer_rc.borrow_mut();

            resize_framebuffer(resolution, &framebuffer_rc, &mut renderer, &mut window_list);
        } else {
            // Clear the framebuffer before rendering this frame.
            let mut framebuffer = framebuffer_rc.borrow_mut();

            framebuffer.clear();
        }

        {
            // Render scene.

            SCENE_CONTEXT.with(|scene_context| -> Result<(), String> {
                let resources = &scene_context.resources;

                let scenes = scene_context.scenes.borrow();

                let scene = &scenes[0];

                scene.render(resources, &renderer_rc, None)
            })?;
        }

        {
            let framebuffer = framebuffer_rc.borrow_mut();

            if let Some(color_buffer_rc) = &framebuffer.attachments.color {
                let mut color_buffer = color_buffer_rc.borrow_mut();

                SETTINGS.with(|settings_rc| {
                    let current_settings = settings_rc.borrow();

                    if current_settings.effects.outline {
                        let mut effect = outline_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }

                    if current_settings.effects.invert {
                        let mut effect = invert_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }

                    if current_settings.effects.grayscale {
                        let mut effect = grayscale_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }

                    if current_settings.effects.sharpen_kernel {
                        let mut effect = sharpen_kernel_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }

                    if current_settings.effects.blur_kernel {
                        let mut effect = blur_kernel_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }

                    if current_settings.effects.edge_detection_kernel {
                        let mut effect = edge_detection_kernel_effect_rc.borrow_mut();

                        effect.apply(&mut color_buffer);
                    }
                });
            }
        }

        //

        let mut framebuffer = framebuffer_rc.borrow_mut();
        let mut color_buffer = framebuffer.attachments.color.as_mut().unwrap().borrow_mut();

        GLOBAL_UI_CONTEXT.with(|ctx| {
            window_list.render(frame_index, &mut color_buffer).unwrap();

            {
                let cursor_kind = ctx.cursor_kind.borrow();

                let mut retained_cursor = retained_cursor_rc.borrow_mut();

                retain_cursor(&cursor_kind, &mut retained_cursor);
            }
        });

        color_buffer.copy_to(canvas);

        Ok(())
    };

    // Instantiate our app, using the rendering callback we defined above.

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // Load the font indicated by the CLI argument(s).

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.load_font(&app, args[1].to_string(), 12);
    });

    // Use the rendering context to load any images in our texture arena.

    SCENE_CONTEXT.with(|ctx| -> Result<(), String> {
        let resources = &ctx.resources;

        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        for texture in texture_u8_arena
            .entries
            .iter_mut()
            .flatten()
            .map(|entry| &mut entry.item)
        {
            texture.load(&app.context.rendering_context)?;

            if texture.generate_mipmaps().is_ok() {}
        }

        Ok(())
    })?;

    // Define `update()` in the context of our app's main loop.

    #[allow(clippy::too_many_arguments)]
    fn update_node(
        _current_world_transform: &Mat4,
        node: &mut SceneNode,
        resources: &SceneResources,
        app: &App,
        _mouse_state: &MouseState,
        _keyboard_state: &KeyboardState,
        _game_controller_state: &GameControllerState,
        shader_context: &mut ShaderContext,
    ) -> Result<bool, String> {
        let uptime = app.timing_info.uptime_seconds;

        let (node_type, handle) = (node.get_type(), node.get_handle());

        match node_type {
            SceneNodeType::Entity => {
                let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                node.get_transform_mut().set_rotation(q);

                Ok(true)
            }
            SceneNodeType::Camera => {
                let camera_arena = resources.camera.borrow();
                let camera_handle = handle.unwrap();

                if let Ok(entry) = camera_arena.get(&camera_handle) {
                    let camera = &entry.item;

                    camera.update_shader_context(shader_context);
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    let update_node_rc = Rc::new(update_node);

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        // Check if the app's native window has been resized.

        {
            let window_info = app.window_info.borrow();
            if window_info.window_resolution.width != framebuffer_rc.borrow().width
                || window_info.window_resolution.height != framebuffer_rc.borrow().height
            {
                // Resize our framebuffer to match the new window resolution.

                let mut renderer = renderer_rc.borrow_mut();
                let mut canvas = app.context.rendering_context.canvas.borrow_mut();
                let window = canvas.window_mut();
                let mut window_list = window_list_rc.borrow_mut();

                resize_framebuffer(
                    Resolution::new(window.size()),
                    &framebuffer_rc,
                    &mut renderer,
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

                let is_ctrl_pressed = keyboard_state.pressed_keycodes.contains(&Keycode::LCtrl)
                    || keyboard_state.pressed_keycodes.contains(&Keycode::RCtrl);

                let is_shift_pressed = keyboard_state.pressed_keycodes.contains(&Keycode::LShift)
                    || keyboard_state.pressed_keycodes.contains(&Keycode::RShift);

                keyboard_state
                    .newly_pressed_keycodes
                    .retain(|keycode| match *keycode {
                        Keycode::F7 => {
                            GLOBAL_UI_CONTEXT.with(|ctx| {
                                let mut debug_options = ctx.debug.borrow_mut();

                                debug_options.draw_box_boundaries =
                                    !debug_options.draw_box_boundaries;
                            });

                            false
                        }
                        Keycode::Z => {
                            if is_ctrl_pressed {
                                if let Some(executed_command) = executed_commands.pop_back() {
                                    let (new_pending_command, is_undo) = {
                                        if is_shift_pressed {
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
                            if is_ctrl_pressed {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let vsync = current_settings.vsync;

                                    let cmd_str = format!(
                                        "set vsync {}",
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
                            if is_ctrl_pressed {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let hdr = current_settings.hdr;

                                    let cmd_str =
                                        format!("set hdr {}", if hdr { "false" } else { "true " })
                                            .to_string();

                                    pending_commands.push_back((cmd_str, false));
                                });

                                false
                            } else {
                                true
                            }
                        }
                        Keycode::B => {
                            if is_ctrl_pressed {
                                SETTINGS.with(|settings_rc| {
                                    let current_settings = settings_rc.borrow();

                                    let bloom = current_settings
                                        .render_options
                                        .render_pass_flags
                                        .contains(RenderPassFlag::Bloom);

                                    let cmd_str = format!(
                                        "set render_options.do_bloom {}",
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

            let mut renderer = renderer_rc.borrow_mut();
            let mut window_list = window_list_rc.borrow_mut();

            if let Some(resolution) = new_resolution {
                resize_framebuffer(resolution, &framebuffer_rc, &mut renderer, &mut window_list);

                app.resize_window(resolution)
            } else {
                if let Some(mode) = new_windowing_mode {
                    app.set_windowing_mode(mode)?;

                    let mut canvas = app.context.rendering_context.canvas.borrow_mut();
                    let window = canvas.window_mut();

                    resize_framebuffer(
                        Resolution::new(window.size()),
                        &framebuffer_rc,
                        &mut renderer,
                        &mut window_list,
                    );
                }

                Ok(())
            }
        })?;

        // Update our scene graph, shader context, and rendering and shading
        // options.

        SCENE_CONTEXT.with(|ctx| -> Result<(), String> {
            let resources = &ctx.resources;

            let mut shader_context = (*shader_context_rc).borrow_mut();

            let mut scenes = ctx.scenes.borrow_mut();
            let scene = &mut scenes[0];

            // Traverse the scene graph and update its nodes.

            scene.update(
                resources,
                &mut shader_context,
                app,
                mouse_state,
                keyboard_state,
                game_controller_state,
                Some(update_node_rc.clone()),
            )?;

            let mut renderer = renderer_rc.borrow_mut();

            SETTINGS.with(|settings_rc| {
                let current_settings = settings_rc.borrow();

                renderer.options = current_settings.render_options;
                renderer.shader_options = current_settings.shader_options;

                let shader = [
                    DEFAULT_FRAGMENT_SHADER,
                    AlbedoFragmentShader,
                    DepthFragmentShader,
                    NormalFragmentShader,
                    UvTestFragmentShader,
                ][current_settings.fragment_shader];

                renderer.set_fragment_shader(shader);

                let framebuffer = framebuffer_rc.borrow_mut();

                if let Some(depth_buffer_rc) = &framebuffer.attachments.depth {
                    let mut depth_buffer = depth_buffer_rc.borrow_mut();

                    depth_buffer.set_depth_test_method(current_settings.depth_test_method);
                }
            });

            // renderer.options.update(keyboard_state);

            renderer.shader_options.update(keyboard_state);

            Ok(())
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

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}
