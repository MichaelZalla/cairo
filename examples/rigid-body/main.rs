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
    matrix::Mat4,
    mesh::Mesh,
    physics::simulation::rigid_body::RigidBodyKind,
    random::sampler::RandomSampler,
    render::{Renderer, options::RenderPassFlags},
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeGlobalTraversalMethod},
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    vec::vec3::Vec3,
};

use make_terrain_simulation::make_obj_terrain_simulation;
use make_terrain_simulation::make_terrain_simulation;
use make_test_simulation::make_test_simulation;

use crate::simulation::Simulation;

pub mod integration;
pub mod make_terrain_simulation;
pub mod make_test_simulation;
pub mod plane_collider;
pub mod scene;
pub mod simulation;

type SimulationMaker = fn(&mut RandomSampler<1024>, &SceneContext) -> (Simulation, Option<Mesh>);

const SIMULATION_MAKERS: [SimulationMaker; 3] = [
    make_test_simulation,
    make_terrain_simulation,
    make_obj_terrain_simulation,
];

// Fixed timestep accumulator for stable physics
const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0; // 60 Hz physics

struct DemoState {
    current_simulation_maker: usize,
    simulation: Simulation,
    renderer: Rc<RefCell<SoftwareRenderer>>,
    should_rasterize: bool,
    time_accumulator: f32,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/rigid-body".to_string(),
        window_resolution: resolution::RESOLUTION_640_BY_480 * 2.0,
        canvas_resolution: resolution::RESOLUTION_640_BY_480,
        relative_mouse_mode: false,
        ..Default::default()
    };

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 1000.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    // Simulation
    let mut sampler: RandomSampler<1024> = {
        let mut sampler: RandomSampler<1024> = Default::default();

        sampler.seed().unwrap();

        sampler
    };

    let (simulation, ground_mesh_option) = SIMULATION_MAKERS[0](&mut sampler, &scene_context);

    // Scene (camera and entities)

    let (scene, shader_context) = scene::make_scene(
        camera_aspect_ratio,
        &scene_context,
        &simulation,
        ground_mesh_option.as_ref(),
    )?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer
    let renderer_rc = {
        let mut renderer =
            SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

        renderer.options.draw_wireframe = true;
        renderer.options.render_pass_flags ^= RenderPassFlags::RASTERIZATION;
        renderer.options.draw_normals_scale = 1.5;

        renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

        Rc::new(RefCell::new(renderer))
    };

    // Mutable demo state
    let demo_state = Rc::new(RefCell::new(DemoState {
        current_simulation_maker: 0,
        simulation,
        renderer: renderer_rc.clone(),
        should_rasterize: false,
        time_accumulator: 0.0,
    }));

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let state = demo_state.borrow();

        state.renderer.borrow_mut().begin_frame();

        // Render scene entities (spheres and ground) - provides wireframes and rasterization
        {
            let scenes = scene_context.scenes.borrow();
            let renderer = state.renderer.as_ref();

            if let Some(scene) = scenes.first() {
                scene.render(&scene_context.resources, renderer, None)?;
            }
        }

        // Render debug visuals (axes, colliders, etc.)
        state.simulation.render(&mut state.renderer.borrow_mut());

        state.renderer.borrow_mut().end_frame();

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

        {
            let mut scenes = scene_context.scenes.borrow_mut();
            let scene = &mut scenes[0];

            // Traverses the scene graph and updates its nodes.
            scene.update(
                resources,
                &mut shader_context_rc.borrow_mut(),
                app,
                mouse_state,
                keyboard_state,
                game_controller_state,
                None,
            )?;
        }

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
                    reset_simulation(
                        &mut state,
                        &mut sampler,
                        &scene_context,
                        camera_aspect_ratio,
                        &shader_context_rc,
                    );
                }
                Keycode::T => {
                    // Toggle rasterization mode
                    state.should_rasterize = !state.should_rasterize;

                    let mut renderer = state.renderer.borrow_mut();

                    renderer.options.draw_wireframe = !state.should_rasterize;

                    if state.should_rasterize {
                        renderer.options.render_pass_flags |= RenderPassFlags::RASTERIZATION;
                        renderer.options.render_pass_flags |= RenderPassFlags::LIGHTING;
                    } else {
                        renderer.options.render_pass_flags &= !RenderPassFlags::RASTERIZATION;
                        renderer.options.render_pass_flags &= !RenderPassFlags::LIGHTING;
                    }
                }
                Keycode::Y => {
                    // Cycle current simulation maker (function).
                    state.current_simulation_maker =
                        (state.current_simulation_maker + 1) % SIMULATION_MAKERS.len();

                    reset_simulation(
                        &mut state,
                        &mut sampler,
                        &scene_context,
                        camera_aspect_ratio,
                        &shader_context_rc,
                    );
                }
                _ => (),
            }
        }

        // Updates renderer options.
        state.renderer.borrow_mut().update(keyboard_state);

        // Run physics simulation with fixed timestep
        if h > 0.0 {
            // Accumulate frame time
            state.time_accumulator += h;

            // Run physics at fixed timestep, subdividing large frame deltas
            while state.time_accumulator >= PHYSICS_TIMESTEP {
                state.simulation.tick(PHYSICS_TIMESTEP, uptime);
                state.time_accumulator -= PHYSICS_TIMESTEP;
            }

            // Update entity transforms in the scene to match simulation state
            let mut scenes = scene_context.scenes.borrow_mut();

            if let Some(scene) = scenes.first_mut() {
                // Update sphere entities by visiting the scene graph
                for (index, rigid_body) in state.simulation.rigid_bodies.iter().enumerate() {
                    let sphere_name = format!("sphere_{}", index);

                    scene
                        .root
                        .visit_mut(
                            SceneNodeGlobalTraversalMethod::DepthFirst,
                            None,
                            &mut |_depth: usize, _world_transform: Mat4, node: &mut SceneNode| {
                                if node.name.as_ref() == Some(&sphere_name) {
                                    // Update node's transform to match rigid body
                                    node.get_transform_mut()
                                        .set_translation(*rigid_body.transform.translation());

                                    node.get_transform_mut()
                                        .set_rotation(*rigid_body.transform.rotation());

                                    // Scale based on radius
                                    if let RigidBodyKind::Sphere(radius) = rigid_body.kind {
                                        node.get_transform_mut().set_scale(Vec3::ones() * radius);
                                    }
                                }
                                Ok(())
                            },
                        )
                        .ok();
                }
            }
        }

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}

