extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    effect::Effect,
    effects::{
        dilation_effect::DilationEffect, grayscale_effect::GrayscaleEffect,
        invert_effect::InvertEffect, kernel_effect::KernelEffect,
    },
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
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/post-effects".to_string(),
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

        let plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
        let cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

        // Checkerboard material

        let mut checkerboard_material = Material::new("checkerboard".to_string());

        let mut checkerboard_diffuse_map = TextureMap::new(
            &"./assets/textures/checkerboard.jpg",
            TextureMapStorageFormat::Index8(0),
        );

        checkerboard_diffuse_map.load(rendering_context)?;

        let checkerboard_diffuse_map_handle = resources
            .texture
            .borrow_mut()
            .insert(Uuid::new_v4(), checkerboard_diffuse_map);

        checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map_handle);
        checkerboard_material.specular_map = Some(checkerboard_diffuse_map_handle);

        // Lava material

        let mut lava_material = Material::new("container".to_string());

        lava_material.diffuse_map = Some(resources.texture.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                &"./examples/post-effects/assets/lava.png",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        lava_material.emissive_map = Some(resources.texture.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                &"./examples/post-effects/assets/lava_emissive.png",
                TextureMapStorageFormat::Index8(0),
            ),
        ));

        lava_material
            .load_all_maps(&mut resources.texture.borrow_mut(), rendering_context)
            .unwrap();

        // Assign the meshes to entities

        let plane_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), plane_mesh);

        let plane_entity = Entity::new(plane_mesh_handle, Some(checkerboard_material.name.clone()));

        let cube_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), cube_mesh);

        let cube_entity = Entity::new(cube_mesh_handle, Some(lava_material.name.clone()));

        // Collect materials

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(checkerboard_material);
            materials.insert(lava_material);
        }

        // Configure a global scene environment.

        let environment: Environment = Default::default();

        // Set up a camera for our scene.

        let aspect_ratio = framebuffer_rc.borrow().width_over_height;

        let mut camera: Camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: -10.0,
            },
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: 0.0,
            },
            75.0,
            aspect_ratio,
        );

        camera.movement_speed = 5.0;

        // Set up some lights for our scene.

        let ambient_light: AmbientLight = Default::default();

        let directional_light = DirectionalLight {
            intensities: Default::default(),
            direction: Vec4 {
                x: -1.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
        };

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.8;

        let mut spot_light = SpotLight::new();

        spot_light.intensities = Vec3::ones() * 0.1;

        spot_light.look_vector.set_position(Vec3 {
            y: 30.0,
            ..spot_light.look_vector.get_position()
        });

        // Create resource handles from our arenas.

        let plane_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), plane_entity);

        let cube_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), cube_entity);

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

        let mut plane_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(plane_entity_handle),
        );

        let mut cube_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(cube_entity_handle),
        );

        cube_entity_node.get_transform_mut().set_translation(Vec3 {
            y: 3.0,
            ..Default::default()
        });

        plane_entity_node.add_child(cube_entity_node)?;

        scenegraph.root.add_child(plane_entity_node)?;

        // Add camera and light nodes to our scene graph's root.

        let camera_node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        );

        scenegraph.root.add_child(camera_node)?;

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

    pipeline.geometry_shader_options.emissive_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // Create several screen-space post-processing effects.

    let _outline_effect = DilationEffect::new(color::BLUE, color::BLACK, Some(2));
    let _grayscale_effect = GrayscaleEffect {};
    let _invert_effect = InvertEffect {};
    let _sharpen_kernel_effect = KernelEffect::new([2, 2, 2, 2, -15, 2, 2, 2, 2], None);
    let _blur_kernel_effect = KernelEffect::new([1, 2, 1, 2, 4, 2, 1, 2, 1], Some(8));
    let edge_detection_kernel_effect = KernelEffect::new([1, 1, 1, 1, -8, 1, 1, 1, 1], None);

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

        let uptime = app.timing_info.uptime_seconds;

        // Traverse the scene graph and update its nodes.

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
                        let mesh_arena = resources.mesh.borrow();
                        let mut entity_arena = resources.entity.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                if let Ok(entry) = mesh_arena.get(&entity.mesh) {
                                    let mesh = &entry.item;

                                    if let Some(object_name) = &mesh.object_name {
                                        if object_name == "plane" {
                                            return Ok(());
                                        }
                                    }
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
                        let mut point_light_arena = resources.point_light.borrow_mut();

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
                        let mut spot_light_arena = resources.spot_light.borrow_mut();

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
                        let mesh_arena = resources.mesh.borrow();
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
                SceneNodeType::SpotLight => match handle {
                    Some(handle) => {
                        let spot_light_arena = resources.spot_light.borrow();

                        match spot_light_arena.get(handle) {
                            Ok(entry) => {
                                let spot_light = &entry.item;

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

                let prepost_u32 = color_buffer.get_all().clone();

                // Perform a post-processing pass by applying the dilation effect.

                let mut buffer = Buffer2D::from_data(
                    window_info.canvas_resolution.width,
                    window_info.canvas_resolution.height,
                    prepost_u32,
                );

                let effects: Vec<&dyn Effect> = vec![
                    // &outline_effect,
                    // &invert_effect,
                    // &grayscale_effect,
                    // &sharpen_kernel_effect,
                    // &blur_kernel_effect,
                    &edge_detection_kernel_effect,
                ];

                for effect in effects {
                    effect.apply(&mut buffer);
                }

                // Return the post-processed pixels.

                Ok(buffer.get_all().clone())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
