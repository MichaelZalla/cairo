extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use uuid::Uuid;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh,
    resource::handle::Handle,
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
    texture::map::{TextureMap, TextureMapStorageFormat},
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/diplacement-map".to_string(),
        vertical_sync: true,
        relative_mouse_mode: true,
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

        // Bricks material

        let mut brick_material = Material::new("brick".to_string());

        brick_material.albedo_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/bricks2.jpg",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        brick_material.normal_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/bricks2_normal.jpg",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        brick_material.displacement_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/bricks2_disp.jpg",
                TextureMapStorageFormat::Index8(0),
            ),
        ));

        brick_material.displacement_scale = 0.05;

        brick_material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

        // Box material

        let mut box_material = Material::new("box".to_string());

        box_material.albedo_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/wood.png",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        box_material.normal_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/toy_box_normal.png",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        box_material.displacement_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/displacement-map/assets/toy_box_disp.png",
                TextureMapStorageFormat::Index8(0),
            ),
        ));

        box_material.displacement_scale = 0.05;

        box_material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

        // Collect materials

        let brick_material_handle: Handle;
        let box_material_handle: Handle;

        {
            let mut materials = resources.material.borrow_mut();

            brick_material_handle = materials.insert(Uuid::new_v4(), brick_material);
            box_material_handle = materials.insert(Uuid::new_v4(), box_material);
        }

        // Add a brick wall to our scene.

        let brick_wall_mesh = mesh::primitive::cube::generate(1.5, 1.5, 1.5);

        let brick_wall_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_mesh);

        let brick_wall_entity = Entity::new(brick_wall_mesh_handle, Some(brick_material_handle));

        let brick_wall_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_entity);

        let mut brick_wall_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(brick_wall_entity_handle),
        );

        brick_wall_entity_node
            .get_transform_mut()
            .set_translation(Vec3 {
                x: -2.0,
                y: 0.0,
                z: 4.0,
            });

        scene.root.add_child(brick_wall_entity_node)?;

        // Add a wooden box to our scene.

        let wooden_box_mesh = mesh::primitive::cube::generate(1.5, 1.5, 1.5);

        let wooden_box_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), wooden_box_mesh);

        let wooden_box_entity = Entity::new(wooden_box_mesh_handle, Some(box_material_handle));

        let wooden_box_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), wooden_box_entity);

        let mut wooden_box_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(wooden_box_entity_handle),
        );

        wooden_box_entity_node
            .get_transform_mut()
            .set_translation(Vec3 {
                x: 2.0,
                y: 0.0,
                z: 4.0,
            });

        scene.root.add_child(wooden_box_entity_node)?;

        // Add a point light to our scene.

        let mut point_light = PointLight::new();

        point_light.position.y = 0.0;
        point_light.position.z = -4.0;

        point_light.intensities = Vec3::ones() * 10.0;

        point_light.constant_attenuation = 1.0;
        point_light.linear_attenuation = 0.35;
        point_light.quadratic_attenuation = 0.44;

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        ))?;

        // Add a spot light to our scene.

        let spot_light = SpotLight::new();

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        ))?;
    }

    let scene_context_rc = Rc::new(scene_context);

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context_rc.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.normal_mapping_active = true;
    renderer.shader_options.displacement_mapping_active = true;

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
                SceneNodeType::Entity => {
                    let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                    let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                    node.get_transform_mut().set_rotation(q);

                    Ok(())
                }
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        let mut point_light_arena = resources.point_light.borrow_mut();

                        match point_light_arena.get_mut(handle) {
                            Ok(entry) => {
                                let point_light = &mut entry.item;

                                let orbital_radius: f32 = 6.0;

                                point_light.position = Vec3 {
                                    x: 4.0 + orbital_radius * uptime.sin(),
                                    y: orbital_radius * uptime.cos(),
                                    z: -4.0,
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
