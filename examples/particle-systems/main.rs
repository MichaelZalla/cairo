extern crate sdl2;

use core::f32;

use std::{cell::RefCell, f32::consts::PI};

use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color::{self, Color},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    graphics::Graphics,
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

use force::{Acceleration, Force};

use particle::{
    generator::{ParticleGenerator, ParticleGeneratorKind},
    particlelist::ParticleList,
    Particle, PARTICLE_MAX_AGE_SECONDS,
};

mod force;
mod particle;

static PIXELS_PER_METER: f32 = 4.0;

static GRAVITY: Force = |_: &Particle| -> Acceleration {
    Vec3 {
        x: 0.0,
        y: -(SDL_STANDARD_GRAVITY as f32),
        z: 0.0,
    }
};

static AIR_RESISTANCE: Force = |particle: &Particle| -> Acceleration {
    static D: f32 = 0.2;

    static WIND: Vec3 = Vec3 {
        x: -12.5,
        y: -3.0,
        z: 0.0,
    };

    (WIND - particle.velocity) * (D / particle.mass)
};

struct Simulation<'a> {
    pub sampler: RefCell<RandomSampler<1024>>,
    pub pool: RefCell<ParticleList>,
    pub forces: Vec<&'a Force>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
}

fn screen_to_world_space(screen_space_position: &Vec3, framebuffer_center: &Vec3) -> Vec3 {
    let mut world_space_position = *screen_space_position;

    world_space_position = (world_space_position - *framebuffer_center) / PIXELS_PER_METER;

    world_space_position.y *= -1.0;

    world_space_position
}

fn world_to_screen_space(world_space_position: &Vec3, framebuffer_center: &Vec3) -> Vec3 {
    let mut screen_space_position = *world_space_position;

    screen_space_position.y *= -1.0;

    screen_space_position * PIXELS_PER_METER + *framebuffer_center
}

fn draw_particle(
    particle: &Particle,
    screen_space_position: &Vec3,
    _prev_screen_space_position: &Vec3,
    framebuffer: &mut Buffer2D,
) {
    debug_assert!(particle.alive);

    let age_alpha = particle.age / PARTICLE_MAX_AGE_SECONDS;

    let age_alpha_u8 = 255.0 * (1.0 - age_alpha.min(1.0));

    let color = Color {
        r: age_alpha_u8,
        g: age_alpha_u8,
        b: age_alpha_u8,
        a: 1.0,
    };

    // framebuffer.set(
    //     screen_space_position.x as u32,
    //     screen_space_position.y as u32,
    //     color.to_u32(),
    // );

    // Graphics::line(
    //     framebuffer,
    //     screen_space_position.x as i32,
    //     screen_space_position.y as i32,
    //     prev_screen_space_position.x as i32,
    //     prev_screen_space_position.y as i32,
    //     &color,
    // );

    let top_left = (
        screen_space_position.x as i32,
        screen_space_position.y as i32,
    );

    if (top_left.0 as u32) > framebuffer.width - 8 {
        return;
    }

    if (top_left.1 as u32) > framebuffer.height - 8 {
        return;
    }

    if top_left.0 + 8 < 0 {
        return;
    }

    if top_left.1 + 8 < 0 {
        return;
    }

    Graphics::rectangle(
        framebuffer,
        top_left.0 as u32,
        top_left.1 as u32,
        8,
        8,
        Some(&color),
        None,
    )
}

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

    let omnidirectional = ParticleGenerator::new(
        ParticleGeneratorKind::Omnidirectional(Vec3 {
            x: 0.0,
            y: 20.0,
            z: 0.0,
        }),
        100.0,
        None,
        200.0,
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
        200.0,
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
        200.0,
        8.0,
    );

    let sim = Simulation {
        sampler: Default::default(),
        pool: Default::default(),
        forces: vec![&GRAVITY, &AIR_RESISTANCE],
        generators: RefCell::new(vec![omnidirectional, directional_right, directional_up]),
    };

    {
        let mut sampler = sim.sampler.borrow_mut();

        sampler.seed().unwrap();
    }

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime_seconds = app.timing_info.uptime_seconds;

        let h = app.timing_info.seconds_since_last_update;

        let mut sampler = sim.sampler.borrow_mut();
        let mut pool = sim.pool.borrow_mut();
        let mut generators = sim.generators.borrow_mut();

        let cursor_screen_space = Vec3 {
            x: mouse_state.position.0 as f32,
            y: mouse_state.position.1 as f32,
            z: 0.0,
        };

        let cursor_world_space = screen_to_world_space(&cursor_screen_space, &framebuffer_center);

        for generator in generators.iter_mut() {
            match generator.kind {
                ParticleGeneratorKind::Omnidirectional(ref mut origin) => {
                    *origin = Vec3 {
                        y: 30.0 + 20.0 * (uptime_seconds * 3.0).sin(),
                        x: origin.x,
                        z: origin.z,
                    }
                }
                ParticleGeneratorKind::Directed(origin, ref mut direction) => {
                    *direction = (cursor_world_space - origin).as_normal();
                }
            }

            generator.generate(&mut pool, &mut sampler, h)?;
        }

        pool.test_and_deactivate(h);

        pool.compute_accelerations(&sim.forces);

        pool.integrate(h);

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