extern crate sdl2;

use std::{cell::RefCell, f32::consts::TAU, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::Renderer,
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3,
};

use scene::make_scene;

mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/skybox".to_string(),
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 100.0);

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
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();
        let mut cubemap_u8_arena = resources.cubemap_u8.borrow_mut();
        let mut skybox_arena = resources.skybox.borrow_mut();

        make_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut point_light_arena,
            &mut spot_light_arena,
            rendering_context,
            &mut cubemap_u8_arena,
            &mut skybox_arena,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let update_node = |_current_world_transform: &Mat4,
                       node: &mut SceneNode,
                       resources: &SceneResources,
                       app: &App,
                       _mouse_state: &MouseState,
                       _keyboard_state: &KeyboardState,
                       _game_controller_state: &GameControllerState,
                       _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let uptime = app.timing_info.uptime_seconds;

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
                                        return Ok(false);
                                    }
                                }
                            }

                            let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                            let q = Quaternion::new(rotation_axis, uptime % TAU);

                            node.get_transform_mut().set_rotation(q);

                            Ok(false)
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
            _ => Ok(false),
        }
    };

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let mut shader_context = (*shader_context_rc).borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        let update_node_rc = Rc::new(update_node);

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_node_rc),
        )?;
        let mut renderer = renderer_rc.borrow_mut();

        renderer.update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();
        }

        // Render scene.

        scene.render(resources, &renderer_rc, None)?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.end_frame();
        }

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
