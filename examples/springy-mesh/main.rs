extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use cairo::{
    app::{
        resolution::{self, Resolution},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::Renderer,
    scene::{
        context::{utils::make_empty_scene, SceneContext},
        empty::EmptyDisplayKind,
    },
    software_renderer::SoftwareRenderer,
    vec::vec3,
};

use simulation::make_simulation;

mod integration;
mod plane_collider;
mod simulation;
mod springy_mesh;
mod strut;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/springy-mesh".to_string(),
        window_resolution: resolution::RESOLUTION_1280_BY_720,
        canvas_resolution: resolution::RESOLUTION_1280_BY_720,
        ..Default::default()
    };

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 1000.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();

        let (scene, scene_context) = make_empty_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
        )?;

        let active_camera_entry = camera_arena
            .entries
            .iter_mut()
            .flatten()
            .find(|e| e.item.is_active)
            .unwrap();

        active_camera_entry.item.set_projection_z_far(1_000.0);

        (scene, scene_context)
    };

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Simulation

    let simulation = make_simulation();

    let simulation_rc = RefCell::new(simulation);

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        {
            let simulation = simulation_rc.borrow();

            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();

            renderer.render_ground_plane(30);

            for mesh in &simulation.meshes {
                // Visualize points.

                for point in &mesh.points {
                    let transform =
                        Mat4::scale(vec3::ONES * 0.1) * Mat4::translation(point.position);

                    renderer.render_empty(
                        &transform,
                        EmptyDisplayKind::Sphere(12),
                        Some(color::ORANGE),
                    );
                }

                // Visualize struts.

                for strut in &mesh.struts {
                    let (start, end) = (
                        &mesh.points[strut.points.0].position,
                        &mesh.points[strut.points.1].position,
                    );

                    renderer.render_line(*start, *end, strut.color);
                }
            }

            for collider in &simulation.static_plane_colliders {
                // Visualize static plane colliders.

                let mut right = collider.plane.normal.cross(vec3::UP);

                if right.mag() < f32::EPSILON {
                    right = collider.plane.normal.cross(vec3::FORWARD);
                }

                let up = collider.plane.normal.cross(-right);

                // Normal
                renderer.render_line(
                    collider.point,
                    collider.point + collider.plane.normal,
                    color::BLUE,
                );

                // Tangent
                renderer.render_line(collider.point, collider.point + right, color::RED);

                // Bitangent
                renderer.render_line(collider.point, collider.point + up, color::GREEN);
            }
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

    let (mut app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    app.pause_updates();

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let h = app.timing_info.seconds_since_last_update;
        let uptime = app.timing_info.uptime_seconds;

        let resources = &scene_context.resources;

        let mut shader_context = (*shader_context_rc).borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            None,
        )?;

        let mut renderer = renderer_rc.borrow_mut();

        renderer.update(keyboard_state);

        if h > 0.0 {
            let mut simulation = simulation_rc.borrow_mut();

            // Advance our particle simulation by delta time.

            simulation.tick(h, uptime);
        }

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}
