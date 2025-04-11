extern crate sdl2;

use std::{
    cell::RefCell,
    f32::consts::{PI, TAU},
    rc::Rc,
};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_320_BY_180},
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
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::{map::TextureMap, sample::sample_nearest_f32},
    transform::quaternion::Quaternion,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

use scene::make_scene;

mod scene;

static USE_DEMO_CAMERA: bool = false;
static DRAW_DIRECTIONAL_SHADOW_MAP_THUMBNAILS: bool = false;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/primitives".to_string(),
        relative_mouse_mode: true,
        window_resolution: RESOLUTION_320_BY_180 * 4.0,
        canvas_resolution: RESOLUTION_320_BY_180,
        ..Default::default()
    };

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();

        let mut skybox_arena = resources.skybox.borrow_mut();
        let mut texture_vec2_arena = resources.texture_vec2.borrow_mut();
        let mut cubemap_vec3_arena = resources.cubemap_vec3.borrow_mut();

        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();

        let mut texture_u8_arena = resources.texture_u8.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();

        make_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut skybox_arena,
            &mut texture_vec2_arena,
            &mut cubemap_vec3_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut point_light_arena,
            &mut spot_light_arena,
            &mut texture_u8_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            rendering_context,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader contexts

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

                let seconds_since_last_update = app.timing_info.seconds_since_last_update;

                let (node_type, handle) = (node.get_type(), node.get_handle());

                match node_type {
                    SceneNodeType::Camera => {
                        let mut camera_arena = resources.camera.borrow_mut();

                        let handle = handle.unwrap();

                        let mut was_handled = false;

                        if let Ok(entry) = camera_arena.get_mut(&handle) {
                            let camera = &mut entry.item;

                            if (USE_DEMO_CAMERA && !camera.is_active)
                                || (!USE_DEMO_CAMERA && camera.is_active)
                            {
                                view_camera_handle.borrow_mut().replace(handle);
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

                            let rotate_y = Quaternion::new(vec3::UP, uptime / 2.0 % TAU);

                            light.set_direction(rotate_x * rotate_y);
                        }

                        Ok(false)
                    }
                    SceneNodeType::PointLight => {
                        let transform = node.get_transform_mut();

                        let position = transform.translation();

                        let y = position.y;

                        static POINT_LIGHT_ORBIT_RADIUS: f32 = 16.0;

                        transform.set_translation(Vec3 {
                            x: uptime.sin() * POINT_LIGHT_ORBIT_RADIUS,
                            y,
                            z: uptime.cos() * POINT_LIGHT_ORBIT_RADIUS,
                        });

                        Ok(false)
                    }
                    SceneNodeType::SpotLight => {
                        let transform = node.get_transform_mut();

                        transform.set_translation(Vec3 {
                            x: 25.0,
                            y: 25.0,
                            z: 0.0,
                        });

                        if let Some(handle) = node.get_handle() {
                            let mut spot_light_arena = resources.spot_light.borrow_mut();

                            if let Ok(entry) = spot_light_arena.get_mut(handle) {
                                let spot_light = &mut entry.item;

                                spot_light.look_vector.set_target(Vec3 {
                                    x: 25.0 + 15.0 * uptime.sin(),
                                    y: 0.0,
                                    z: 0.0 + 15.0 * uptime.cos(),
                                });
                            }
                        }

                        Ok(false)
                    }
                    SceneNodeType::Entity => match handle {
                        Some(handle) => {
                            let mut entity_arena = resources.entity.borrow_mut();

                            match entity_arena.get_mut(handle) {
                                Ok(entry) => {
                                    let entity = &mut entry.item;

                                    if let Ok(entry) = resources.mesh.borrow_mut().get(&entity.mesh)
                                    {
                                        let mesh = &entry.item;

                                        if let Some(object_name) = &mesh.object_name {
                                            if object_name == "plane" {
                                                return Ok(false);
                                            }
                                        }
                                    }

                                    node.get_transform_mut().set_rotation(
                                        Quaternion::new(vec3::UP, uptime % TAU)
                                            * Quaternion::new(vec3::RIGHT, uptime % TAU)
                                            * Quaternion::new(vec3::FORWARD, uptime % TAU),
                                    );

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

        {
            let mut camera_arena = resources.camera.borrow_mut();

            for entry in camera_arena.entries.iter_mut().flatten() {
                let camera = &mut entry.item;

                if camera.is_active {
                    let framebuffer = framebuffer_rc.borrow_mut();

                    if let Some(depth_buffer_rc) = framebuffer.attachments.depth.as_ref() {
                        let mut depth_buffer = depth_buffer_rc.borrow_mut();

                        depth_buffer.set_projection_z_near(camera.get_projection_z_near());
                        depth_buffer.set_projection_z_far(camera.get_projection_z_far());
                    }
                }
            }
        }

        {
            let camera_handle_option = view_camera_handle.borrow();

            if let Some(camera_handle) = camera_handle_option.as_ref() {
                let camera_arena = resources.camera.borrow();

                let mut directional_light_arena = resources.directional_light.borrow_mut();

                if let Some(directional_light_handle) = scene
                    .root
                    .find(|node| *node.get_type() == SceneNodeType::DirectionalLight)?
                {
                    match (
                        directional_light_arena.get_mut(&directional_light_handle),
                        camera_arena.get(camera_handle),
                    ) {
                        (Ok(light_entry), Ok(camera_entry)) => {
                            let view_camera = &camera_entry.item;

                            let directional_light = &mut light_entry.item;

                            directional_light.update_shadow_map_cameras(view_camera);

                            if let Some(shadow_map_cameras) =
                                directional_light.shadow_map_cameras.as_ref()
                            {
                                let transforms = shadow_map_cameras
                                    .iter()
                                    .map(|(far_z, camera)| {
                                        (
                                            *far_z,
                                            camera.get_view_inverse_transform()
                                                * camera.get_projection(),
                                        )
                                    })
                                    .collect();

                                shader_context
                                    .set_directional_light_view_projections(Some(transforms));
                            }
                        }
                        _ => panic!(),
                    }
                }
            }
        }

        let mut renderer = renderer_rc.borrow_mut();

        renderer.options.update(keyboard_state);

        renderer.shader_options.update(keyboard_state);

        let camera_handle = scene
            .root
            .find(|node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
            .unwrap();

        let camera_arena = resources.camera.borrow();

        if let Ok(entry) = camera_arena.get(&camera_handle) {
            let camera = &entry.item;

            renderer.set_clipping_frustum(*camera.get_frustum());
        }

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

                if DRAW_DIRECTIONAL_SHADOW_MAP_THUMBNAILS {
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
