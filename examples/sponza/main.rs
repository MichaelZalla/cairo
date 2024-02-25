extern crate sdl2;

use std::cell::RefCell;

use sdl2::keyboard::Keycode;
use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    color,
    debug::message::DebugMessageBuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    matrix::Mat4,
    mesh,
    pipeline::{zbuffer::DepthTestMethod, Pipeline},
    resource::{arena::Arena, handle::Handle},
    scene::{
        camera::Camera,
        graph::SceneGraph,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        debug_shaders::{
            albedo_fragment_shader::AlbedoFragmentShader,
            depth_fragment_shader::DepthFragmentShader,
            normal_fragment_shader::NormalFragmentShader,
            specular_intensity_fragment_shader::SpecularIntensityFragmentShader,
            uv_test_fragment_shader::UvTestFragmentShader,
        },
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{cubemap::CubeMap, map::TextureMapStorageFormat},
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/sponza".to_string(),
        window_width: 860,
        window_height: 520,
        canvas_width: 860,
        canvas_height: 520,
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

    font_cache_rc.borrow_mut().load(&font_info)?;

    // Debug messages

    let debug_message_buffer_rc: RefCell<DebugMessageBuffer> = Default::default();

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Sponza meshes and materials

    let (atrium_meshes, mut atrium_materials_cache) =
        mesh::obj::load_obj("./examples/sponza/assets/sponza.obj");

    match &mut atrium_materials_cache {
        Some(cache) => {
            for material in cache.values_mut() {
                material.load_all_maps(rendering_context)?;
            }
        }
        None => (),
    }

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let mut point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    for i in 0..atrium_meshes.len() {
        entity_arena.insert(Uuid::new_v4(), Entity::new(&atrium_meshes[i]));
    }

    // Set up a camera for rendering our scene

    let camera_position = Vec3 {
        x: 1000.0,
        y: 300.0,
        z: 0.0,
    };

    let aspect_ratio = framebuffer_rc.borrow().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        camera_position,
        camera_position + vec3::LEFT,
        75.0,
        aspect_ratio,
    );

    camera.movement_speed = 300.0;

    camera.set_projection_z_far(10000.0);

    // Set up some lights for our scene.

    let ambient_light = AmbientLight {
        intensities: Vec3::ones() * 0.1,
    };

    let directional_light = DirectionalLight {
        intensities: Vec3::ones() * 0.1,
        direction: Vec4::new(vec3::UP * -1.0, 1.0).as_normal(),
    };

    let mut point_light = PointLight::new();

    point_light.intensities = color::BLUE.to_vec3() / 255.0 * 15.0;

    point_light.specular_intensity = 1.0;

    point_light.constant_attenuation = 1.0;
    point_light.linear_attenuation = 0.007;
    point_light.quadratic_attenuation = 0.0002;

    let mut spot_light = SpotLight::new();

    spot_light.intensities = color::RED.to_vec3() / 255.0 * 15.0;

    spot_light.constant_attenuation = 1.0;
    spot_light.linear_attenuation = 0.007;
    spot_light.quadratic_attenuation = 0.0002;

    // Skybox

    let mut skybox = CubeMap::cross(
        "examples/skybox/assets/cross/horizontal_cross.png",
        TextureMapStorageFormat::RGB24,
    );

    skybox.load(rendering_context).unwrap();

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Fragment shaders

    let fragment_shaders = vec![
        DEFAULT_FRAGMENT_SHADER,
        AlbedoFragmentShader,
        DepthFragmentShader,
        NormalFragmentShader,
        SpecularIntensityFragmentShader,
        UvTestFragmentShader,
    ];

    let active_fragment_shader_index_rc: RefCell<usize> = Default::default();

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.geometry_shader_options.diffuse_mapping_active = false;
    pipeline.geometry_shader_options.specular_mapping_active = true;
    pipeline.geometry_shader_options.normal_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // Create resource handles from our arenas.

    // let cube_entity_handle = entity_arena.insert(Uuid::new_v4(), cube_entity);
    let camera_handle = camera_arena.insert(Uuid::new_v4(), camera);
    let ambient_light_handle = ambient_light_arena.insert(Uuid::new_v4(), ambient_light);
    let directional_light_handle =
        directional_light_arena.insert(Uuid::new_v4(), directional_light);
    let point_light_handle = point_light_arena.insert(Uuid::new_v4(), point_light);
    let spot_light_handle = spot_light_arena.insert(Uuid::new_v4(), spot_light);

    let entity_arena_rc = RefCell::new(entity_arena);
    let camera_arena_rc = RefCell::new(camera_arena);
    let ambient_light_arena_rc = RefCell::new(ambient_light_arena);
    let directional_light_arena_rc = RefCell::new(directional_light_arena);
    let point_light_arena_rc = RefCell::new(point_light_arena);
    let spot_light_arena_rc = RefCell::new(spot_light_arena);

    // Create a scene graph.

    let mut scenegraph = SceneGraph::new();

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::Camera,
        Default::default(),
        Some(camera_handle),
        None,
    ));

    let ambient_light_node = SceneNode::new(
        SceneNodeType::AmbientLight,
        Default::default(),
        Some(ambient_light_handle),
        None,
    );

    scenegraph.root.add_child(ambient_light_node);

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::DirectionalLight,
        Default::default(),
        Some(directional_light_handle),
        None,
    ));

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::PointLight,
        Default::default(),
        Some(point_light_handle),
        None,
    ));

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::SpotLight,
        Default::default(),
        Some(spot_light_handle),
        None,
    ));

    for (index, entry) in entity_arena_rc.borrow().entries.iter().enumerate() {
        match entry {
            Some(entry) => {
                let handle = Handle {
                    index,
                    uuid: entry.uuid,
                };

                scenegraph.root.add_child(SceneNode::new(
                    SceneNodeType::Entity,
                    Default::default(),
                    Some(handle),
                    None,
                ));
            }
            None => (),
        }
    }

    // Prints the scenegraph to stdout.

    println!("{}", scenegraph);

    let scenegraph_rc = RefCell::new(scenegraph);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            app.window_info.canvas_width, app.window_info.canvas_height
        ));

        let uptime = app.timing_info.uptime_seconds;

        debug_message_buffer.write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        debug_message_buffer.write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        let mut context = shader_context_rc.borrow_mut();

        context.set_ambient_light(None);
        context.set_directional_light(None);
        context.get_point_lights_mut().clear();
        context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        let mut scenegraph = scenegraph_rc.borrow_mut();

        let mut point_lights_visited: usize = 0;
        let mut spot_lights_visited: usize = 0;

        let mut update_scene_graph_node = |_current_depth: usize,
                                           _current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Scene => Ok(()),
                SceneNodeType::Entity => Ok(()),
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        let mut camera_arena = camera_arena_rc.borrow_mut();

                        match camera_arena.get_mut(handle) {
                            Ok(entry) => {
                                let camera = &mut entry.item;

                                camera.update(
                                    &app.timing_info,
                                    keyboard_state,
                                    mouse_state,
                                    game_controller_state,
                                );

                                debug_message_buffer.write(format!(
                                    "Camera position: {}",
                                    camera.look_vector.get_position()
                                ));

                                let camera_view_inverse_transform =
                                    camera.get_view_inverse_transform();

                                context.set_view_position(Vec4::new(
                                    camera.look_vector.get_position(),
                                    1.0,
                                ));

                                context.set_view_inverse_transform(camera_view_inverse_transform);

                                context.set_projection(camera.get_projection());

                                let framebuffer = framebuffer_rc.borrow_mut();

                                match framebuffer.attachments.depth.as_ref() {
                                    Some(lock) => {
                                        let mut depth_buffer = lock.borrow_mut();

                                        depth_buffer
                                            .set_projection_z_near(camera.get_projection_z_near());
                                        depth_buffer
                                            .set_projection_z_far(camera.get_projection_z_far());
                                    }
                                    None => (),
                                }

                                Ok(())
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
                },
                SceneNodeType::AmbientLight => {
                    match handle {
                        Some(handle) => match ambient_light_arena_rc.borrow_mut().get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                context.set_ambient_light(Some(*light))
                            }
                            Err(err) => panic!(
                                "Failed to get AmbientLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        },
                        None => {
                            panic!("Encountered a `AmbientLight` node with no resource handle!")
                        }
                    }
                    Ok(())
                }
                SceneNodeType::DirectionalLight => match handle {
                    Some(handle) => {
                        let mut arena = directional_light_arena_rc.borrow_mut();

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

                                context.set_directional_light(Some(*light));

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
                        let mut arena = point_light_arena_rc.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                light.position = SPONZA_CENTER
                                    + Vec3 {
                                        x: 1000.0 * uptime.sin(),
                                        y: 300.0,
                                        z: 0.0,
                                    };

                                context.get_point_lights_mut().push(point_light.clone());

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
                        let mut arena = spot_light_arena_rc.borrow_mut();

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

                                context.get_spot_lights_mut().push(spot_light.clone());

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
            }
        };

        scenegraph.root.visit_mut(
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

                        let methods = vec![
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
                    "SpecularIntensityFragmentShader",
                    "UvTestFragmentShader",
                ][*active_fragment_shader_index]
            ));
        }

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline.bind_framebuffer(Some(&framebuffer_rc));

        // Begin frame

        pipeline.begin_frame();

        // Render entities.

        let scenegraph = scenegraph_rc.borrow_mut();

        let mut render_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Scene => Ok(()),
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mut entity_arena = entity_arena_rc.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                pipeline.render_entity(
                                    entity,
                                    &current_world_transform,
                                    atrium_materials_cache.as_ref(),
                                );

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
                SceneNodeType::Camera => Ok(()),
                SceneNodeType::AmbientLight => Ok(()),
                SceneNodeType::DirectionalLight => Ok(()),
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        let mut point_light_arena = point_light_arena_rc.borrow_mut();

                        match point_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let point_light = &mut entry.item;

                                pipeline.render_point_light(point_light, None, None);

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
                        let spot_light_arena = spot_light_arena_rc.borrow_mut();

                        match spot_light_arena.get(handle) {
                            Ok(entry) => {
                                let spot_light = &entry.item;

                                // @TODO Migrate light position to node transform.

                                pipeline.render_spot_light(spot_light, None, None);

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get SpotLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no resource handle!")
                    }
                },
            }
        };

        // Traverse the scene graph and render its nodes.

        scenegraph.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_scene_graph_node,
        )?;

        // End frame

        pipeline.end_frame();

        // Write out.

        let mut framebuffer = framebuffer_rc.borrow_mut();

        match framebuffer.attachments.color.as_mut() {
            Some(color_buffer_lock) => {
                let mut color_buffer = color_buffer_lock.borrow_mut();

                let debug_messages = &mut *debug_message_buffer_rc.borrow_mut();

                {
                    Graphics::render_debug_messages(
                        &mut *color_buffer,
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
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
