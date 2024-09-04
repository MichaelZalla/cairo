extern crate sdl2;

use core::f32;

use std::{cell::RefCell, rc::Rc};

use draw_particle::{draw_particle, screen_to_world_space, world_to_screen_space};
use make_simulation::{make_simulation, SEED_SIZE};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color::{self},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

mod draw_particle;
mod force;
mod make_simulation;
mod operator;
mod particle;
mod simulation;
mod state_vector;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/particle-systems".to_string(),
        window_resolution: RESOLUTION_1280_BY_720,
        canvas_resolution: RESOLUTION_1280_BY_720,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Set up our app

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let framebuffer_center = Vec3 {
        x: framebuffer.width as f32 / 2.0,
        y: framebuffer.height as f32 / 2.0,
        z: 0.0,
    };

    let framebuffer_rc = RefCell::new(framebuffer);

    // Set up our particle simulation.

    let mut sampler: RandomSampler<SEED_SIZE> = Default::default();

    // Seed the simulation's random number sampler.

    match sampler.seed() {
        Ok(_) => (),
        Err(err) => return Err(format!("{}", err)),
    }

    let sampler_rc = Rc::new(RefCell::new(sampler));
    let sampler_rc_for_random_acceleration_operator = sampler_rc.clone();

    let sim = make_simulation(sampler_rc, sampler_rc_for_random_acceleration_operator);

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let h = app.timing_info.seconds_since_last_update;

        let uptime_seconds = app.timing_info.uptime_seconds;

        let cursor_screen_space = Vec3 {
            x: mouse_state.position.0 as f32,
            y: mouse_state.position.1 as f32,
            z: 0.0,
        };

        let cursor_world_space = screen_to_world_space(&cursor_screen_space, &framebuffer_center);

        // Simulation tick.

        sim.tick(h, uptime_seconds, &cursor_world_space)?;

        Ok(())
    };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
        let pool = sim.pool.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        // Clears pixel buffer

        framebuffer.clear(Some(color::BLACK.to_u32()));

        for particle in pool.iter() {
            if particle.alive {
                let screen_space_position =
                    world_to_screen_space(&particle.position, &framebuffer_center);

                if (screen_space_position.x as u32) < framebuffer.width
                    && (screen_space_position.y as u32) < framebuffer.height
                {
                    let prev_screen_space_position =
                        world_to_screen_space(&particle.prev_position, &framebuffer_center);

                    draw_particle(
                        particle,
                        &screen_space_position,
                        &prev_screen_space_position,
                        &mut framebuffer,
                    );
                }
            }
        }

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &render)?;

    Ok(())
}
