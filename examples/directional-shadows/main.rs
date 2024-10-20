extern crate sdl2;

use std::{cell::RefCell, f32, f32::consts::PI, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::culling::FaceCullingReject,
    resource::handle::Handle,
    scene::{
        context::SceneContext,
        graph::SceneGraphRenderOptions,
        light::directional_light::{SHADOW_MAP_CAMERA_COUNT, SHADOW_MAP_CAMERA_NEAR},
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
        directional_shadow_map_fragment_shader::DirectionalShadowMapFragmentShader,
        directional_shadow_map_geometry_shader::DirectionalShadowMapGeometryShader,
        directional_shadow_map_vertex_shader::DirectionalShadowMapVertexShader,
    },
    software_renderer::SoftwareRenderer,
    texture::{map::TextureMap, sample::sample_nearest_f32},
    transform::quaternion::Quaternion,
    vec::{vec2::Vec2, vec3},
};

use scene::make_scene;
use shadow::render_shadow_maps;

mod scene;
mod shadow;

static SHADOW_MAP_SIZE: u32 = 768;

static USE_DEMO_CAMERA: bool = true;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/directional-shadows".to_string(),
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

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Directional shadow map framebuffer

    let mut directional_shadow_map_framebuffer = Framebuffer::new(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE);

    directional_shadow_map_framebuffer.complete(SHADOW_MAP_CAMERA_NEAR, 100.0);

    let directional_shadow_map_framebuffer_rc =
        Rc::new(RefCell::new(directional_shadow_map_framebuffer));

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

    let directional_shadow_map_shader_context_rc = Rc::new(RefCell::new(ShaderContext::default()));

    {
        let resources = &scene_context.resources;

        let mut shader_context = directional_shadow_map_shader_context_rc.borrow_mut();

        let directional_light_arena = resources.directional_light.borrow();

        let entry = directional_light_arena
            .entries
            .iter()
            .flatten()
            .next()
            .unwrap();

        shader_context.set_directional_light(Some(Handle {
            index: 0,
            uuid: entry.uuid,
        }));
    }

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

    // Directional shadow map renderer

    let mut directional_shadow_map_renderer = SoftwareRenderer::new(
        directional_shadow_map_shader_context_rc.clone(),
        scene_context.resources.clone(),
        DirectionalShadowMapVertexShader,
        DirectionalShadowMapFragmentShader,
        Default::default(),
    );

    directional_shadow_map_renderer.set_geometry_shader(DirectionalShadowMapGeometryShader);

    directional_shadow_map_renderer
        .options
        .rasterizer_options
        .face_culling_strategy
        .reject = FaceCullingReject::None;

    directional_shadow_map_renderer
        .bind_framebuffer(Some(directional_shadow_map_framebuffer_rc.clone()));

    let directional_shadow_map_renderer_rc = RefCell::new(directional_shadow_map_renderer);

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

                            let rotate_y =
                                Quaternion::new(vec3::UP, uptime / 2.0 % f32::consts::TAU);

                            light.set_direction(rotate_x * rotate_y);
                        }

                        Ok(false)
                    }
                    _ => Ok(false),
                }
            },
        );

        scene.update(
            &resources,
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

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        // Render directional shadow map.

        let directional_light_shadow_maps: [TextureMap<f32>; SHADOW_MAP_CAMERA_COUNT] = {
            let resources = &scene_context.resources;

            let directional_light_arena = resources.directional_light.borrow();

            let entry = directional_light_arena
                .entries
                .iter()
                .flatten()
                .next()
                .unwrap();

            let light = &entry.item;

            render_shadow_maps(
                light,
                &scene_context,
                &directional_shadow_map_shader_context_rc,
                &directional_shadow_map_framebuffer_rc,
                &directional_shadow_map_renderer_rc,
            )
            .unwrap()
        };

        {
            let mut shader_context = shader_context_rc.borrow_mut();

            shader_context.set_directional_light_shadow_maps(Some(directional_light_shadow_maps));
        }

        // Render scene.

        let resources = &scene_context.resources;

        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        match scene.render(
            resources,
            &renderer_rc,
            Some(SceneGraphRenderOptions {
                draw_lights: true,
                draw_cameras: USE_DEMO_CAMERA,
                draw_shadow_map_cameras: USE_DEMO_CAMERA,
                camera: None,
            }),
        ) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let shader_context = shader_context_rc.borrow();

                        let mut color_buffer = color_buffer_lock.borrow_mut();

                        static SHADOW_MAP_THUMBNAIL_SIZE: u32 = 175;

                        if let Some(shadow_maps) =
                            shader_context.directional_light_shadow_maps.as_ref()
                        {
                            for (shadow_map_index, shadow_map) in shadow_maps.iter().enumerate() {
                                for y in 0..SHADOW_MAP_THUMBNAIL_SIZE {
                                    for x in 0..SHADOW_MAP_THUMBNAIL_SIZE {
                                        static UV_STEP: f32 =
                                            1.0 / SHADOW_MAP_THUMBNAIL_SIZE as f32;

                                        let uv = Vec2 {
                                            x: x as f32 * UV_STEP,
                                            y: y as f32 * UV_STEP,
                                            z: 0.0,
                                        };

                                        let sampled_depth =
                                            sample_nearest_f32(uv, shadow_map) * 100.0;

                                        let sampled_depth_color =
                                            Color::from_vec3(vec3::ONES * sampled_depth * 255.0);

                                        color_buffer.set(
                                            x,
                                            y + (shadow_map_index as u32
                                                * SHADOW_MAP_THUMBNAIL_SIZE),
                                            sampled_depth_color.to_u32(),
                                        );
                                    }
                                }
                            }
                        }

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
