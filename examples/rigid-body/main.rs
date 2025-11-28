extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{
        App, AppWindowInfo,
        resolution::{self, Resolution},
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    mesh::primitive::sphere,
    random::sampler::RandomSampler,
    render::{Renderer, options::RenderPassFlags},
    scene::context::{SceneContext, utils::make_empty_scene},
    software_renderer::SoftwareRenderer,
};

use make_terrain_simulation::make_obj_terrain_simulation as make_simulation;

use crate::simulation::Simulation;

pub mod integration;
pub mod make_terrain_simulation;
pub mod make_test_simulation;
pub mod plane_collider;
pub mod simulation;
pub mod state_vector;
pub mod static_collider;

// Fixed timestep accumulator for stable physics
const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0; // 60 Hz physics

struct DemoState {
    simulation: Simulation,
    renderer: SoftwareRenderer,
    should_rasterize: bool,
    time_accumulator: f32,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/rigid-body".to_string(),
        window_resolution: resolution::RESOLUTION_640_BY_480,
        canvas_resolution: resolution::RESOLUTION_640_BY_480,
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

        let (scene, shader_context) = make_empty_scene(
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

        (scene, shader_context)
    };

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Generate sphere mesh for rasterized rendering
    let sphere_mesh = sphere::generate(1.0, 16, 16);

    // Renderer
    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    renderer.options.draw_wireframe = true;
    renderer.options.render_pass_flags ^= RenderPassFlags::RASTERIZATION;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    // Simulation
    let mut sampler: RandomSampler<1024> = {
        let mut sampler: RandomSampler<1024> = Default::default();

        sampler.seed().unwrap();

        sampler
    };

    let simulation = make_simulation(&mut sampler, &scene_context);

    // Mutable demo state
    let demo_state = Rc::new(RefCell::new(DemoState {
        simulation,
        renderer,
        should_rasterize: false,
        time_accumulator: 0.0,
    }));

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let mut state = demo_state.borrow_mut();

        // Split borrow to allow simultaneous access to simulation and renderer
        let DemoState {
            simulation,
            renderer,
            should_rasterize,
            ..
        } = &mut *state;

        renderer.begin_frame();

        let sphere_mesh_option = if *should_rasterize {
            Some(&sphere_mesh)
        } else {
            None
        };

        simulation.render(renderer, sphere_mesh_option);

        renderer.end_frame();

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

    let (mut app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    app.pause_updates();

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let h = app.timing_info.seconds_since_last_update;
        let uptime = app.timing_info.uptime_seconds;

        let resources = &scene_context.resources;

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        scene.update(
            resources,
            &mut shader_context_rc.borrow_mut(),
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            None,
        )?;

        // Borrow demo state once for the entire update
        let mut state = demo_state.borrow_mut();

        // Updates the simulation parameters using keyboard.

        for keycode in keyboard_state.newly_pressed_keycodes.iter() {
            #[allow(clippy::single_match)]
            match *keycode {
                Keycode::G => {
                    // Toggles hash grid visualization.
                    state.simulation.visualize_hash_grid = !state.simulation.visualize_hash_grid;
                }
                Keycode::U => {
                    // Pushes bodies in contact away from their collider.
                    for body in state.simulation.rigid_bodies.iter_mut() {
                        for contact in &body.static_contacts {
                            body.linear_momentum += contact.normal * 10.0;
                        }
                    }
                }
                Keycode::R => {
                    // Reset simulation to its initial state (preserved debug flags).
                    let current_visualize_hash_grid = state.simulation.visualize_hash_grid;

                    state.simulation = make_simulation(&mut sampler, &scene_context);

                    state.simulation.visualize_hash_grid = current_visualize_hash_grid;

                    // Reset time accumulator
                    state.time_accumulator = 0.0;
                }
                Keycode::T => {
                    // Toggle rasterization mode
                    state.should_rasterize = !state.should_rasterize;

                    state.renderer.options.draw_wireframe = !state.should_rasterize;

                    if state.should_rasterize {
                        state.renderer.options.render_pass_flags |= RenderPassFlags::RASTERIZATION;
                    } else {
                        state.renderer.options.render_pass_flags &= !RenderPassFlags::RASTERIZATION;
                    }
                }

                _ => (),
            }
        }

        // Updates renderer options.
        state.renderer.update(keyboard_state);

        // Run physics simulation with fixed timestep
        if h > 0.0 {
            // Accumulate frame time
            state.time_accumulator += h;

            // Run physics at fixed timestep, subdividing large frame deltas
            while state.time_accumulator >= PHYSICS_TIMESTEP {
                state.simulation.tick(PHYSICS_TIMESTEP, uptime);
                state.time_accumulator -= PHYSICS_TIMESTEP;
            }
        }

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}
