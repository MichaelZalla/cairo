extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_640_BY_320},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::culling::FaceCullingReject,
    scene::{
        context::SceneContext,
        light::point_light::{
            POINT_LIGHT_SHADOW_CAMERA_FAR, POINT_LIGHT_SHADOW_CAMERA_NEAR,
            POINT_LIGHT_SHADOW_MAP_SIZE,
        },
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
        shadow_shaders::point_shadows::{
            PointShadowMapFragmentShader, PointShadowMapGeometryShader, PointShadowMapVertexShader,
        },
    },
    software_renderer::SoftwareRenderer,
    vec::vec3::Vec3,
};

use crate::{scene::make_scene, shadow::update_point_light_shadow_maps};

pub mod scene;
pub mod shadow;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/point-shadows".to_string(),
        canvas_resolution: RESOLUTION_640_BY_320,
        window_resolution: RESOLUTION_640_BY_320,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Point shadow map framebuffer

    let mut point_shadow_map_framebuffer =
        Framebuffer::new(POINT_LIGHT_SHADOW_MAP_SIZE, POINT_LIGHT_SHADOW_MAP_SIZE);

    point_shadow_map_framebuffer.complete(
        POINT_LIGHT_SHADOW_CAMERA_NEAR,
        POINT_LIGHT_SHADOW_CAMERA_FAR,
    );

    let point_shadow_map_framebuffer_rc = Rc::new(RefCell::new(point_shadow_map_framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = scene_context.resources.borrow();

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut cubemap_f32_arena = resources.cubemap_f32.borrow_mut();

        make_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut point_light_arena,
            &mut cubemap_f32_arena,
            point_shadow_map_framebuffer_rc.clone(),
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    let point_shadow_map_shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    // Renderer

    let renderer_rc = {
        let mut renderer = SoftwareRenderer::new(
            shader_context_rc.clone(),
            scene_context.resources.clone(),
            DEFAULT_VERTEX_SHADER,
            DEFAULT_FRAGMENT_SHADER,
            Default::default(),
        );

        renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

        RefCell::new(renderer)
    };

    let point_shadow_map_renderer_rc = {
        let mut renderer = SoftwareRenderer::new(
            point_shadow_map_shader_context_rc.clone(),
            scene_context.resources.clone(),
            PointShadowMapVertexShader,
            PointShadowMapFragmentShader,
            Default::default(),
        );

        renderer.set_geometry_shader(PointShadowMapGeometryShader);

        renderer
            .options
            .rasterizer_options
            .face_culling_strategy
            .reject = FaceCullingReject::Frontfaces;

        renderer.bind_framebuffer(Some(point_shadow_map_framebuffer_rc.clone()));

        RefCell::new(renderer)
    };

    // App update and render callbacks

    let update_node = |_current_world_transform: &Mat4,
                       node: &mut SceneNode,
                       _resources: &SceneResources,
                       app: &App,
                       _mouse_state: &MouseState,
                       _keyboard_state: &KeyboardState,
                       _game_controller_state: &GameControllerState,
                       _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let uptime = app.timing_info.uptime_seconds;

        match node.get_type() {
            SceneNodeType::PointLight => {
                let transform = node.get_transform_mut();

                let position = transform.translation();

                let y = position.y;

                let factor = (y - 5.0) / 2.0;

                transform.set_translation(Vec3 {
                    x: 10.0 * (uptime + PI / 2.0 * factor).sin(),
                    y,
                    z: 10.0 * (uptime + PI / 2.0 * factor).cos(),
                });

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
        let resources = scene_context.resources.borrow();

        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.clear_lights();

        // Traverse the scene graph and update its nodes.

        let update_node_rc = Rc::new(update_node);

        let scene = &mut scenes[0];

        scene.update(
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
        // Render point shadow map.

        update_point_light_shadow_maps(
            &scene_context,
            &point_shadow_map_renderer_rc,
            &point_shadow_map_shader_context_rc,
            &point_shadow_map_framebuffer_rc,
        )?;

        // Render scene.

        let resources = scene_context.resources.borrow();

        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.bind_framebuffer(Some(framebuffer_rc.clone()));
        }

        match scene.render(&resources, &renderer_rc, None) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let color_buffer = color_buffer_lock.borrow_mut();

                        // for (index, entry) in resources
                        //     .point_light
                        //     .borrow()
                        //     .entries
                        //     .iter()
                        //     .flatten()
                        //     .enumerate()
                        // {
                        //     if index != 0 {
                        //         continue;
                        //     }

                        //     let light = &entry.item;

                        //     match &light.shadow_map {
                        //         Some(shadow_map) => {
                        //             blit_shadow_map_horizontal_cross(shadow_map, &mut color_buffer)
                        //         }
                        //         None => (),
                        //     }
                        // }

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
