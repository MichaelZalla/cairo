extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    pipeline::Pipeline,
    resource::arena::Arena,
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
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/diffuse-map".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Meshes and materials

    let (mut cube_meshes, mut cube_materials_cache) =
        mesh::obj::load_obj(&"./data/obj/cube-textured.obj");

    let cube_mesh = &mut cube_meshes[0];

    match &mut cube_materials_cache {
        Some(cache) => {
            for material in cache.values_mut() {
                material.load_all_maps(rendering_context).unwrap();
            }
        }
        None => (),
    }

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    // let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let mut point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    let cube_entity = Entity::new(&cube_mesh);

    // Set up a camera for rendering our scene

    let aspect_ratio = framebuffer_rc.borrow().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: -4.0,
        },
        Default::default(),
        75.0,
        aspect_ratio,
    );

    camera.movement_speed = 5.0;

    // Set up some lights for our scene.

    let ambient_light = AmbientLight {
        intensities: Vec3::ones() * 0.4,
    };

    let directional_light = DirectionalLight {
        intensities: Vec3::ones() * 0.3,
        direction: Vec4 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
            w: 1.0,
        }
        .as_normal(),
    };

    let mut point_light = PointLight::new();

    point_light.intensities = Vec3::ones() * 0.7;

    point_light.position = Vec3 {
        x: 0.0,
        y: 4.0,
        z: 0.0,
    };

    let spot_light = SpotLight::new();

    // Bind initial state to our shader context.

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    {
        let mut context = shader_context_rc.borrow_mut();

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);
    }

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.geometry_shader_options.normal_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // Create resource handles from our arenas.

    let cube_entity_handle = entity_arena.insert(Uuid::new_v4(), cube_entity);
    let camera_handle = camera_arena.insert(Uuid::new_v4(), camera);
    // let ambient_light_handle = ambient_light_arena.insert(Uuid::new_v4(), ambient_light);
    let directional_light_handle =
        directional_light_arena.insert(Uuid::new_v4(), directional_light);
    let point_light_handle = point_light_arena.insert(Uuid::new_v4(), point_light);
    let spot_light_handle = spot_light_arena.insert(Uuid::new_v4(), spot_light);

    let entity_arena_rc = RefCell::new(entity_arena);
    let camera_arena_rc = RefCell::new(camera_arena);
    // let ambient_light_arena_rc = RefCell::new(ambient_light_arena);
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

    scenegraph.root.add_child(SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cube_entity_handle),
        None,
    ));

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

        // Traverse the scene graph and update its nodes.

        let mut scenegraph = scenegraph_rc.borrow_mut();

        let mut update_scene_graph_node = |_current_depth: usize,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Empty => Ok(()),
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
                SceneNodeType::DirectionalLight => match handle {
                    Some(handle) => {
                        let arena = directional_light_arena_rc.borrow();

                        match arena.get(handle) {
                            Ok(entry) => {
                                let light = &entry.item;

                                context.set_directional_light(*light);

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
                        let arena = point_light_arena_rc.borrow();

                        match arena.get(handle) {
                            Ok(entry) => {
                                let light = &entry.item;

                                context.set_point_light(0, *light);

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
                        let arena = spot_light_arena_rc.borrow();

                        match arena.get(handle) {
                            Ok(entry) => {
                                let light = &entry.item;

                                context.set_spot_light(0, *light);

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

        let mut render_scene_graph_node =
            |_current_depth: usize, node: &SceneNode| -> Result<(), String> {
                let (node_type, handle) = (node.get_type(), node.get_handle());

                match node_type {
                    SceneNodeType::Empty => Ok(()),
                    SceneNodeType::Entity => match handle {
                        Some(handle) => {
                            let mut entity_arena = entity_arena_rc.borrow_mut();

                            match entity_arena.get_mut(handle) {
                                Ok(entry) => {
                                    let entity = &mut entry.item;

                                    let world_transform = node.get_transform().mat();

                                    pipeline.render_entity(
                                        entity,
                                        &world_transform,
                                        cube_materials_cache.as_ref(),
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
