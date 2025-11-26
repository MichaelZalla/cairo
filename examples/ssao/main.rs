extern crate sdl2;

use std::{cell::RefCell, f32, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_480_BY_270},
        App, AppWindowInfo,
    },
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::{options::RenderPassFlags, Renderer},
    resource::handle::Handle,
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    texture::sample::sample_nearest_f32,
    transform::quaternion::Quaternion,
    vec::{vec2::Vec2, vec3},
};

use scene::make_ssao_scene;

mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ssao".to_string(),
        window_resolution: RESOLUTION_480_BY_270 * 2.0,
        canvas_resolution: RESOLUTION_480_BY_270,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

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

        make_ssao_scene(
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

    renderer.options.render_pass_flags |=
        RenderPassFlags::DEFERRED_LIGHTING | RenderPassFlags::SSAO | RenderPassFlags::SSAO_BLUR;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let view_camera_handle: &'static RefCell<Option<Handle>> =
        Box::leak(Box::new(RefCell::new(Default::default())));

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

        let update_node_rc = Rc::new(
            |_current_world_transform: &Mat4,
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
                    SceneNodeType::Camera => match handle {
                        Some(handle) => {
                            view_camera_handle.borrow_mut().replace(*handle);

                            Ok(false)
                        }
                        None => panic!("Encountered a `Camera` node with no resource handle!"),
                    },
                    SceneNodeType::Entity => match handle {
                        Some(handle) => {
                            let entity_arena = resources.entity.borrow();

                            if let Ok(entry) = entity_arena.get(handle) {
                                let entity = &entry.item;

                                let mesh_arena = resources.mesh.borrow();

                                if let Ok(entry) = mesh_arena.get(&entity.mesh) {
                                    let mesh = &entry.item;

                                    if let Some(object_name) = mesh.object_name.as_ref() {
                                        if object_name == "ground_plane" {
                                            return Ok(false);
                                        }
                                    }

                                    let transform = node.get_transform_mut();

                                    let rotate_y =
                                        Quaternion::new(vec3::UP, uptime / 5.0 % f32::consts::TAU);

                                    transform.set_rotation(rotate_y);
                                }
                            }

                            Ok(false)
                        }
                        None => panic!("Encountered an `Entity` node with no resource handle!"),
                    },
                    _ => Ok(false),
                }
            },
        );

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
        // Render scene.

        let resources = &scene_context.resources;

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

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
                let mut color_buffer = color_buffer_lock.borrow_mut();

                draw_ambient_occlusion_buffer(&renderer_rc, &mut color_buffer);

                color_buffer.copy_to(canvas);

                Ok(())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn draw_ambient_occlusion_buffer(
    renderer_rc: &RefCell<SoftwareRenderer>,
    color_buffer: &mut Buffer2D,
) {
    static SSAO_BUFFER_THUMBNAIL_WIDTH: u32 = (RESOLUTION_480_BY_270.width as f32 * 0.33) as u32;

    let renderer = renderer_rc.borrow();

    let thumbnail_height =
        (SSAO_BUFFER_THUMBNAIL_WIDTH as f32 / color_buffer.width_over_height) as u32;

    let (offset_x, offset_y) = (0, color_buffer.height - 1 - thumbnail_height);

    let uv_step_x = 1.0 / SSAO_BUFFER_THUMBNAIL_WIDTH as f32;
    let uv_step_y = 1.0 / thumbnail_height as f32;

    if let Some(occlusion_map) = renderer.ssao_buffer.as_ref() {
        for y in 0..thumbnail_height {
            for x in 0..SSAO_BUFFER_THUMBNAIL_WIDTH {
                let uv = Vec2 {
                    x: x as f32 * uv_step_x,
                    y: 1.0 - y as f32 * uv_step_y,
                    z: 0.0,
                };

                let occlusion = sample_nearest_f32(uv, occlusion_map);

                let occlusion_color = Color::from_vec3(vec3::ONES * (1.0 - occlusion) * 255.0);

                color_buffer.set(x + offset_x, y + offset_y, occlusion_color.to_u32());
            }
        }
    }
}
