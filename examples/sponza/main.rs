extern crate sdl2;

use std::{cell::RefCell, f32::consts::TAU, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_640_BY_480},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

pub mod scene;

use scene::make_sponza_scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/sponza".to_string(),
        window_resolution: RESOLUTION_640_BY_480 * 2.0,
        canvas_resolution: RESOLUTION_640_BY_480,
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

    framebuffer.complete(0.3, 10_000.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();

        let mut skybox_arena = resources.skybox.borrow_mut();
        let mut texture_vec2_arena = resources.texture_vec2.borrow_mut();
        let mut cubemap_vec3_arena = resources.cubemap_vec3.borrow_mut();

        make_sponza_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut texture_u8_arena,
            rendering_context,
            &mut point_light_arena,
            &mut spot_light_arena,
            &mut skybox_arena,
            &mut texture_vec2_arena,
            &mut cubemap_vec3_arena,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.albedo_mapping_active = true;
    renderer.shader_options.specular_exponent_mapping_active = true;
    renderer.shader_options.normal_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    #[allow(clippy::too_many_arguments)]
    let update_scene_graph_node = |_current_world_transform: &Mat4,
                                   node: &mut SceneNode,
                                   resources: &SceneResources,
                                   app: &App,
                                   _mouse_state: &MouseState,
                                   _keyboard_state: &KeyboardState,
                                   _game_controller_state: &GameControllerState,
                                   _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let (node_type, handle) = (node.get_type(), node.get_handle());

        let uptime = app.timing_info.uptime_seconds;

        match node_type {
            SceneNodeType::DirectionalLight => match handle {
                Some(handle) => {
                    let mut arena = resources.directional_light.borrow_mut();

                    match arena.get_mut(handle) {
                        Ok(entry) => {
                            let light = &mut entry.item;

                            light.set_direction(Quaternion::new(vec3::UP, uptime.rem_euclid(TAU)));

                            Ok(false)
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
            SceneNodeType::PointLight => {
                let transform = node.get_transform_mut();

                let position = Vec3 {
                    x: 800.0 * (uptime / 20.0).sin(),
                    y: 200.0,
                    z: -75.0,
                };

                transform.set_translation(position);

                Ok(false)
            }
            SceneNodeType::SpotLight => {
                let transform = node.get_transform_mut();

                let position = Vec3 {
                    x: -800.0 * uptime.sin(),
                    y: 500.0,
                    z: 0.0,
                };

                transform.set_translation(position);

                Ok(false)
            }
            _ => Ok(false),
        }
    };

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let mut shader_context = shader_context_rc.borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        let update_scene_graph_node_rc = Rc::new(update_scene_graph_node);

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_scene_graph_node_rc),
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
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        // Render scene.

        scene.render(resources, &renderer_rc, None)?;

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
    };

    app.run(&mut update, &render)?;

    Ok(())
}