fn reset_simulation(
    state: &mut DemoState,
    sampler: &mut RandomSampler<1024>,
    scene_context: &SceneContext,
    camera_aspect_ratio: f32,
    shader_context_rc: &Rc<RefCell<ShaderContext>>,
) {
    // Reset simulation to its initial state (preserved debug flags).
    let current_visualize_hash_grid = state.simulation.visualize_hash_grid;

    sampler.seed().unwrap();

    let (simulation, ground_mesh_option) =
        SIMULATION_MAKERS[state.current_simulation_maker](sampler, scene_context);

    state.simulation = simulation;

    state.simulation.visualize_hash_grid = current_visualize_hash_grid;

    // Reset time accumulator
    state.time_accumulator = 0.0;

    // Clear scene resource arenas to avoid accumulating resources on each reset
    scene_context.resources.clear();

    // Rebuilds the scene with new simulation and ground mesh.
    let (new_scene, new_shader_context) = scene::make_scene(
        camera_aspect_ratio,
        scene_context,
        &state.simulation,
        ground_mesh_option.as_ref(),
    )
    .expect("Failed to rebuild scene");

    // Replace the old scene with the new one
    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.clear();

        scenes.push(new_scene);
    }

    // Replace shader context since cameras/lights were recreated with new handles
    *shader_context_rc.borrow_mut() = new_shader_context;
}
