extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use uuid::Uuid;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    effect::Effect,
    effects::{
        dilation_effect::DilationEffect, grayscale_effect::GrayscaleEffect,
        invert_effect::InvertEffect, kernel_effect::KernelEffect,
    },
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh,
    scene::{
        context::utils::make_empty_scene,
        light::{PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
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

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = Rc::new(make_empty_scene(framebuffer_rc.borrow().width_over_height)?);

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a textured ground plane to our scene.

        {
            let mut materials = resources.material.borrow_mut();

            let checkerboard_material = {
                let mut material = Material::new("checkerboard".to_string());

                let mut albedo_map = TextureMap::new(
                    "./assets/textures/checkerboard.jpg",
                    TextureMapStorageFormat::Index8(0),
                );

                // Checkerboard material

                albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

                albedo_map.load(rendering_context)?;

                let albedo_map_handle = resources
                    .texture_u8
                    .borrow_mut()
                    .insert(Uuid::new_v4(), albedo_map);

                material.albedo_map = Some(albedo_map_handle);

                material
            };

            materials.insert(checkerboard_material);
        }

        let mut plane_entity_node = {
            let mut mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);

            mesh.material_name = Some("checkerboard".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                z: 3.0,
                y: -3.0,
                ..Default::default()
            });

            node
        };

        // Add a container (cube) to our scene.

        {
            let mut materials = resources.material.borrow_mut();

            let emissive_material = {
                let mut material = Material::new("emissive".to_string());

                material.albedo_map = Some(resources.texture_u8.borrow_mut().insert(
                    Uuid::new_v4(),
                    TextureMap::new(
                        "./examples/post-effects/assets/lava.png",
                        TextureMapStorageFormat::RGB24,
                    ),
                ));

                material.emissive_color_map = Some(resources.texture_u8.borrow_mut().insert(
                    Uuid::new_v4(),
                    TextureMap::new(
                        "./examples/post-effects/assets/lava_emissive.png",
                        TextureMapStorageFormat::Index8(0),
                    ),
                ));

                material
                    .load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)
                    .unwrap();

                material
            };

            materials.insert(emissive_material);
        }

        let cube_entity_node = {
            let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("emissive".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                y: 3.0,
                ..Default::default()
            });

            node
        };

        plane_entity_node.add_child(cube_entity_node)?;

        scene.root.add_child(plane_entity_node)?;

        // Add a point light to our scene.

        let point_light_node = {
            let mut light = PointLight::new();

            light.intensities = Vec3::ones() * 0.8;

            let light_handle = resources
                .point_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(light_handle),
            )
        };

        scene.root.add_child(point_light_node)?;

        // Add a spot light to our scene.

        let spot_light_node = {
            let mut light = SpotLight::new();

            light.intensities = Vec3::ones() * 0.1;

            light.look_vector.set_position(Vec3 {
                y: 30.0,
                ..light.look_vector.get_position()
            });

            let light_handle = resources
                .spot_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            SceneNode::new(
                SceneNodeType::SpotLight,
                Default::default(),
                Some(light_handle),
            )
        };

        scene.root.add_child(spot_light_node)?;
    }

    // Shader context

    let shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.emissive_color_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // Create several screen-space post-processing effects.

    let _outline_effect = DilationEffect::new(color::BLUE, color::BLACK, Some(2));
    let _grayscale_effect = GrayscaleEffect {};
    let _invert_effect = InvertEffect {};
    let _sharpen_kernel_effect = KernelEffect::new([2, 2, 2, 2, -15, 2, 2, 2, 2], None);
    let _blur_kernel_effect = KernelEffect::new([1, 2, 1, 2, 4, 2, 1, 2, 1], Some(8));
    let edge_detection_kernel_effect = KernelEffect::new([1, 1, 1, 1, -8, 1, 1, 1, 1], None);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

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

        let mut renderer = renderer_rc.borrow_mut();

        renderer
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        renderer
            .shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
        // Render scene.

        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        match scene.render(&resources, &renderer_rc, None) {
            Ok(()) => {
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
            }
            Err(e) => panic!("{}", e),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}
