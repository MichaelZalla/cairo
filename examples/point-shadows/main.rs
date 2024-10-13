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
        light::{
            POINT_LIGHT_SHADOW_CAMERA_FAR, POINT_LIGHT_SHADOW_CAMERA_NEAR,
            POINT_LIGHT_SHADOW_MAP_SIZE,
        },
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
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
};

use crate::{scene::make_cubes_scene, shadow::update_point_light_shadow_maps};

pub mod scene;
pub mod shadow;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/point-shadows".to_string(),
        canvas_resolution: RESOLUTION_640_BY_320,
        window_resolution: RESOLUTION_640_BY_320,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

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

    let (scene_context, shader_context) = make_cubes_scene(
        framebuffer_rc.borrow().width_over_height,
        point_shadow_map_framebuffer_rc.clone(),
    )?;

    let scene_context_rc = Rc::new(scene_context);

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    let point_shadow_map_shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    // Renderer

    let renderer_rc = {
        let mut renderer = SoftwareRenderer::new(
            shader_context_rc.clone(),
            scene_context_rc.resources.clone(),
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
            scene_context_rc.resources.clone(),
            PointShadowMapVertexShader,
            PointShadowMapFragmentShader,
            Default::default(),
        );

        renderer.set_geometry_shader(PointShadowMapGeometryShader);

        renderer.options.face_culling_strategy.reject = FaceCullingReject::Frontfaces;

        renderer.bind_framebuffer(Some(point_shadow_map_framebuffer_rc.clone()));

        RefCell::new(renderer)
    };

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = scene_context_rc.resources.borrow_mut();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        let uptime = app.timing_info.uptime_seconds;

        // Traverse the scene graph and update its nodes.

        let mut update_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            match node.get_type() {
                SceneNodeType::PointLight => {
                    if let Ok(entry) = resources
                        .point_light
                        .borrow_mut()
                        .get_mut(&node.get_handle().unwrap())
                    {
                        let point_light = &mut entry.item;

                        let index = (point_light.position.y - 5.0) / 2.0;

                        point_light.position.x = 10.0 * (uptime + PI / 2.0 * index).sin();
                        point_light.position.z = 10.0 * (uptime + PI / 2.0 * index).cos();
                    }

                    node.update(
                        &current_world_transform,
                        &resources,
                        app,
                        mouse_state,
                        keyboard_state,
                        game_controller_state,
                        &mut shader_context,
                    )
                }
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

        renderer
            .shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
        // Render point shadow map.

        update_point_light_shadow_maps(
            &scene_context_rc,
            &point_shadow_map_renderer_rc,
            &point_shadow_map_shader_context_rc,
            point_shadow_map_framebuffer_rc.clone(),
        );

        // Render scene.

        let resources = scene_context_rc.resources.borrow();
        let mut scenes = scene_context_rc.scenes.borrow_mut();
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
