extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use sdl2::keyboard::Keycode;

use uuid::Uuid;

use cairo::{
    app::{resolution::RESOLUTIONS_16X9, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    debug::message::DebugMessageBuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh,
    pipeline::{zbuffer::DepthTestMethod, Pipeline},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        environment::Environment,
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
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::{vec3::Vec3, vec4::Vec4},
};

fn main() -> Result<(), String> {
    let current_resolution_index_rc: RefCell<usize> = RefCell::new(2);

    let resolution = RESOLUTIONS_16X9[*current_resolution_index_rc.borrow()];

    let mut window_info = AppWindowInfo {
        title: "examples/generate-primitives".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: resolution,
        canvas_resolution: resolution,
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

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(32.0, 32.0, 1, 1);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);
    let mut cone_mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);
    let mut cylinder_mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

    // Create a new textured material

    let mut checkerboard_mat = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(
        &"./assets/textures/checkerboard.jpg",
        TextureMapStorageFormat::Index8(0),
    );

    // Checkerboard material

    checkerboard_diffuse_map.options.wrapping = TextureMapWrapping::Repeat;

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    // Pump up diffuse value of the darkest pixels

    checkerboard_diffuse_map.map(|r, g, b| {
        if r < 4 && g < 4 && b < 4 {
            return (18, 18, 18);
        }
        (r, g, b)
    })?;

    checkerboard_mat.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_mat.specular_exponent = 8;

    checkerboard_mat.specular_map = Some(checkerboard_specular_map);

    // Point light decal material

    let mut point_light_decal_mat = Material::new("point_light_decal".to_string());

    point_light_decal_mat.alpha_map = Some(TextureMap::new(
        &"./assets/decals/point_light_small.png",
        TextureMapStorageFormat::Index8(0),
    ));

    point_light_decal_mat.emissive_map = point_light_decal_mat.alpha_map.clone();

    point_light_decal_mat.load_all_maps(rendering_context)?;

    // Spot light decal material

    let mut spot_light_decal_mat = Material::new("spot_light_decal".to_string());

    spot_light_decal_mat.alpha_map = Some(TextureMap::new(
        &"./assets/decals/spot_light_small.png",
        TextureMapStorageFormat::Index8(0),
    ));

    spot_light_decal_mat.emissive_map = spot_light_decal_mat.alpha_map.clone();

    spot_light_decal_mat.load_all_maps(rendering_context)?;

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_mat.name.clone());
    cube_mesh.material_name = Some(checkerboard_mat.name.clone());
    cone_mesh.material_name = Some(checkerboard_mat.name.clone());
    cylinder_mesh.material_name = Some(checkerboard_mat.name.clone());

    // Collect materials

    let mut materials_cache: MaterialCache = Default::default();

    materials_cache.insert(checkerboard_mat);
    materials_cache.insert(point_light_decal_mat);
    materials_cache.insert(spot_light_decal_mat);

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    let mut environment_arena: Arena<_> = Arena::<Environment>::new();
    let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    let plane_entity = Entity::new(&plane_mesh);
    let cube_entity = Entity::new(&cube_mesh);
    let cone_entity = Entity::new(&cone_mesh);
    let cylinder_entity = Entity::new(&cylinder_mesh);

    // Configure a global scene environment.

    let environment: Environment = Default::default();

    // Set up some cameras for our scene.

    let aspect_ratio = framebuffer_rc.borrow().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        Vec3 {
            x: 15.0,
            y: 8.0,
            z: -15.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: -7.5,
        }
        .as_normal(),
        75.0,
        aspect_ratio,
    );

    camera.set_projection_z_far(100.0);

    camera.movement_speed = 10.0;

    let camera2: Camera = Camera::from_perspective(
        Vec3 {
            x: 4.0,
            y: 8.0,
            z: -4.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: -7.5,
        }
        .as_normal(),
        75.0,
        aspect_ratio,
    );

    let looking_at_point_light_rc = RefCell::new(false);

    // Set up some lights for our scene.

    let ambient_light = AmbientLight {
        intensities: Vec3::ones() * 0.1,
    };

    let directional_light = DirectionalLight {
        intensities: Vec3::ones() * 0.15,
        direction: Vec4 {
            x: -1.0,
            y: -1.0,
            z: 1.0,
            w: 1.0,
        }
        .as_normal(),
    };

    let mut point_lights: Vec<PointLight> = vec![];

    let light_grid_subdivisions: usize = 1;
    let light_grid_size = 20.0;

    for x in 0..(light_grid_subdivisions + 1) {
        for z in 0..(light_grid_subdivisions + 1) {
            let mut light = PointLight::new();

            light.position = Vec3 {
                x: -(light_grid_size / 2.0)
                    + (x as f32 / light_grid_subdivisions as f32) * light_grid_size,
                y: 1.0,
                z: -(light_grid_size / 2.0)
                    + (z as f32 / light_grid_subdivisions as f32) * light_grid_size,
            };

            point_lights.push(light);
        }
    }

    let mut spot_light: SpotLight = SpotLight::new();

    spot_light.look_vector.set_position(Vec3 {
        x: -6.0,
        y: 15.0,
        z: -6.0,
    });

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

    let pipeline = Pipeline::new(
        &shader_context_rc,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let pipeline_rc = RefCell::new(pipeline);

    // Create resource handles from our arenas.

    let plane_entity_handle = entity_arena.insert(Uuid::new_v4(), plane_entity);
    let cube_entity_handle = entity_arena.insert(Uuid::new_v4(), cube_entity);
    let cone_entity_handle = entity_arena.insert(Uuid::new_v4(), cone_entity);
    let cylinder_entity_handle = entity_arena.insert(Uuid::new_v4(), cylinder_entity);

    let camera_handle = camera_arena.insert(Uuid::new_v4(), camera);
    let camera2_handle = camera_arena.insert(Uuid::new_v4(), camera2);

    let active_camera_handle_rc = RefCell::new(camera_handle.clone());

    let environment_handle = environment_arena.insert(Uuid::new_v4(), environment);
    let ambient_light_handle = ambient_light_arena.insert(Uuid::new_v4(), ambient_light);

    let directional_light_handle =
        directional_light_arena.insert(Uuid::new_v4(), directional_light);

    let spot_light_handle = spot_light_arena.insert(Uuid::new_v4(), spot_light);

    let entity_arena_rc = RefCell::new(entity_arena);
    let camera_arena_rc = RefCell::new(camera_arena);
    let ambient_light_arena_rc = RefCell::new(ambient_light_arena);
    let directional_light_arena_rc = RefCell::new(directional_light_arena);
    let point_light_arena_rc = RefCell::new(point_light_arena);
    let spot_light_arena_rc = RefCell::new(spot_light_arena);

    // Create a scene graph.

    let mut scenegraph = SceneGraph::new();

    // Add an environment (node) to our scene.

    let mut environment_node = SceneNode::new(
        SceneNodeType::Environment,
        Default::default(),
        Some(environment_handle),
        None,
    );

    environment_node.add_child(SceneNode::new(
        SceneNodeType::AmbientLight,
        Default::default(),
        Some(ambient_light_handle),
        None,
    ))?;

    environment_node.add_child(SceneNode::new(
        SceneNodeType::DirectionalLight,
        Default::default(),
        Some(directional_light_handle),
        None,
    ))?;

    scenegraph.root.add_child(environment_node)?;

    // Add geometry nodes to our scene.

    let mut plane_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(plane_entity_handle),
        None,
    );

    plane_entity_node.get_transform_mut().set_translation(Vec3 {
        x: -5.0,
        z: -5.0,
        ..Default::default()
    });

    scenegraph.root.add_child(plane_entity_node)?;

    let mut cube_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cube_entity_handle),
        None,
    );

    cube_entity_node.get_transform_mut().set_translation(Vec3 {
        x: -4.0,
        y: 1.0,
        ..Default::default()
    });

    scenegraph.root.add_child(cube_entity_node)?;

    let mut cone_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cone_entity_handle),
        None,
    );

    cone_entity_node.get_transform_mut().set_translation(Vec3 {
        x: 0.0,
        y: 1.0,
        ..Default::default()
    });

    scenegraph.root.add_child(cone_entity_node)?;

    let mut cylinder_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cylinder_entity_handle),
        None,
    );

    cylinder_entity_node
        .get_transform_mut()
        .set_translation(Vec3 {
            x: 4.0,
            y: 1.0,
            ..Default::default()
        });

    scenegraph.root.add_child(cylinder_entity_node)?;

    // Add camera and light nodes to our scene graph's root.

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::Camera,
        Default::default(),
        Some(camera_handle),
        None,
    ))?;

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::Camera,
        Default::default(),
        Some(camera2_handle),
        None,
    ))?;

    let point_lights_count = point_lights.len();

    {
        let mut point_light_arena = point_light_arena_rc.borrow_mut();

        for point_light in point_lights {
            let point_light_handle = point_light_arena.insert(Uuid::new_v4(), point_light);

            scenegraph.root.add_child(SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(point_light_handle),
                None,
            ))?;
        }
    }

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::SpotLight,
        Default::default(),
        Some(spot_light_handle),
        None,
    ))?;

    // Prints the scenegraph to stdout.

    println!("{}", scenegraph);

    let scenegraph_rc = RefCell::new(scenegraph);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let mut pipeline = pipeline_rc.borrow_mut();

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::R { .. } => {
                    // Resize the app's rendering canvas.

                    let mut current_resolution_index = current_resolution_index_rc.borrow_mut();

                    *current_resolution_index =
                        (*current_resolution_index + 1) % RESOLUTIONS_16X9.len();

                    let new_resolution = RESOLUTIONS_16X9[*current_resolution_index];

                    app.resize_window(new_resolution).unwrap();

                    app.resize_canvas(new_resolution).unwrap();

                    // Resize the framebuffer to match.
                    let mut framebuffer = framebuffer_rc.borrow_mut();

                    framebuffer.resize(new_resolution.width, new_resolution.height, true);
                }
                Keycode::H { .. } => {
                    let mut active_fragment_shader_index =
                        active_fragment_shader_index_rc.borrow_mut();

                    *active_fragment_shader_index += 1;

                    if *active_fragment_shader_index == fragment_shaders.len() {
                        *active_fragment_shader_index = 0;
                    }

                    // let mut pipeline = pipeline_rc.borrow_mut();

                    pipeline.set_fragment_shader(fragment_shaders[*active_fragment_shader_index]);
                }
                Keycode::L { .. } => {
                    let mut looking_at_point_light = looking_at_point_light_rc.borrow_mut();

                    *looking_at_point_light = !*looking_at_point_light;
                }
                _ => {}
            }
        }

        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            app.window_info.canvas_resolution.width, app.window_info.canvas_resolution.height
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

        static COLOR_CHANNEL_PHASE_SHIFT: f32 = 2.0 * PI / 3.0;

        let mut point_lights_visited: usize = 0;
        let mut spot_lights_visited: usize = 0;

        let mut update_scene_graph_node = |_current_depth: usize,
                                           _current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Scene => Ok(()),
                SceneNodeType::Environment => Ok(()),
                SceneNodeType::Skybox => Ok(()),
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mut entity_arena = entity_arena_rc.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                if entity
                                    .mesh
                                    .object_name
                                    .as_ref()
                                    .is_some_and(|n| n == "plane")
                                {
                                    return Ok(());
                                }

                                static ENTITY_ROTATION_SPEED: f32 = 0.3;

                                let mut rotation = *node.get_transform().rotation();

                                rotation.z += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.z %= 2.0 * PI;

                                rotation.x += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.x %= 2.0 * PI;

                                rotation.y += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.y %= 2.0 * PI;

                                node.get_transform_mut().set_rotation(rotation);

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
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        let mut camera_arena = camera_arena_rc.borrow_mut();

                        if *handle != *active_camera_handle_rc.borrow() {
                            return Ok(());
                        }

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
                        let arena = directional_light_arena_rc.borrow();

                        match arena.get(handle) {
                            Ok(entry) => {
                                let light = &entry.item;

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

                                static LIGHT_SPEED_FACTOR: f32 = 0.66;
                                static ORBIT_RADIUS: f32 = 12.0;
                                static MAX_POINT_LIGHT_INTENSITY: f32 = 25.0;

                                let light_phase_shift = (2.0 * PI / (point_lights_count as f32))
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

                                context.get_point_lights_mut().push(light.clone());

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

                                static MAX_SPOT_LIGHT_INTENSITY: f32 = 25.0;

                                light.intensities = Vec3 {
                                    x: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 0.0).cos() / 2.0 + 0.5,
                                    y: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 1.0).cos() / 2.0 + 0.5,
                                    z: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 2.0).cos() / 2.0 + 0.5,
                                } * MAX_SPOT_LIGHT_INTENSITY;

                                context.get_spot_lights_mut().push(light.clone());

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

        debug_message_buffer.write(format!(
            "Visualize normals: {}",
            if pipeline.options.do_visualize_normals {
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
                SceneNodeType::Environment => Ok(()),
                SceneNodeType::Skybox => Ok(()),
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mut entity_arena = entity_arena_rc.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                pipeline.render_entity(
                                    entity,
                                    &current_world_transform,
                                    Some(&materials_cache),
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
                        let camera_arena = camera_arena_rc.borrow_mut();

                        let mut point_light_arena = point_light_arena_rc.borrow_mut();

                        match camera_arena.get(&*active_camera_handle_rc.borrow()) {
                            Ok(entry) => {
                                let active_camera = &entry.item;

                                match point_light_arena.get_mut(handle) {
                                    Ok(entry) => {
                                        let point_light = &mut entry.item;

                                        pipeline.render_point_light(
                                            &point_light,
                                            Some(active_camera),
                                            Some(&mut materials_cache),
                                        );

                                        Ok(())
                                    }
                                    Err(err) => panic!(
                                        "Failed to get PointLight from Arena with Handle {:?}: {}",
                                        handle, err
                                    ),
                                }
                            }
                            Err(err) => panic!(
                                "Failed to get Camera from Arena with Handle {:?}: {}",
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
                        let camera_arena = camera_arena_rc.borrow_mut();

                        let mut spot_light_arena = spot_light_arena_rc.borrow_mut();

                        match camera_arena.get(&*active_camera_handle_rc.borrow()) {
                            Ok(entry) => {
                                let active_camera = &entry.item;

                                match spot_light_arena.get_mut(handle) {
                                    Ok(entry) => {
                                        let spot_light = &mut entry.item;

                                        pipeline.render_spot_light(
                                            &spot_light,
                                            Some(active_camera),
                                            Some(&mut materials_cache),
                                        );

                                        Ok(())
                                    }
                                    Err(err) => panic!(
                                        "Failed to get PointLight from Arena with Handle {:?}: {}",
                                        handle, err
                                    ),
                                }
                            }
                            Err(err) => panic!(
                                "Failed to get Camera from Arena with Handle {:?}: {}",
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
