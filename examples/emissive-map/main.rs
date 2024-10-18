extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh,
    render::options::RenderOptions,
    scene::{
        context::utils::make_empty_scene,
        light::{PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/emissive-map".to_string(),
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let (scene_context, shader_context) =
        make_empty_scene(framebuffer_rc.borrow().width_over_height)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a textured ground plane to our scene.

        let checkerboard_material_handle = {
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

                let albedo_map_handle = resources.texture_u8.borrow_mut().insert(albedo_map);

                material.albedo_map = Some(albedo_map_handle);

                material
            };

            materials.insert(checkerboard_material)
        };

        let mut plane_entity_node = {
            let mut mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);

            mesh.material = Some(checkerboard_material_handle);

            let mesh_handle = resources.mesh.borrow_mut().insert(mesh);

            let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

            let entity_handle = resources.entity.borrow_mut().insert(entity);

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

        let emissive_material_handle = {
            let mut materials = resources.material.borrow_mut();

            let emissive_material = {
                let mut material = Material::new("emissive".to_string());

                material.albedo_map =
                    Some(resources.texture_u8.borrow_mut().insert(TextureMap::new(
                        "./examples/post-effects/assets/lava.png",
                        TextureMapStorageFormat::RGB24,
                    )));

                material.emissive_color_map =
                    Some(resources.texture_u8.borrow_mut().insert(TextureMap::new(
                        "./examples/post-effects/assets/lava_emissive.png",
                        TextureMapStorageFormat::Index8(0),
                    )));

                material
                    .load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)
                    .unwrap();

                material
            };

            materials.insert(emissive_material)
        };

        let cube_entity_node = {
            let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

            let mesh_handle = resources.mesh.borrow_mut().insert(mesh);

            let entity = Entity::new(mesh_handle, Some(emissive_material_handle));

            let entity_handle = resources.entity.borrow_mut().insert(entity);

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

            let light_handle = resources.point_light.borrow_mut().insert(light);

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

            let light_handle = resources.spot_light.borrow_mut().insert(light);

            SceneNode::new(
                SceneNodeType::SpotLight,
                Default::default(),
                Some(light_handle),
            )
        };

        scene.root.add_child(spot_light_node)?;
    }

    // Scene context

    let scene_context_rc = Rc::new(scene_context);

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let render_options = RenderOptions {
        do_bloom: true,
        ..Default::default()
    };

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context_rc.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        render_options,
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.emissive_color_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = scene_context_rc.resources.borrow_mut();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.clear_lights();

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

                                let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                                let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                                node.get_transform_mut().set_rotation(q);

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

        renderer.options.update(keyboard_state);

        renderer.shader_options.update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        // Render scene.

        let resources = scene_context_rc.resources.borrow();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
        let scene = &mut scenes[0];

        match scene.render(&resources, &renderer_rc, None) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let color_buffer = color_buffer_lock.borrow();

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
