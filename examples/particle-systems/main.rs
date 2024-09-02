extern crate sdl2;

use core::f32;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use draw_particle::{draw_particle, screen_to_world_space, world_to_screen_space};
use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color::{self},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    random::sampler::{DirectionSampler, RandomSampler},
    vec::{vec3::Vec3, vec4::Vec4},
};

use force::{Force, Newtons};

use particle::{
    generator::{ParticleGenerator, ParticleGeneratorKind},
    Particle,
};

use simulation::{Operators, Simulation};

mod draw_particle;
mod force;
mod particle;
mod simulation;

static GRAVITY: Force = |particle: &Particle| -> Newtons {
    Vec3 {
        x: 0.0,
        y: -(SDL_STANDARD_GRAVITY as f32),
        z: 0.0,
    } * particle.mass
};

static AIR_RESISTANCE: Force = |particle: &Particle| -> Newtons {
    static D: f32 = 0.2;

    static WIND: Vec3 = Vec3 {
        x: -12.5,
        y: -3.0,
        z: 0.0,
    };

    (WIND - particle.velocity) * D
};

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

    // Define some particle generators.

    let omnidirectional = ParticleGenerator::new(
        ParticleGeneratorKind::Omnidirectional(Vec3 {
            x: 0.0,
            y: 20.0,
            z: 0.0,
        }),
        100.0,
        None,
        100.0,
        8.0,
    );

    let directional_right = ParticleGenerator::new(
        ParticleGeneratorKind::Directed(
            Vec3 {
                x: -50.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        100.0,
        Some(PI / 4.0),
        100.0,
        8.0,
    );

    let directional_up = ParticleGenerator::new(
        ParticleGeneratorKind::Directed(
            Vec3 {
                x: 50.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.001,
                y: 1.0,
                z: 0.0,
            },
        ),
        100.0,
        Some(PI / 2.0),
        100.0,
        8.0,
    );

    // Set up our particle simulation.

    const SEED_SIZE: usize = 2048;

    let mut sampler: RandomSampler<SEED_SIZE> = Default::default();

    // Seed the simulation's random number sampler.

    match sampler.seed() {
        Ok(_) => (),
        Err(err) => return Err(format!("{}", err)),
    }

    let sampler_rc = Rc::new(RefCell::new(sampler));

    let sampler_rc_for_random_acceleration_operator = sampler_rc.clone();

    let operators = Operators {
        additive_acceleration: vec![
            // Additive acceleration operator: Contributes a random acceleration.
            Box::new(move |_particle: &Particle, h: f32| -> Vec3 {
                static SCALING_FACTOR: f32 = 1.0;

                let mut sampler = sampler_rc_for_random_acceleration_operator.borrow_mut();

                sampler.sample_direction_uniform() * SCALING_FACTOR / h
            }),
        ],
        functional_acceleration: vec![
            // Functional acceleration operator: Enforces a minimum velocity;
            Box::new(
                |particle: &Particle, new_velocity: &Vec3, _h: f32| -> Vec3 {
                    static MINIMUM_SPEED: f32 = 20.0;

                    let current_speed = particle.velocity.mag();
                    let new_speed = new_velocity.mag();

                    if new_speed >= MINIMUM_SPEED {
                        return *new_velocity;
                    }

                    if current_speed > MINIMUM_SPEED {
                        particle.velocity
                    } else {
                        particle.velocity.as_normal() * MINIMUM_SPEED
                    }
                },
            ),
            // Functional acceleration operator: Rotation around the Z-axis.
            Box::new(
                |_particle: &Particle, new_velocity: &Vec3, h: f32| -> Vec3 {
                    static ANGLE: f32 = PI / 2.0;

                    let new_velocity_vec4 =
                        Vec4::new(*new_velocity, 1.0) * Mat4::rotation_z(ANGLE * h);

                    new_velocity_vec4.to_vec3()
                },
            ),
        ],
    };

    let sim = Simulation {
        sampler: sampler_rc.clone(),
        pool: Default::default(),
        forces: vec![&GRAVITY, &AIR_RESISTANCE],
        operators: RefCell::new(operators),
        generators: RefCell::new(vec![omnidirectional, directional_right, directional_up]),
    };

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
