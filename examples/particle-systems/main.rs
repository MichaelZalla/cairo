extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use sdl2::{keyboard::Keycode, mouse::MouseButton};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{MouseEventKind, MouseState},
    },
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

use collider::LineSegmentCollider;
use coordinates::{screen_to_world_space, world_to_screen_space};
use draw_collider::draw_collider;
use draw_particle::draw_particle;
use draw_quadtree::draw_quadtree;
use make_simulation::{make_simulation, SEED_SIZE};
use particle::MAX_PARTICLE_SIZE_PIXELS;

mod collider;
mod coordinates;
mod draw_collider;
mod draw_particle;
mod draw_quadtree;
mod make_simulation;
mod operator;
mod particle;
mod quadtree;
mod simulation;
mod state_vector;

struct LineSegmentColliderCreationState {
    start: Option<Vec3>,
    current: Option<Vec3>,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/particle-systems".to_string(),
        window_resolution: RESOLUTION_1280_BY_720,
        canvas_resolution: RESOLUTION_1280_BY_720,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

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

    let collider_creation_state_rc = RefCell::new(LineSegmentColliderCreationState {
        start: None,
        current: None,
    });

    let draw_debug = RefCell::new(false);

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
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

        // Check if we're creating any new collider.

        let mut collider_creation_state = collider_creation_state_rc.borrow_mut();

        collider_creation_state.current.replace(cursor_world_space);

        if let Some(button_event) = mouse_state.button_event {
            match (button_event.button, button_event.kind) {
                (MouseButton::Left, MouseEventKind::Down) => {
                    // Are we starting a new line segment?

                    if collider_creation_state.start.is_none() {
                        collider_creation_state.start.replace(cursor_world_space);
                    }
                }
                (MouseButton::Left, MouseEventKind::Up) => {
                    // Are we finishing a new line segment?

                    if let Some(start) = collider_creation_state.start {
                        let distance = (cursor_world_space - start).mag();

                        if distance > 16.0 {
                            let end = cursor_world_space;

                            let collider = LineSegmentCollider::new(start, end);

                            sim.colliders.borrow_mut().push(collider);
                        }

                        collider_creation_state.start.take();
                    }
                }
                (MouseButton::Right, MouseEventKind::Down) => {
                    // Are we cancelling a new line segment?

                    collider_creation_state.start.take();
                    collider_creation_state.current.take();
                }
                _ => (),
            }
        }

        // Simulation tick.

        sim.tick(h, uptime_seconds, &cursor_world_space)?;

        // Inputs.

        if keyboard_state.newly_pressed_keycodes.contains(&Keycode::Q) {
            let mut debug = draw_debug.borrow_mut();

            *debug = !*debug;
        }

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let pool = sim.pool.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        // Clears pixel buffer

        framebuffer.clear(None);

        for particle in pool.iter() {
            if particle.alive {
                let screen_space_position =
                    world_to_screen_space(&particle.position, &framebuffer_center);

                if (screen_space_position.x as i32 - MAX_PARTICLE_SIZE_PIXELS as i32)
                    < framebuffer.width as i32
                    && (screen_space_position.y as i32 - MAX_PARTICLE_SIZE_PIXELS as i32)
                        < framebuffer.height as i32
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

        // Visualize our (created and pending) colliders.

        {
            let colliders = sim.colliders.borrow();

            for collider in colliders.iter() {
                draw_collider(collider, &mut framebuffer, &framebuffer_center);
            }

            let collider_creation_state = collider_creation_state_rc.borrow();

            if let (Some(start), Some(end)) = (
                collider_creation_state.start,
                collider_creation_state.current,
            ) {
                draw_collider(
                    &LineSegmentCollider::new(start, end),
                    &mut framebuffer,
                    &framebuffer_center,
                );
            }
        }

        // Visualize of our simulation's quadtree.

        if *draw_debug.borrow() {
            let quadtree = sim.quadtree.borrow();

            draw_quadtree(&quadtree, &mut framebuffer, &framebuffer_center);
        }

        framebuffer.copy_to(canvas);

        Ok(())
    };

    app.run(&mut update, &render)?;

    Ok(())
}
