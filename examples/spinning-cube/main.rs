extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    scene::{
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

use scene::make_spinning_cube_scene;

mod scene;

#[allow(clippy::too_many_arguments)]
fn update_node(
    _current_world_transform: &Mat4,
    node: &mut SceneNode,
    resources: &SceneResources,
    app: &App,
    _mouse_state: &MouseState,
    _keyboard_state: &KeyboardState,
    _game_controller_state: &GameControllerState,
    shader_context: &mut ShaderContext,
) -> Result<bool, String> {
    let uptime = app.timing_info.uptime_seconds;

    let (node_type, handle) = (node.get_type(), node.get_handle());

    match node_type {
        SceneNodeType::Entity => {
            let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

            let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

            node.get_transform_mut().set_rotation(q);

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
                            x: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                            y: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                            z: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                        } * MAX_POINT_LIGHT_INTENSITY;

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
        _ => Ok(false),
    }
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/spinning-cube".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Scene context

    let (scene_context, shader_context) = make_spinning_cube_scene(camera_aspect_ratio)?;

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

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        // Traverse the scene graph and update its nodes.

        let update_node_rc = Rc::new(update_node);

        scenes[0].update(
            &resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_node_rc),
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
