extern crate sdl2;

use std::{cell::RefCell, env, f32::consts::PI, rc::Rc};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{
        handle_window_resize_event,
        resolution::{Resolution, RESOLUTIONS_16X9},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    debug::message::DebugMessageBuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::{cache::FontCache, FontInfo},
    matrix::Mat4,
    scene::node::{
        SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
    },
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
    software_renderer::{zbuffer::DEPTH_TEST_METHODS, SoftwareRenderer},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

use scene::{make_primitives_scene, POINT_LIGHTS_COUNT};

mod scene;

fn main() -> Result<(), String> {
    // Validates command line arguments.

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example generate-primitives /path/to/your-font.fon");
        return Ok(());
    }

    let current_resolution_index_rc: RefCell<usize> = RefCell::new(2);

    let resolution = RESOLUTIONS_16X9[*current_resolution_index_rc.borrow()];

    let mut window_info = AppWindowInfo {
        title: "examples/generate-primitives".to_string(),
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: resolution,
        canvas_resolution: resolution,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = Box::leak(Box::new(FontInfo {
        filepath: args[1].to_string(),
        point_size: 14,
    }));

    // Debug messages

    let debug_message_buffer_rc: RefCell<DebugMessageBuffer> = Default::default();

    // Main application window's framebuffer.

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Current frame index.

    let current_frame_index_rc = RefCell::new(0_u32);

    // Scene context

    let (scene_context, shader_context) =
        make_primitives_scene(aspect_ratio, Some(rendering_context))?;

    let scene_context_rc = Rc::new(scene_context);

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context_rc.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Font cache

    let font_cache_rc = Box::leak(Box::new(RefCell::new(FontCache::new(
        app.context.ttf_context,
    ))));

    font_cache_rc.borrow_mut().load(font_info)?;

    // Fragment shaders

    let fragment_shaders = [
        DEFAULT_FRAGMENT_SHADER,
        AlbedoFragmentShader,
        DepthFragmentShader,
        NormalFragmentShader,
        UvTestFragmentShader,
    ];

    let active_fragment_shader_index_rc: RefCell<usize> = Default::default();

    let looking_at_point_light_rc = RefCell::new(false);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut renderer = renderer_rc.borrow_mut();

        let resources = scene_context_rc.resources.borrow_mut();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                (Keycode::R, _) => {
                    // Resize the app's rendering canvas.

                    let mut current_resolution_index = current_resolution_index_rc.borrow_mut();

                    *current_resolution_index =
                        (*current_resolution_index + 1) % RESOLUTIONS_16X9.len();

                    let mut canvas_window = app.context.rendering_context.canvas.borrow_mut();

                    let window_info = &mut (*app.window_info).borrow_mut();

                    let canvas_texture = &mut (*app.canvas_texture).borrow_mut();

                    let new_resolution = RESOLUTIONS_16X9[*current_resolution_index];

                    handle_window_resize_event(
                        &mut canvas_window,
                        window_info,
                        canvas_texture,
                        new_resolution,
                    )?;

                    // Resize the framebuffer to match.
                    let mut framebuffer = framebuffer_rc.borrow_mut();

                    framebuffer.resize(new_resolution.width, new_resolution.height, true);
                }
                (Keycode::H, _) => {
                    let mut active_fragment_shader_index =
                        active_fragment_shader_index_rc.borrow_mut();

                    *active_fragment_shader_index += 1;

                    if *active_fragment_shader_index == fragment_shaders.len() {
                        *active_fragment_shader_index = 0;
                    }

                    // let mut renderer = renderer_rc.borrow_mut();

                    renderer.set_fragment_shader(fragment_shaders[*active_fragment_shader_index]);
                }
                (Keycode::L, _) => {
                    let mut looking_at_point_light = looking_at_point_light_rc.borrow_mut();

                    *looking_at_point_light = !*looking_at_point_light;
                }
                _ => {}
            }
        }

        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        {
            let window_info = (*app.window_info).borrow();

            debug_message_buffer.write(format!("{:?}", &window_info.canvas_resolution));
        }

        let uptime = app.timing_info.uptime_seconds;

        debug_message_buffer.write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        debug_message_buffer.write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        static COLOR_CHANNEL_PHASE_SHIFT: f32 = 2.0 * PI / 3.0;

        let mut point_lights_visited: usize = 0;
        let mut spot_lights_visited: usize = 0;

        let mut update_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mut entity_arena = resources.entity.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                if let Ok(entry) = resources.mesh.borrow_mut().get(&entity.mesh) {
                                    let mesh = &entry.item;

                                    if let Some(object_name) = &mesh.object_name {
                                        if object_name == "plane" {
                                            return Ok(());
                                        }
                                    }
                                }

                                let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                                let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                                node.get_transform_mut().set_rotation(q);

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get Entity from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `Entity` node with no resource handle!")
                    }
                },
                SceneNodeType::Camera => {
                    match handle {
                        Some(handle) => {
                            let mut camera_arena = resources.camera.borrow_mut();

                            match camera_arena.get_mut(handle) {
                                Ok(entry) => {
                                    let camera = &mut entry.item;

                                    debug_message_buffer.write(format!(
                                        "Camera position: {}",
                                        camera.look_vector.get_position()
                                    ));
                                }
                                Err(err) => panic!(
                                    "Failed to get Camera from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `Camera` node with no resource handle!")
                        }
                    }

                    node.update(
                        &current_world_transform,
                        &resources,
                        app,
                        mouse_state,
                        keyboard_state,
                        game_controller_state,
                        &mut shader_context,
                    )
                }
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.point_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                static LIGHT_SPEED_FACTOR: f32 = 0.66;
                                static ORBIT_RADIUS: f32 = 12.0;
                                static MAX_POINT_LIGHT_INTENSITY: f32 = 25.0;

                                let light_phase_shift = (2.0 * PI / (POINT_LIGHTS_COUNT as f32))
                                    * point_lights_visited as f32;

                                light.intensities = Vec3 {
                                    x: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 0.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                    y: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 1.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                    z: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 2.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                } * MAX_POINT_LIGHT_INTENSITY;

                                let offset = point_lights_visited % 2 == 0;

                                light.position = Vec3 {
                                    x: ORBIT_RADIUS
                                        * ((uptime * LIGHT_SPEED_FACTOR) + light_phase_shift).sin()
                                        * if offset { 1.5 } else { 1.0 },
                                    y: 1.0,
                                    z: ORBIT_RADIUS
                                        * ((uptime * LIGHT_SPEED_FACTOR) + light_phase_shift).cos()
                                        * if offset { 1.5 } else { 1.0 },
                                };

                                shader_context.get_point_lights_mut().push(*handle);

                                point_lights_visited += 1;

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get PointLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no resource handle!")
                    }
                },
                SceneNodeType::SpotLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.spot_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                static MAX_SPOT_LIGHT_INTENSITY: f32 = 25.0;

                                light.intensities = Vec3 {
                                    x: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 0.0).cos() / 2.0 + 0.5,
                                    y: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 1.0).cos() / 2.0 + 0.5,
                                    z: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 2.0).cos() / 2.0 + 0.5,
                                } * MAX_SPOT_LIGHT_INTENSITY;

                                shader_context.get_spot_lights_mut().push(*handle);

                                spot_lights_visited += 1;

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get SpotLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `SpotLight` node with no resource handle!")
                    }
                },
                _ => node.update(
                    &current_world_transform,
                    &resources,
                    app,
                    mouse_state,
                    keyboard_state,
                    game_controller_state,
                    &mut shader_context,
                ),
            }
        };

        scenes[0].root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut update_scene_graph_node,
        )?;

        renderer.options.update(keyboard_state);

        debug_message_buffer.write(format!(
            "Wireframe: {}",
            if renderer.options.draw_wireframe {
                "On"
            } else {
                "Off"
            }
        ));

        debug_message_buffer.write(format!(
            "Rasterized geometry: {}",
            if renderer.options.do_rasterization {
                "On"
            } else {
                "Off"
            }
        ));

        if renderer.options.do_rasterization {
            debug_message_buffer.write(format!(
                "Culling reject mask: {:?}",
                renderer
                    .options
                    .rasterizer_options
                    .face_culling_strategy
                    .reject
            ));

            debug_message_buffer.write(format!(
                "Culling winding order: {:?}",
                renderer
                    .options
                    .rasterizer_options
                    .face_culling_strategy
                    .winding_order
            ));

            {
                let framebuffer = (*framebuffer_rc).borrow();

                let depth_buffer = framebuffer.attachments.depth.as_ref().unwrap().borrow();

                debug_message_buffer.write(format!(
                    "Depth test method: {:?}",
                    depth_buffer.get_depth_test_method()
                ));
            }

            debug_message_buffer.write(format!(
                "Lighting: {}",
                if renderer.options.do_lighting {
                    "On"
                } else {
                    "Off"
                }
            ));

            renderer.shader_options.update(keyboard_state);

            //

            let mut active_fragment_shader_index = active_fragment_shader_index_rc.borrow_mut();

            for keycode in &keyboard_state.keys_pressed {
                match keycode {
                    (Keycode::I, _) => {
                        let framebuffer = framebuffer_rc.borrow_mut();

                        let mut depth_buffer =
                            framebuffer.attachments.depth.as_ref().unwrap().borrow_mut();

                        let mut index = DEPTH_TEST_METHODS
                            .iter()
                            .position(|&method| method == *(depth_buffer.get_depth_test_method()))
                            .unwrap();

                        index = if index == (DEPTH_TEST_METHODS.len() - 1) {
                            0
                        } else {
                            index + 1
                        };

                        depth_buffer.set_depth_test_method(DEPTH_TEST_METHODS[index])
                    }
                    (Keycode::H, _) => {
                        *active_fragment_shader_index += 1;

                        if *active_fragment_shader_index == fragment_shaders.len() {
                            *active_fragment_shader_index = 0;
                        }

                        renderer
                            .set_fragment_shader(fragment_shaders[*active_fragment_shader_index]);
                    }
                    _ => {}
                }
            }

            debug_message_buffer.write(format!(
                "Fragment shader: {}",
                [
                    "DEFAULT_FRAGMENT_SHADER",
                    "AlbedoFragmentShader",
                    "DepthFragmentShader",
                    "NormalFragmentShader",
                    "UvTestFragmentShader",
                ][*active_fragment_shader_index]
            ));
        }

        debug_message_buffer.write(format!(
            "Visualize normals: {}",
            if renderer.options.draw_normals {
                "On"
            } else {
                "Off"
            }
        ));

        debug_message_buffer.write(format!(
            "Looking at point light: {}",
            looking_at_point_light_rc.borrow(),
        ));

        Ok(())
    };

    let render = |frame_index, new_resolution: Option<Resolution>| -> Result<Vec<u32>, String> {
        if let Some(index) = frame_index {
            let mut current_frame_index = current_frame_index_rc.borrow_mut();

            *current_frame_index = index;
        }

        {
            let mut framebuffer = framebuffer_rc.borrow_mut();

            // Check if our application window was just resized.

            if let Some(resolution) = new_resolution {
                // Resize our framebuffer to match the window's new resolution.

                framebuffer.resize(resolution.width, resolution.height, false);
            }

            framebuffer.clear();
        }

        // Render scene.

        let resources = scene_context_rc.resources.borrow();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
        let scene = &mut scenes[0];

        match scene.render(&resources, &renderer_rc, None) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow_mut();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let color_buffer = color_buffer_lock.borrow_mut();

                        // {
                        //     let debug_messages = &mut *debug_message_buffer_rc.borrow_mut();
                        //     let mut font_cache = font_cache_rc.borrow_mut();

                        //     Graphics::render_debug_messages(
                        //         &mut color_buffer,
                        //         &mut font_cache,
                        //         font_info,
                        //         (12, 12),
                        //         1.0,
                        //         debug_messages,
                        //     );
                        // }

                        Ok(color_buffer.get_all().clone())
                    }
                    None => panic!(),
                }
            }
            Err(e) => panic!("{}", e),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}
