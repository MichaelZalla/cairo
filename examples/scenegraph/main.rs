extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use uuid::Uuid;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
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
        skybox::Skybox,
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::{
        cubemap::CubeMap,
        map::{TextureMap, TextureMapStorageFormat},
    },
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/scenegraph".to_string(),
        relative_mouse_mode: true,
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

    static RED_CUBE_ORIGINAL_UNIFORM_SCALE: f32 = 1.0;
    static GREEN_CUBE_ORIGINAL_UNIFORM_SCALE: f32 = 2.0 / 3.0;
    static BLUE_CUBE_ORIGINAL_UNIFORM_SCALE: f32 =
        GREEN_CUBE_ORIGINAL_UNIFORM_SCALE * GREEN_CUBE_ORIGINAL_UNIFORM_SCALE;

    let (scene_context, shader_context) =
        make_empty_scene(framebuffer_rc.borrow().width_over_height)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a textured ground plane to our scene.

        {
            let mut materials = resources.material.borrow_mut();

            let checkerboard_material = {
                let mut material = Material::new("checkerboard".to_string());

                let mut checkerboard_albedo_map = TextureMap::new(
                    "./assets/textures/checkerboard.jpg",
                    TextureMapStorageFormat::Index8(0),
                );

                checkerboard_albedo_map.load(rendering_context)?;

                let checkerboard_albedo_map_handle = resources
                    .texture_u8
                    .borrow_mut()
                    .insert(Uuid::new_v4(), checkerboard_albedo_map);

                material.albedo_map = Some(checkerboard_albedo_map_handle);

                material
            };

            materials.insert(checkerboard_material);
        }

        let mut plane_entity_node = {
            let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);

            plane_mesh.material_name = Some("checkerboard".to_string());

            let plane_mesh_handle = resources
                .mesh
                .borrow_mut()
                .insert(Uuid::new_v4(), plane_mesh);

            let plane_entity = Entity::new(plane_mesh_handle, Some("checkerboard".to_string()));

            let plane_entity_handle = resources
                .entity
                .borrow_mut()
                .insert(Uuid::new_v4(), plane_entity);

            SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(plane_entity_handle),
            )
        };

        // Add some cubes to our scene.

        let mut red_cube_material = Material::new("red".to_string());
        red_cube_material.albedo = color::RED.to_vec3() / 255.0;

        let mut green_cube_material = Material::new("green".to_string());
        green_cube_material.albedo = color::GREEN.to_vec3() / 255.0;

        let mut blue_cube_material = Material::new("blue".to_string());
        blue_cube_material.albedo = color::BLUE.to_vec3() / 255.0;

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(red_cube_material);
            materials.insert(green_cube_material);
            materials.insert(blue_cube_material);
        }

        // Blue cube (1x1)

        let cube_mesh = mesh::primitive::cube::generate(3.0, 3.0, 3.0);

        let blue_cube_entity_node = {
            let mut mesh = cube_mesh.clone();

            mesh.object_name = Some("blue_cube".to_string());
            mesh.material_name = Some("blue".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("blue".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            let mut scale = *(node.get_transform().scale());
            let mut translate = *(node.get_transform().translation());

            scale *= 2.0 / 3.0;
            translate.y = 4.0;

            node.get_transform_mut().set_translation(translate);
            node.get_transform_mut().set_scale(scale);

            node
        };

        // Green cube (2x2)

        let mut green_cube_entity_node = {
            let mut mesh = cube_mesh.clone();

            mesh.object_name = Some("green_cube".to_string());
            mesh.material_name = Some("green".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("green".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            let mut scale = *(node.get_transform().scale());
            let mut translate = *(node.get_transform().translation());

            scale *= 2.0 / 3.0;
            translate.y = 4.0;

            node.get_transform_mut().set_translation(translate);
            node.get_transform_mut().set_scale(scale);

            node
        };

        // Red cube (3x3)

        let mut red_cube_entity_node = {
            let mut mesh = cube_mesh.clone();

            mesh.object_name = Some("red_cube".to_string());
            mesh.material_name = Some("red".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("red".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            let mut translate = *(node.get_transform().translation());

            translate.y = 3.0;

            node.get_transform_mut().set_translation(translate);

            node
        };

        // Add a skybox to our scene.

        let skybox_node = {
            let mut skybox_cubemap = CubeMap::new(
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

            skybox_cubemap.load(rendering_context).unwrap();

            let skybox_cubemap_handle = resources
                .cubemap_u8
                .borrow_mut()
                .insert(Uuid::new_v4(), skybox_cubemap);

            let skybox = Skybox {
                is_hdr: false,
                radiance: Some(skybox_cubemap_handle),
                irradiance: None,
                specular_prefiltered_environment: None,
            };

            let skybox_handle = resources.skybox.borrow_mut().insert(Uuid::new_v4(), skybox);

            SceneNode::new(
                SceneNodeType::Skybox,
                Default::default(),
                Some(skybox_handle),
            )
        };

        for node in scene.root.children_mut().as_mut().unwrap() {
            if *node.get_type() == SceneNodeType::Environment {
                node.add_child(skybox_node)?;

                break;
            }
        }

        // Add a point light to our scene.

        let point_light = PointLight::new();

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        let point_light_node = SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        );

        plane_entity_node.add_child(point_light_node)?;

        // Add a spot light to our scene.

        let mut spot_light = SpotLight::new();

        spot_light
            .look_vector
            .set_target_position(Default::default());

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        let mut spot_light_node = SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        );

        // Add a spot light as a child of the green cube.

        spot_light_node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        });

        green_cube_entity_node.add_child(spot_light_node)?;

        // Add the blue cube as a child of the green cube.

        green_cube_entity_node.add_child(blue_cube_entity_node)?;

        // Add the green cube as a child of the red cube.

        red_cube_entity_node.add_child(green_cube_entity_node)?;

        // Add the red cube as a child of the ground plane.

        plane_entity_node.add_child(red_cube_entity_node)?;

        scene.root.add_child(plane_entity_node)?;

        // Adjust our scene's default camera.

        if let Some(camera_handle) = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
        {
            let mut camera_arena = resources.camera.borrow_mut();

            if let Ok(entry) = camera_arena.get_mut(&camera_handle) {
                let camera = &mut entry.item;

                camera.look_vector.set_position(Vec3 {
                    x: 0.0,
                    y: 24.0,
                    z: -32.0,
                });

                camera.look_vector.set_target_position(Default::default());
            }
        }
    }

    // ShaderContext

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;

        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

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

                                let mut scale = *node.get_transform().scale();
                                let mut translation = *node.get_transform().translation();

                                let transform = node.get_transform_mut();

                                if let Ok(entry) = mesh_arena.get(&entity.mesh) {
                                    let mesh = &entry.item;

                                    if let Some(object_name) = &mesh.object_name {
                                        match object_name.as_str() {
                                            "plane" => {
                                                let qx = Quaternion::new(
                                                    vec3::RIGHT,
                                                    PI / 12.0 * (uptime).cos(),
                                                );

                                                let qz = Quaternion::new(
                                                    vec3::FORWARD,
                                                    PI / 12.0 * (uptime).sin(),
                                                );

                                                transform.set_rotation(qx * qz);
                                            }
                                            "red_cube" => {
                                                let qy = Quaternion::new(
                                                    vec3::UP,
                                                    (uptime / 2.0) % 2.0 * PI,
                                                );

                                                transform.set_rotation(qy);

                                                let uniform_scale = RED_CUBE_ORIGINAL_UNIFORM_SCALE
                                                    + (uptime * 2.0).sin()
                                                        * RED_CUBE_ORIGINAL_UNIFORM_SCALE
                                                        * 0.25;

                                                scale.x = uniform_scale;
                                                scale.y = uniform_scale;
                                                scale.z = uniform_scale;
                                            }
                                            "green_cube" => {
                                                let qy = Quaternion::new(
                                                    vec3::UP,
                                                    (-uptime / 4.0) % 2.0 * PI,
                                                );

                                                transform.set_rotation(qy);

                                                let uniform_scale =
                                                    GREEN_CUBE_ORIGINAL_UNIFORM_SCALE
                                                        + (-uptime * 2.0).sin()
                                                            * GREEN_CUBE_ORIGINAL_UNIFORM_SCALE
                                                            * 0.25;

                                                scale.x = uniform_scale;
                                                scale.y = uniform_scale;
                                                scale.z = uniform_scale;

                                                translation.x = (uptime).sin() * 1.0;
                                                translation.z = (uptime).cos() * 1.0;
                                            }
                                            "blue_cube" => {
                                                let qy = Quaternion::new(
                                                    vec3::UP,
                                                    (uptime / 8.0) % 2.0 * PI,
                                                );

                                                transform.set_rotation(qy);

                                                let uniform_scale = BLUE_CUBE_ORIGINAL_UNIFORM_SCALE
                                                    + (uptime * 2.0).sin()
                                                        * BLUE_CUBE_ORIGINAL_UNIFORM_SCALE
                                                        * 0.25;

                                                scale.x = uniform_scale;
                                                scale.y = uniform_scale;
                                                scale.z = uniform_scale;

                                                translation.x = (-uptime).sin() * 1.0;
                                                translation.z = (-uptime).cos() * 1.0;
                                            }
                                            _ => (),
                                        }
                                    }
                                }

                                transform.set_scale(scale);
                                transform.set_translation(translation);

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
                                static MAX_POINT_LIGHT_INTENSITY: f32 = 1.0;

                                point_light.intensities = Vec3 {
                                    x: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0
                                        + 0.5,
                                    y: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).cos() / 2.0
                                        + 0.5,
                                    z: -(uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0
                                        + 0.5,
                                } * MAX_POINT_LIGHT_INTENSITY;

                                let orbital_radius: f32 = 10.0;

                                point_light.position = (Vec4::new(Default::default(), 1.0)
                                    * Mat4::translation(Vec3 {
                                        x: orbital_radius * uptime.sin(),
                                        y: 1.0,
                                        z: orbital_radius * uptime.cos(),
                                    })
                                    * current_world_transform)
                                    .to_vec3();

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

        renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

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

                        Ok(color_buffer.get_all().clone())
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
