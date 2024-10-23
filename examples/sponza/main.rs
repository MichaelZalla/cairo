extern crate sdl2;

use std::{cell::RefCell, env, f32::consts::PI, rc::Rc};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_640_BY_480},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    debug::message::DebugMessageBuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    matrix::Mat4,
    render::options::RenderPassFlag,
    scene::{
        context::SceneContext,
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
    software_renderer::{zbuffer::DEPTH_TEST_METHODS, SoftwareRenderer},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

pub mod scene;

use scene::make_sponza_scene;

fn main() -> Result<(), String> {
    // Validates command line arguments.

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example sponza /path/to/your-font.fon");
        return Ok(());
    }

    let mut window_info = AppWindowInfo {
        title: "examples/sponza".to_string(),
        window_resolution: RESOLUTION_640_BY_480 * 2.0,
        canvas_resolution: RESOLUTION_640_BY_480,
        relative_mouse_mode: true,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = Box::leak(Box::new(FontInfo {
        filepath: args[1].to_string(),
        point_size: 14,
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

    framebuffer.complete(0.3, 10_000.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();
        let mut cubemap_u8_arena = resources.cubemap_u8.borrow_mut();
        let mut skybox_arena = resources.skybox.borrow_mut();

        make_sponza_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut texture_u8_arena,
            rendering_context,
            &mut point_light_arena,
            &mut spot_light_arena,
            &mut cubemap_u8_arena,
            &mut skybox_arena,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Fragment shaders

    let fragment_shaders = [
        DEFAULT_FRAGMENT_SHADER,
        AlbedoFragmentShader,
        DepthFragmentShader,
        NormalFragmentShader,
        UvTestFragmentShader,
    ];

    let active_fragment_shader_index_rc: RefCell<usize> = Default::default();

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.albedo_mapping_active = true;
    renderer.shader_options.specular_exponent_mapping_active = true;
    renderer.shader_options.normal_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    #[allow(clippy::too_many_arguments)]
    let update_scene_graph_node = |_current_world_transform: &Mat4,
                                   node: &mut SceneNode,
                                   resources: &SceneResources,
                                   app: &App,
                                   _mouse_state: &MouseState,
                                   _keyboard_state: &KeyboardState,
                                   _game_controller_state: &GameControllerState,
                                   _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let (node_type, handle) = (node.get_type(), node.get_handle());

        let uptime = app.timing_info.uptime_seconds;

        match node_type {
            SceneNodeType::DirectionalLight => match handle {
                Some(handle) => {
                    let mut arena = resources.directional_light.borrow_mut();

                    match arena.get_mut(handle) {
                        Ok(entry) => {
                            let light = &mut entry.item;

                            light.set_direction(Quaternion::new(
                                vec3::UP,
                                uptime.rem_euclid(PI * 2.0),
                            ));

                            Ok(false)
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
            SceneNodeType::PointLight => {
                let transform = node.get_transform_mut();

                let position = Vec3 {
                    x: 800.0 * (uptime / 20.0).sin(),
                    y: 200.0,
                    z: -75.0,
                };

                transform.set_translation(position);

                Ok(false)
            }
            SceneNodeType::SpotLight => {
                let transform = node.get_transform_mut();

                let position = Vec3 {
                    x: -800.0 * uptime.sin(),
                    y: 500.0,
                    z: 0.0,
                };

                transform.set_translation(position);

                Ok(false)
            }
            _ => Ok(false),
        }
    };

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let window_info = (*app.window_info).borrow();

        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            window_info.canvas_resolution.width, window_info.canvas_resolution.height
        ));

        let uptime = app.timing_info.uptime_seconds;

        debug_message_buffer.write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        debug_message_buffer.write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        let resources = &scene_context.resources;

        let mut shader_context = shader_context_rc.borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        let update_scene_graph_node_rc = Rc::new(update_scene_graph_node);

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_scene_graph_node_rc),
        )?;

        let mut renderer = renderer_rc.borrow_mut();

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
            if renderer
                .options
                .render_pass_flags
                .contains(RenderPassFlag::Rasterization)
            {
                "On"
            } else {
                "Off"
            }
        ));

        if renderer
            .options
            .render_pass_flags
            .contains(RenderPassFlag::Rasterization)
        {
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
                let framebuffer = framebuffer_rc.borrow();

                let depth_buffer = framebuffer.attachments.depth.as_ref().unwrap().borrow();

                debug_message_buffer.write(format!(
                    "Depth test method: {:?}",
                    depth_buffer.get_depth_test_method()
                ));
            }

            debug_message_buffer.write(format!(
                "Lighting: {}",
                if renderer
                    .options
                    .render_pass_flags
                    .contains(RenderPassFlag::Lighting)
                {
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

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        // Render scene.

        match scene.render(resources, &renderer_rc, None) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                let mut font_cache = font_cache_rc.borrow_mut();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let mut color_buffer = color_buffer_lock.borrow_mut();

                        let debug_messages = &mut *debug_message_buffer_rc.borrow_mut();

                        {
                            Graphics::render_debug_messages(
                                &mut color_buffer,
                                &mut font_cache,
                                font_info,
                                (12, 12),
                                1.0,
                                debug_messages,
                            );
                        }

                        color_buffer.copy_to(canvas);

                        Ok(())
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
