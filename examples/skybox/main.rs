extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::{cache::MaterialCache, Material},
    matrix::Mat4,
    mesh,
    pipeline::Pipeline,
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
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{
        cubemap::CubeMap,
        map::{TextureMap, TextureMapStorageFormat},
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/skybox".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Meshes

    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Skybox (environment) textures

    // Option 1. Skybox as a set of 6 separate textures.

    let mut skybox = CubeMap::new(
        [
            "examples/skybox/assets/sides/front.jpg",
            "examples/skybox/assets/sides/back.jpg",
            "examples/skybox/assets/sides/top.jpg",
            "examples/skybox/assets/sides/bottom.jpg",
            "examples/skybox/assets/sides/left.jpg",
            "examples/skybox/assets/sides/right.jpg",
        ],
        TextureMapStorageFormat::RGB24,
    );

    // let mut skybox = CubeMap::cross(
    //     "examples/skybox/assets/cross/horizontal_cross.png",
    //     // "examples/skybox/assets/cross/vertical_cross.png",
    //     TextureMapStorageFormat::RGB24,
    // );

    // Option 2. Skybox as one horizontal cross texture.

    // let mut skybox = CubeMap::from_cross("examples/skybox/assets/temple.png");

    // Option 3. Skybox as one vertical cross texture.

    // let mut skybox = CubeMap::from_cross("examples/skybox/assets/vertical_cross.png");

    skybox.load(rendering_context).unwrap();

    // Initialize materials

    // Checkerboard material

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(
        &"./assets/textures/checkerboard.jpg",
        TextureMapStorageFormat::Index8,
    );

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Assign textures to mesh materials

    cube_mesh.material_name = Some(checkerboard_material.name.clone());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(checkerboard_material);

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    let mut environment_arena: Arena<_> = Arena::<Environment>::new();
    let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let mut point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    let cube_entity = Entity::new(&cube_mesh);

    // Configure a global scene environment.

    let environment: Environment = Default::default();

    // Set up a camera for our scene.

    let aspect_ratio = framebuffer_rc.borrow().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        Vec3 {
            x: 0.0,
            y: 8.0,
            z: -12.0,
        },
        Vec3 {
            y: 3.0,
            ..Default::default()
        },
        75.0,
        aspect_ratio,
    );

    camera.movement_speed = 10.0;

    // Set up some lights for our scene.

    let ambient_light = AmbientLight {
        intensities: Vec3::ones() * 0.1,
    };

    let directional_light = DirectionalLight {
        intensities: Vec3::ones() * 0.3,
        direction: Vec4 {
            x: 0.25,
            y: -1.0,
            z: -0.25,
            w: 1.0,
        },
    };

    let mut point_light = PointLight::new();

    point_light.intensities = Vec3::ones() * 0.4;

    let mut spot_light = SpotLight::new();

    spot_light.look_vector.set_position(Vec3 {
        y: 10.0,
        ..spot_light.look_vector.get_position()
    });

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Pipeline

    let pipeline = Pipeline::new(
        &shader_context_rc,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let pipeline_rc = RefCell::new(pipeline);

    // Create resource handles from our arenas.

    let cube_entity_handle = entity_arena.insert(Uuid::new_v4(), cube_entity);
    let camera_handle = camera_arena.insert(Uuid::new_v4(), camera);
    let environment_handle = environment_arena.insert(Uuid::new_v4(), environment);
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

    let mut cube_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cube_entity_handle),
        None,
    );

    cube_entity_node.get_transform_mut().set_translation(Vec3 {
        y: 3.0,
        ..Default::default()
    });

    scenegraph.root.add_child(cube_entity_node)?;

    // Add camera and light nodes to our scene graph's root.

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::Camera,
        Default::default(),
        Some(camera_handle),
        None,
    ))?;

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::PointLight,
        Default::default(),
        Some(point_light_handle),
        None,
    ))?;

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
        let mut context = shader_context_rc.borrow_mut();

        context.set_ambient_light(None);
        context.set_directional_light(None);
        context.get_point_lights_mut().clear();
        context.get_spot_lights_mut().clear();

        let uptime = app.timing_info.uptime_seconds;

        // Traverse the scene graph and update its nodes.

        let mut scenegraph = scenegraph_rc.borrow_mut();

        let mut point_lights_visited: usize = 0;
        let mut spot_lights_visited: usize = 0;

        let mut update_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
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

                                if entity.mesh.object_name == "plane" {
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

                        match camera_arena.get_mut(handle) {
                            Ok(entry) => {
                                let camera = &mut entry.item;

                                camera.update(
                                    &app.timing_info,
                                    keyboard_state,
                                    mouse_state,
                                    game_controller_state,
                                );

                                let camera_view_inverse_transform =
                                    camera.get_view_inverse_transform();

                                context.set_view_position(Vec4::new(
                                    camera.look_vector.get_position(),
                                    1.0,
                                ));

                                context.set_view_inverse_transform(camera_view_inverse_transform);

                                context.set_projection(camera.get_projection());

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
                        let mut point_light_arena = point_light_arena_rc.borrow_mut();

                        match point_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let point_light = &mut entry.item;

                                static POINT_LIGHT_INTENSITY_PHASE_SHIFT: f32 = 2.0 * PI / 3.0;
                                static MAX_POINT_LIGHT_INTENSITY: f32 = 0.5;

                                point_light.intensities = Vec3 {
                                    x: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0
                                        + 0.5,
                                    y: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0
                                        + 0.5,
                                    z: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0
                                        + 0.5,
                                } * MAX_POINT_LIGHT_INTENSITY;

                                let orbital_radius: f32 = 3.0;

                                point_light.position = Vec3 {
                                    x: orbital_radius * uptime.sin(),
                                    y: 3.0,
                                    z: orbital_radius * uptime.cos(),
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
                        let mut spot_light_arena = spot_light_arena_rc.borrow_mut();

                        match spot_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let spot_light = &mut entry.item;

                                spot_light.look_vector.set_position(
                                    (Vec4::new(Default::default(), 1.0) * current_world_transform)
                                        .to_vec3(),
                                );

                                spot_light.look_vector.set_target_position(
                                    (Vec4::new(vec3::UP * -1.0, 1.0) * current_world_transform)
                                        .to_vec3(),
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

        pipeline
            .geometry_shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline.bind_framebuffer(Some(&framebuffer_rc));

        // Begin frame

        pipeline.begin_frame();

        // Bind skybox (environment) map to shader context.

        {
            let mut context = shader_context_rc.borrow_mut();

            let cubemap_raw_mut = &skybox as *const CubeMap;

            context.set_active_environment_map(Some(cubemap_raw_mut));
        }

        // Render scene.

        let scenegraph = scenegraph_rc.borrow_mut();

        let mut active_camera: Option<Camera> = None;

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
                                    Some(&material_cache),
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
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        let camera_arena = camera_arena_rc.borrow();

                        match camera_arena.get(handle) {
                            Ok(entry) => {
                                let camera = &entry.item;

                                active_camera = Some(camera.clone());

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get Entity from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `Camera` node with no resource handle!")
                    }
                },
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
                        let mut spot_light_arena = spot_light_arena_rc.borrow_mut();

                        match spot_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let spot_light = &mut entry.item;

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

        match active_camera {
            Some(camera) => {
                pipeline.render_skybox(&skybox, &camera);

                // pipeline.render_ground_plane(1.0);
            }
            None => (),
        }

        {
            let mut context = shader_context_rc.borrow_mut();

            context.set_active_environment_map(None);
        }

        pipeline.end_frame();

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.borrow();

                Ok(color_buffer.get_all().clone())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
