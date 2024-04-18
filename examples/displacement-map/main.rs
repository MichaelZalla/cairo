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
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/diplacement-map".to_string(),
        full_screen: false,
        vertical_sync: true,
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

    let mut brick_wall_mesh = mesh::primitive::cube::generate(4.0, 4.0, 4.0);

    let mut box_mesh = brick_wall_mesh.clone();

    // Initialize materials

    // Bricks material

    let mut brick_material = Material::new("bricks".to_string());

    brick_material.diffuse_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2.jpg",
        TextureMapStorageFormat::RGB24,
    ));

    brick_material.normal_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2_normal.jpg",
        TextureMapStorageFormat::RGB24,
    ));

    brick_material.displacement_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2_disp.jpg",
        TextureMapStorageFormat::Index8(0),
    ));

    brick_material.displacement_scale = 0.05;

    brick_material.load_all_maps(rendering_context)?;

    // Box material

    let mut box_material = Material::new("box".to_string());

    box_material.diffuse_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/wood.png",
        TextureMapStorageFormat::RGB24,
    ));

    box_material.normal_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/toy_box_normal.png",
        TextureMapStorageFormat::RGB24,
    ));

    box_material.displacement_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/toy_box_disp.png",
        TextureMapStorageFormat::Index8(0),
    ));

    box_material.displacement_scale = 0.05;

    box_material.load_all_maps(rendering_context)?;

    // Assign textures to mesh materials

    brick_wall_mesh.material_name = Some(brick_material.name.to_string());

    box_mesh.material_name = Some(box_material.name.to_string());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(brick_material);
    material_cache.insert(box_material);

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    let mut environment_arena: Arena<_> = Arena::<Environment>::new();
    let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let mut point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    let brick_wall_entity = Entity::new(&brick_wall_mesh);
    let box_entity = Entity::new(&box_mesh);

    // Configure a global scene environment.

    let environment: Environment = Default::default();

    // Set up a camera for our scene.

    let aspect_ratio = framebuffer_rc.borrow().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: -12.0,
        },
        Default::default(),
        75.0,
        aspect_ratio,
    );

    camera.movement_speed = 10.0;

    // Set up some lights for our scene.

    let ambient_light = AmbientLight {
        intensities: Vec3::ones() * 0.1,
    };

    let directional_light = DirectionalLight {
        intensities: Vec3::ones() * 0.2,
        direction: Vec4 {
            x: 1.0,
            y: -1.0,
            z: 1.0,
            w: 1.0,
        }
        .as_normal(),
    };

    let mut point_light = PointLight::new();

    point_light.position.y = 0.0;
    point_light.position.z = -4.0;

    point_light.intensities = Vec3::ones() * 10.0;
    point_light.specular_intensity = 10.0;
    point_light.constant_attenuation = 1.0;
    point_light.linear_attenuation = 0.35;
    point_light.quadratic_attenuation = 0.44;

    let spot_light = SpotLight::new();

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.geometry_shader_options.normal_mapping_active = true;
    pipeline.geometry_shader_options.displacement_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // Create resource handles from our arenas.

    let brick_wall_entity_handle = entity_arena.insert(Uuid::new_v4(), brick_wall_entity);
    let box_entity_handle = entity_arena.insert(Uuid::new_v4(), box_entity);
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

    let mut brick_wall_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(brick_wall_entity_handle),
        None,
    );

    brick_wall_entity_node
        .get_transform_mut()
        .set_translation(Vec3 {
            x: -4.0,
            y: 0.0,
            z: 2.0,
        });

    scenegraph.root.add_child(brick_wall_entity_node)?;

    let mut box_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(box_entity_handle),
        None,
    );

    box_entity_node.get_transform_mut().set_translation(Vec3 {
        x: 4.0,
        y: 0.0,
        z: 2.0,
    });

    scenegraph.root.add_child(box_entity_node)?;

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
                SceneNodeType::Entity => {
                    static ENTITY_ROTATION_SPEED: f32 = 0.1;

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
                        let mut point_light_arena = point_light_arena_rc.borrow_mut();

                        match point_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let point_light = &mut entry.item;

                                let orbital_radius: f32 = 6.0;

                                point_light.position = Vec3 {
                                    x: 4.0 + orbital_radius * uptime.sin(),
                                    y: orbital_radius * uptime.cos(),
                                    z: -4.0,
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
                SceneNodeType::SpotLight => Ok(()),
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
