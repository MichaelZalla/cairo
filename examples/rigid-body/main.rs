extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use cairo::{
    app::{
        resolution::{self, Resolution},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    render::Renderer,
    scene::context::{utils::make_empty_scene, SceneContext},
    software_renderer::SoftwareRenderer,
};

use simulation::make_simulation;

pub mod hash_grid;
pub mod integration;
pub mod plane_collider;
pub mod simulation;
pub mod state_vector;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/rigid-body".to_string(),
        window_resolution: resolution::RESOLUTION_1280_BY_720,
        canvas_resolution: resolution::RESOLUTION_1280_BY_720,
        relative_mouse_mode: false,
        ..Default::default()
    };

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 1000.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Scene context

    let scene_context = SceneContext::default();

    // Scene (camera)

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

        let camera = &mut active_camera_entry.item;

        camera.set_projection_z_far(1_000.0);

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

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        {
            let simulation = simulation_rc.borrow();

            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();

            simulation.render(&mut renderer);

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

    // Create and run our app.

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

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
