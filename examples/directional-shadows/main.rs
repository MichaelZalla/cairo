extern crate sdl2;

use std::{cell::RefCell, f32, f32::consts::PI, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_640_BY_480},
        App, AppWindowInfo,
    },
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::Renderer,
    resource::{arena::Arena, handle::Handle},
    scene::{
        context::SceneContext,
        graph::options::SceneGraphRenderOptions,
        light::directional_light::{DirectionalLight, SHADOW_MAP_CAMERA_COUNT},
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    texture::{map::TextureMap, sample::sample_nearest_f32},
    transform::quaternion::Quaternion,
    vec::{vec2::Vec2, vec3},
};

use scene::make_scene;

mod scene;

static USE_DEMO_CAMERA: bool = false;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/directional-shadows".to_string(),
        canvas_resolution: RESOLUTION_640_BY_480,
        window_resolution: RESOLUTION_640_BY_480,
        ..Default::default()
    };

    // Pipeline framebuffer

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
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();

        make_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader contexts

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

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

                let seconds_since_last_update = app.timing_info.seconds_since_last_update;

                let (node_type, _handle) = (node.get_type(), node.get_handle());

                match node_type {
                    SceneNodeType::Camera => {
                        let mut camera_arena = resources.camera.borrow_mut();
                        let handle = node.get_handle().unwrap();

                        let mut was_handled = false;

                        if let Ok(entry) = camera_arena.get_mut(&handle) {
                            let camera = &mut entry.item;

                            if (USE_DEMO_CAMERA && !camera.is_active)
                                || (!USE_DEMO_CAMERA && camera.is_active)
                            {
                                view_camera_handle
                                    .borrow_mut()
                                    .replace(node.get_handle().unwrap());
                            }

                            if camera.is_active {
                            } else {
                                was_handled = true;

                                if USE_DEMO_CAMERA {
                                    let rotation =
                                        Quaternion::new(vec3::UP, seconds_since_last_update);

                                    camera.look_vector.apply_rotation(rotation);
                                }
                            }
                        }

                        Ok(was_handled)
                    }
                    SceneNodeType::DirectionalLight => {
                        if let Ok(entry) = resources
                            .directional_light
                            .borrow_mut()
                            .get_mut(&node.get_handle().unwrap())
                        {
                            let light = &mut entry.item;

                            let rotate_x = Quaternion::new(vec3::RIGHT, -PI / 4.0);

                            let rotate_y = Quaternion::new(vec3::UP, uptime % f32::consts::TAU);

                            light.set_direction(rotate_x * rotate_y);
                        }

                        Ok(false)
                    }
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
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();
        }

        // Render scene.

        scene.render(
            resources,
            &renderer_rc,
            Some(SceneGraphRenderOptions {
                draw_lights: true,
                draw_cameras: USE_DEMO_CAMERA,
                draw_shadow_map_cameras: USE_DEMO_CAMERA,
                ..Default::default()
            }),
        )?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.end_frame();
        }

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let mut color_buffer = color_buffer_lock.borrow_mut();

                let directional_light_arena = resources.directional_light.borrow();

                let texture_f32_arena = resources.texture_f32.borrow();

                if let Some(handle) = scene
                    .root
                    .find(|node| *node.get_type() == SceneNodeType::DirectionalLight)?
                {
                    if let Ok(entry) = directional_light_arena.get(&handle) {
                        let directional_light = &entry.item;

                        blit_directional_shadow_maps(
                            directional_light,
                            &texture_f32_arena,
                            &mut color_buffer,
                        );
                    }
                }

                color_buffer.copy_to(canvas);

                Ok(())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn blit_directional_shadow_maps(
    light: &DirectionalLight,
    texture_f32_arena: &Arena<TextureMap<f32>>,
    target: &mut Buffer2D,
) {
    let shadow_map_thumbnail_size = target.height / SHADOW_MAP_CAMERA_COUNT as u32;

    let uv_step = 1.0 / shadow_map_thumbnail_size as f32;

    if let Some(handles) = light.shadow_maps.as_ref() {
        for (index, handle) in handles.iter().enumerate() {
            if let Ok(entry) = texture_f32_arena.get(handle) {
                let map = &entry.item;

                for y in 0..shadow_map_thumbnail_size {
                    for x in 0..shadow_map_thumbnail_size {
                        let uv = Vec2 {
                            x: x as f32 * uv_step,
                            y: 1.0 - y as f32 * uv_step,
                            z: 0.0,
                        };

                        let closest_depth_ndc_space = sample_nearest_f32(uv, map);

                        let closest_depth_alpha = closest_depth_ndc_space;

                        let sampled_depth_color =
                            Color::from_vec3(vec3::ONES * closest_depth_alpha * 255.0);

                        target.set(
                            x,
                            y + (index as u32 * shadow_map_thumbnail_size),
                            sampled_depth_color.to_u32(),
                        );
                    }
                }
            }
        }
    }
}
