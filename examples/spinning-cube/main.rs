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
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};
use uuid::Uuid;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/spinning-cube".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    // Render callback

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context =
        Rc::new(make_cube_scene(framebuffer_rc.borrow().width_over_height).unwrap());

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a point light to the scene.

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.4;

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        let mut point_light_node = SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        );

        point_light_node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 5.0,
            z: 0.0,
        });

        scene.root.add_child(point_light_node)?;

        // Add a spot light to the scene.

        let spot_light = SpotLight::new();

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        let mut spot_light_node = SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        );

        spot_light_node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 5.0,
            z: 0.0,
        });

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

    let renderer_rc = RefCell::new(renderer);

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

        let mut update_node = |current_world_transform: &Mat4,
                               node: &mut SceneNode,
                               resources: &SceneResources,
                               app: &App,
                               _mouse_state: &MouseState,
                               _keyboard_state: &KeyboardState,
                               _game_controller_state: &GameControllerState,
                               shader_context: &mut ShaderContext|
         -> Result<bool, String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => {
                    static ENTITY_ROTATION_SPEED: f32 = 0.2;

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

                    Ok(true)
                }
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

                                point_light.position = (Vec4::new(Default::default(), 1.0)
                                    * *current_world_transform
                                    * Mat4::translation(Vec3 {
                                        x: orbital_radius * uptime.sin(),
                                        y: 3.0,
                                        z: orbital_radius * uptime.cos(),
                                    }))
                                .to_vec3();

                                shader_context.get_point_lights_mut().push(*handle);

                                Ok(true)
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
                                    (Vec4::new(Default::default(), 1.0) * *current_world_transform)
                                        .to_vec3(),
                                );

                                spot_light.look_vector.set_target_position(
                                    (Vec4::new(vec3::UP * -1.0, 1.0) * *current_world_transform)
                                        .to_vec3(),
                                );

                                shader_context.get_spot_lights_mut().push(*handle);

                                Ok(true)
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
                _ => Ok(false),
            }
        };

        scenes[0].update(
            &resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            &mut update_node,
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
