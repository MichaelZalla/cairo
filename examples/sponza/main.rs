extern crate sdl2;

use std::cell::RefCell;

use sdl2::keyboard::Keycode;

use cairo::{
    app::{resolution::RESOLUTION_960_BY_540, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    debug::message::DebugMessageBuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    matrix::Mat4,
    pipeline::{zbuffer::DepthTestMethod, Pipeline},
    scene::node::{
        SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
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
    vec::{vec3::Vec3, vec4::Vec4},
};

pub mod scene;

use scene::make_sponza_scene;

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/sponza".to_string(),
        window_resolution: RESOLUTION_960_BY_540,
        canvas_resolution: RESOLUTION_960_BY_540,
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = Box::leak(Box::new(FontInfo {
        filepath: "C:/Windows/Fonts/vgasys.fon".to_string(),
        point_size: 16,
    }));

    let font_cache_rc = Box::leak(Box::new(RefCell::new(FontCache::new(
        app.context.ttf_context,
    ))));

    font_cache_rc.borrow_mut().load(font_info)?;

    // Debug messages

    let debug_message_buffer_rc: RefCell<DebugMessageBuffer> = Default::default();

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Scene context

    let scene_context = make_sponza_scene(rendering_context, &framebuffer_rc.borrow())?;

    let scene_context_rc = RefCell::new(scene_context);

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Fragment shaders

    let fragment_shaders = [
        DEFAULT_FRAGMENT_SHADER,
        AlbedoFragmentShader,
        DepthFragmentShader,
        NormalFragmentShader,
        UvTestFragmentShader,
    ];

    let active_fragment_shader_index_rc: RefCell<usize> = Default::default();

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context_rc.borrow().resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.bind_framebuffer(Some(&framebuffer_rc));

    pipeline.geometry_shader_options.base_color_mapping_active = false;

    pipeline
        .geometry_shader_options
        .specular_exponent_mapping_active = true;

    pipeline.geometry_shader_options.normal_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            app.window_info.canvas_resolution.width, app.window_info.canvas_resolution.height
        ));

        let uptime = app.timing_info.uptime_seconds;

        debug_message_buffer.write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        debug_message_buffer.write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        let scene_context = scene_context_rc.borrow_mut();
        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = shader_context_rc.borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        let mut update_scene_graph_node = |_current_depth: usize,
                                           _current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
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

                                    let framebuffer = framebuffer_rc.borrow_mut();

                                    if let Some(lock) = framebuffer.attachments.depth.as_ref() {
                                        let mut depth_buffer = lock.borrow_mut();

                                        depth_buffer
                                            .set_projection_z_near(camera.get_projection_z_near());
                                        depth_buffer
                                            .set_projection_z_far(camera.get_projection_z_far());
                                    }
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
                        &resources,
                        app,
                        mouse_state,
                        keyboard_state,
                        game_controller_state,
                        &mut shader_context,
                    )
                }
                SceneNodeType::DirectionalLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.directional_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                light.direction = Vec4::new(
                                    Vec3 {
                                        x: uptime.sin(),
                                        y: -1.0,
                                        z: uptime.cos(),
                                    },
                                    1.0,
                                )
                                .as_normal();

                                shader_context.set_directional_light(Some(*handle));

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get DirectionalLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `DirectionalLight` node with no resource handle!")
                    }
                },
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.point_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                light.position = SPONZA_CENTER
                                    + Vec3 {
                                        x: 1000.0 * uptime.sin(),
                                        y: 300.0,
                                        z: 0.0,
                                    };

                                shader_context.get_point_lights_mut().push(*handle);

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

                                light.look_vector.set_position(
                                    SPONZA_CENTER
                                        + Vec3 {
                                            x: -1000.0 * uptime.sin(),
                                            y: 500.0,
                                            z: 0.0,
                                        },
                                );

                                shader_context.get_spot_lights_mut().push(*handle);

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

        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        debug_message_buffer.write(format!(
            "Wireframe: {}",
            if pipeline.options.do_wireframe {
                "On"
            } else {
                "Off"
            }
        ));

        debug_message_buffer.write(format!(
            "Rasterized geometry: {}",
            if pipeline.options.do_rasterized_geometry {
                "On"
            } else {
                "Off"
            }
        ));

        if pipeline.options.do_rasterized_geometry {
            debug_message_buffer.write(format!(
                "Culling reject mask: {:?}",
                pipeline.options.face_culling_strategy.reject
            ));

            debug_message_buffer.write(format!(
                "Culling window order: {:?}",
                pipeline.options.face_culling_strategy.winding_order
            ));

            {
                let framebuffer = framebuffer_rc.borrow();

                let depth_buffer = framebuffer.attachments.depth.as_ref().unwrap().borrow();

                debug_message_buffer.write(format!(
                    "Depth test method: {:?}",
                    depth_buffer.get_depth_test_method()
                ));
            }

            debug_message_buffer.write(format!(
                "Lighting: {}",
                if pipeline.options.do_lighting {
                    "On"
                } else {
                    "Off"
                }
            ));

            pipeline.geometry_shader_options.update(
                keyboard_state,
                mouse_state,
                game_controller_state,
            );

            //

            let mut active_fragment_shader_index = active_fragment_shader_index_rc.borrow_mut();

            for keycode in &keyboard_state.keys_pressed {
                match keycode {
                    Keycode::I { .. } => {
                        let framebuffer = framebuffer_rc.borrow_mut();

                        let mut depth_buffer =
                            framebuffer.attachments.depth.as_ref().unwrap().borrow_mut();

                        let methods = [
                            DepthTestMethod::Always,
                            DepthTestMethod::Never,
                            DepthTestMethod::Less,
                            DepthTestMethod::Equal,
                            DepthTestMethod::LessThanOrEqual,
                            DepthTestMethod::Greater,
                            DepthTestMethod::NotEqual,
                            DepthTestMethod::GreaterThanOrEqual,
                        ];

                        let mut index = methods
                            .iter()
                            .position(|&method| method == *(depth_buffer.get_depth_test_method()))
                            .unwrap();

                        index = if index == (methods.len() - 1) {
                            0
                        } else {
                            index + 1
                        };

                        depth_buffer.set_depth_test_method(methods[index])
                    }
                    Keycode::H { .. } => {
                        *active_fragment_shader_index += 1;

                        if *active_fragment_shader_index == fragment_shaders.len() {
                            *active_fragment_shader_index = 0;
                        }

                        pipeline
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

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        let mut pipeline = pipeline_rc.borrow_mut();

        match scene.render(&resources, &mut pipeline, None) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let mut color_buffer = color_buffer_lock.borrow_mut();

                        let debug_messages = &mut *debug_message_buffer_rc.borrow_mut();

                        {
                            Graphics::render_debug_messages(
                                &mut color_buffer,
                                font_cache_rc,
                                font_info,
                                (12, 12),
                                1.0,
                                debug_messages,
                            );
                        }

                        Ok(color_buffer.get_all().clone())
                    }
                    None => panic!(),
                }
            }
            Err(e) => panic!("{}", e),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
