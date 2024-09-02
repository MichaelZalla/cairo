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
                x: -75.0,
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
                x: 75.0,
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
            Box::new(
                move |_particle: &Particle, _total_acceleration: &Vec3, h: f32| -> Vec3 {
                    static SCALING_FACTOR: f32 = 1.0;

                    let mut sampler = sampler_rc_for_random_acceleration_operator.borrow_mut();

                    sampler.sample_direction_uniform() * SCALING_FACTOR / h
                },
            ),
            // Additive acceleration operator: Avoids a static sphere collider.
            Box::new(
                |particle: &Particle, total_acceleration: &Vec3, _h: f32| -> Vec3 {
                    static COLLIDER_CENTER: Vec3 = Vec3 {
                        x: -15.0,
                        y: -15.0,
                        z: 0.0,
                    };

                    static COLLIDER_RADIUS: f32 = 40.0;
                    static COLLIDER_SAFE_RADIUS: f32 = COLLIDER_RADIUS + 5.0;
                    static THRESHOLD_TIME: f32 = 3.0;

                    if particle.velocity.mag() == 0.0 {
                        return Default::default();
                    }

                    let particle_direction = particle.velocity.as_normal();

                    let particle_to_collider_center = COLLIDER_CENTER - particle.position;

                    let distance_of_particle_closest_approach_to_collider_center =
                        particle_to_collider_center.dot(particle_direction);

                    if distance_of_particle_closest_approach_to_collider_center < 0.0 {
                        return Default::default();
                    }

                    let distance_of_concern = particle.velocity.mag() * THRESHOLD_TIME;

                    if distance_of_particle_closest_approach_to_collider_center
                        > distance_of_concern
                    {
                        return Default::default();
                    }

                    let closest_approach = particle.position
                        + particle_direction
                            * distance_of_particle_closest_approach_to_collider_center;

                    let collider_center_to_closest_approach = closest_approach - COLLIDER_CENTER;

                    let collider_center_to_closest_approach_direction =
                        collider_center_to_closest_approach.as_normal();

                    let collider_center_to_closest_approach_distance =
                        collider_center_to_closest_approach.mag();

                    if collider_center_to_closest_approach_distance > COLLIDER_SAFE_RADIUS {
                        return Default::default();
                    }

                    let turning_target = COLLIDER_CENTER
                        + collider_center_to_closest_approach_direction * COLLIDER_SAFE_RADIUS;

                    let particle_to_turning_target = turning_target - particle.position;
                    let particle_to_turning_target_distance = particle_to_turning_target.mag();

                    let velocity_towards_turning_target = particle
                        .velocity
                        .dot(particle_to_turning_target / particle_to_turning_target_distance);

                    let time_to_reach_turning_target =
                        particle_to_turning_target_distance / velocity_towards_turning_target;

                    let average_speed_increase_othogonal_to_velocity =
                        (particle_direction.cross(particle_to_turning_target)).mag()
                            / time_to_reach_turning_target;

                    let required_magnitude_of_acceleration = 2.0
                        * average_speed_increase_othogonal_to_velocity
                        / time_to_reach_turning_target;

                    let existing_acceleration_in_collider_center_to_closest_approach_direction =
                        collider_center_to_closest_approach_direction.dot(*total_acceleration);

                    collider_center_to_closest_approach_direction * (required_magnitude_of_acceleration
                    - existing_acceleration_in_collider_center_to_closest_approach_direction)
                    .max(0.0)
                },
            ),
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
            // Box::new(
            //     |_particle: &Particle, new_velocity: &Vec3, h: f32| -> Vec3 {
            //         static ANGLE: f32 = PI / 2.0;

            //         let new_velocity_vec4 =
            //             Vec4::new(*new_velocity, 1.0) * Mat4::rotation_z(ANGLE * h);

            //         new_velocity_vec4.to_vec3()
            //     },
            // ),
        ],
        velocity: vec![
            // Velocity operator: Vortex.
            Box::new(|particle: &Particle, new_velocity: &Vec3, h: f32| -> Vec3 {
                static VORTEX_CENTER: Vec3 = Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                };

                static VORTEX_RADIUS: f32 = 200.0;

                static VORTEX_ROTATIONAL_FREQUENCY_AT_RADIUS: f32 = 1.5;

                static VORTEX_ROTATIONAL_FREQUENCY_MAX: f32 = 10.0;

                static VORTEX_TIGHTNESS: f32 = 1.5;

                let particle_distance_to_vortex_center = (particle.position - VORTEX_CENTER).mag();

                let particle_rotational_frequency_scaling_factor =
                    (VORTEX_RADIUS / particle_distance_to_vortex_center).powf(VORTEX_TIGHTNESS);

                let particle_rotational_frequency = (VORTEX_ROTATIONAL_FREQUENCY_AT_RADIUS
                    * particle_rotational_frequency_scaling_factor)
                    .max(VORTEX_ROTATIONAL_FREQUENCY_MAX);

                let omega = 2.0 * PI * particle_rotational_frequency;

                let new_velocity_vec4 = Vec4::new(*new_velocity, 1.0) * Mat4::rotation_z(omega * h);

                new_velocity_vec4.to_vec3()
            }),
            // Velocity operator: Translation by offset.
            Box::new(
                |_particle: &Particle, new_velocity: &Vec3, _h: f32| -> Vec3 {
                    static OFFSET: Vec3 = Vec3 {
                        x: 50.0,
                        y: 0.0,
                        z: 0.0,
                    };

                    *new_velocity + OFFSET
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
