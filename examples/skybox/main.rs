extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    scene::{
        context::utils::make_cube_scene,
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
    texture::{cubemap::CubeMap, map::TextureMapStorageFormat},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/skybox".to_string(),
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

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let (scene_context, shader_context) = make_cube_scene(camera_aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a skybox to our scene.

        let skybox_node = {
            // Option 1. Cubemap as a set of 6 separate textures.

            let mut skybox_cubemap: CubeMap = CubeMap::new(
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

            // Option 2. Cubemap as one horizontal cross texture.

            // let mut skybox_cubemap: CubeMap = CubeMap::cross(
            //     "examples/skybox/assets/cross/horizontal_cross.png",
            //     TextureMapStorageFormat::RGB24,
            // );

            // Option 3. Cubemap as one vertical cross texture.

            // let mut skybox_cubemap: CubeMap = CubeMap::cross(
            //     "examples/skybox/assets/cross/vertical_cross.png",
            //     TextureMapStorageFormat::RGB24,
            // );

            skybox_cubemap.load(rendering_context).unwrap();

            let skybox_cubemap_handle = resources.cubemap_u8.borrow_mut().insert(skybox_cubemap);

            let skybox = Skybox {
                is_hdr: false,
                radiance: Some(skybox_cubemap_handle),
                irradiance: None,
                specular_prefiltered_environment: None,
            };

            let skybox_handle = resources.skybox.borrow_mut().insert(skybox);

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

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.4;

        let point_light_handle = resources.point_light.borrow_mut().insert(point_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        ))?;

        // Add a spot light to our scene.

        let mut spot_light = SpotLight::new();

        spot_light.look_vector.set_position(Vec3 {
            y: 10.0,
            ..spot_light.look_vector.get_position()
        });

        let spot_light_handle = resources.spot_light.borrow_mut().insert(spot_light);

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

        let resources = (*scene_context_rc.resources).borrow();
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
