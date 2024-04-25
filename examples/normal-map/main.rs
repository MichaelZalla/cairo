extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use uuid::Uuid;

use cairo::{
    app::{resolution::RESOLUTION_1200_BY_675, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{self},
    pipeline::Pipeline,
    scene::{
        camera::Camera,
        context::SceneContext,
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
    vec::{vec3::Vec3, vec4::Vec4},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/normal-map".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: RESOLUTION_1200_BY_675,
        canvas_resolution: RESOLUTION_1200_BY_675,
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

    // Scene context

    let scene_context: SceneContext = Default::default();

    {
        let resources = scene_context.resources.borrow_mut();

        // Meshes

        let brick_wall_mesh = mesh::primitive::cube::generate(4.0, 4.0, 4.0);

        // Bricks material

        let mut brick_material = Material::new("brick".to_string());

        brick_material.specular_exponent = 32;

        brick_material.diffuse_color_map = Some(resources.texture.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                &"./examples/normal-map/assets/Brick_OldDestroyed_1k_d.tga",
                TextureMapStorageFormat::RGB24,
            ),
        ));
        brick_material.specular_exponent_map = Some(resources.texture.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                &"./examples/normal-map/assets/Brick_OldDestroyed_1k_s.tga",
                TextureMapStorageFormat::Index8(0),
            ),
        ));
        brick_material.normal_map = Some(resources.texture.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                &"./examples/normal-map/assets/Brick_OldDestroyed_1k_nY+.tga",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        brick_material.load_all_maps(&mut resources.texture.borrow_mut(), rendering_context)?;

        // Assign the meshes to entities

        let brick_wall_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_mesh);

        let brick_wall_entity = Entity::new(
            brick_wall_mesh_handle,
            Some(brick_material.name.to_string()),
        );

        // Collect materials

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(brick_material);
        }

        // Configure a global scene environment.

        let environment: Environment = Default::default();

        // Set up a camera for our scene.

        let aspect_ratio = framebuffer_rc.borrow().width_over_height;

        let mut camera: Camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -8.0,
            },
            Default::default(),
            75.0,
            aspect_ratio,
        );

        camera.movement_speed = 5.0;

        // Set up some lights for our scene.

        let ambient_light = AmbientLight {
            intensities: Vec3::ones() * 0.1,
        };

        let directional_light = DirectionalLight {
            intensities: Vec3::ones() * 0.15,
            direction: Vec4 {
                x: 0.0,
                y: 1.0,
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

        // Create resource handles from our arenas.

        let brick_wall_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_entity);

        let camera_handle = resources.camera.borrow_mut().insert(Uuid::new_v4(), camera);

        let environment_handle = resources
            .environment
            .borrow_mut()
            .insert(Uuid::new_v4(), environment);

        let ambient_light_handle = resources
            .ambient_light
            .borrow_mut()
            .insert(Uuid::new_v4(), ambient_light);

        let directional_light_handle = resources
            .directional_light
            .borrow_mut()
            .insert(Uuid::new_v4(), directional_light);

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        // Create a scene graph.

        let mut scenes = scene_context.scenes.borrow_mut();

        let mut scenegraph = SceneGraph::new();

        // Add an environment (node) to our scene.

        let mut environment_node = SceneNode::new(
            SceneNodeType::Environment,
            Default::default(),
            Some(environment_handle),
        );

        environment_node.add_child(SceneNode::new(
            SceneNodeType::AmbientLight,
            Default::default(),
            Some(ambient_light_handle),
        ))?;

        environment_node.add_child(SceneNode::new(
            SceneNodeType::DirectionalLight,
            Default::default(),
            Some(directional_light_handle),
        ))?;

        scenegraph.root.add_child(environment_node)?;

        // Add geometry nodes to our scene.

        scenegraph.root.add_child(SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(brick_wall_entity_handle),
        ))?;

        // Add camera and light nodes to our scene graph's root.

        scenegraph.root.add_child(SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        ))?;

        scenegraph.root.add_child(SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        ))?;

        scenegraph.root.add_child(SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        ))?;

        scenes.push(scenegraph);
    }

    let scene_context_rc = RefCell::new(scene_context);

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context_rc.borrow().resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.geometry_shader_options.base_color_mapping_active = false;
    pipeline.geometry_shader_options.normal_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
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
                        let mut camera_arena = resources.camera.borrow_mut();

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

                                shader_context.set_view_position(Vec4::new(
                                    camera.look_vector.get_position(),
                                    1.0,
                                ));

                                shader_context
                                    .set_view_inverse_transform(camera_view_inverse_transform);

                                shader_context.set_projection(camera.get_projection());

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
                SceneNodeType::AmbientLight => match handle {
                    Some(handle) => {
                        shader_context.set_ambient_light(Some(*handle));

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `AmbientLight` node with no resource handle!")
                    }
                },
                SceneNodeType::DirectionalLight => match handle {
                    Some(handle) => {
                        shader_context.set_directional_light(Some(*handle));

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `DirectionalLight` node with no resource handle!")
                    }
                },
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        shader_context.get_point_lights_mut().push(*handle);

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no resource handle!")
                    }
                },
                SceneNodeType::SpotLight => match handle {
                    Some(handle) => {
                        shader_context.get_spot_lights_mut().push(*handle);

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `SpotLight` node with no resource handle!")
                    }
                },
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

        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let resources = scene_context.resources.borrow();
        let scenes = scene_context.scenes.borrow();

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
                        let mesh_arena = resources.mesh.borrow_mut();
                        let entity_arena = resources.entity.borrow();

                        match entity_arena.get(handle) {
                            Ok(entry) => {
                                let entity = &entry.item;

                                pipeline.render_entity(
                                    entity,
                                    &current_world_transform,
                                    &mesh_arena,
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
                        let point_light_arena = resources.point_light.borrow();

                        match point_light_arena.get(handle) {
                            Ok(entry) => {
                                let point_light = &entry.item;

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

        scenes[0].root.visit(
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
