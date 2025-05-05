use std::cell::RefCell;

use sdl2::mouse::MouseButton;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{MouseEventKind, MouseState},
    },
    graphics::Graphics,
    physics::simulation::{
        force::ContactPoint,
        rigid_body::{rigid_body_simulation_state::RigidBodySimulationState, RigidBodyKind},
        units::Newtons,
    },
    vec::vec3::{self, Vec3},
};

use coordinates::{screen_to_world_space, world_to_screen_space, PIXELS_PER_METER};
use make_simulation::make_simulation;

mod coordinates;
mod make_simulation;
mod simulation;
mod state_vector;

#[derive(Default, Debug, Copy, Clone)]
struct ForceCreationState {
    start: Option<Vec3>,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/rigid-body-2d".to_string(),
        relative_mouse_mode: false,
        resizable: true,
        ..Default::default()
    };

    // Set up our app

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let framebuffer_rc = RefCell::new(framebuffer);

    let simulation = make_simulation();

    let simulation_rc = RefCell::new(simulation);

    let cursor_world_space_rc = RefCell::new(Vec3::default());

    let force_creation_state_rc = RefCell::new(ForceCreationState::default());

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        if let Some(resolution) = &new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);
        }

        // Clear the framebuffer.

        framebuffer.clear(None);

        // Draws a circle with fill and border.

        let simulation = simulation_rc.borrow();

        for circle in simulation.rigid_bodies.iter() {
            match circle.kind {
                RigidBodyKind::Circle(radius) => {
                    let transform = &circle.transform;

                    let position_screen_space =
                        world_to_screen_space(transform.translation(), &framebuffer);

                    // Draw the circle's outline.

                    Graphics::circle(
                        &mut framebuffer,
                        position_screen_space.x as i32,
                        position_screen_space.y as i32,
                        (radius * PIXELS_PER_METER) as u32,
                        None,
                        Some(color::YELLOW.to_u32()),
                    );

                    // Draw a line to indicate the body's orientation.

                    let local_right = vec3::RIGHT;
                    let global_right = local_right * *transform.rotation().mat();

                    let end = *transform.translation() + (global_right * radius);
                    let end_screen_space = world_to_screen_space(&end, &framebuffer);

                    Graphics::line(
                        &mut framebuffer,
                        position_screen_space.x as i32,
                        position_screen_space.y as i32,
                        end_screen_space.x as i32,
                        end_screen_space.y as i32,
                        color::ORANGE.to_u32(),
                    );
                }
                _ => panic!(),
            }
        }

        let cursor_world_space = cursor_world_space_rc.borrow();
        let force_creation_state = force_creation_state_rc.borrow();

        if let Some(start) = &force_creation_state.start {
            let (from, to) = (*start, cursor_world_space);

            let (from_screen_space, to_screen_space) = (
                world_to_screen_space(&from, &framebuffer),
                world_to_screen_space(&to, &framebuffer),
            );

            let color_u32 = color::WHITE.to_u32();

            Graphics::line(
                &mut framebuffer,
                from_screen_space.x as i32,
                from_screen_space.y as i32,
                to_screen_space.x as i32,
                to_screen_space.y as i32,
                color_u32,
            );
        }

        framebuffer.copy_to(canvas);

        Ok(())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime_seconds = app.timing_info.uptime_seconds;
        let h = app.timing_info.seconds_since_last_update;

        // Translate the cursor's current position to world space.

        let cursor_screen_space = (mouse_state.position.0 as u32, mouse_state.position.1 as u32);

        let framebuffer = framebuffer_rc.borrow();

        let mut cursor_world_space = cursor_world_space_rc.borrow_mut();

        *cursor_world_space = screen_to_world_space(
            &Vec3::from_x_y(cursor_screen_space.0 as f32, cursor_screen_space.1 as f32),
            &framebuffer,
        );

        // Are we drawing a new force to dispatch?

        let mut simulation = simulation_rc.borrow_mut();

        let mut force_creation_state = force_creation_state_rc.borrow_mut();

        if let Some(event) = &mouse_state.button_event {
            match (event.button, event.kind) {
                (MouseButton::Left, MouseEventKind::Down) => {
                    let circle = &simulation.rigid_bodies[0];

                    let transform = &circle.transform;

                    let from = *cursor_world_space;

                    let distance = (from - transform.translation()).mag();

                    let circle = &simulation.rigid_bodies[0];

                    let radius = match circle.kind {
                        RigidBodyKind::Circle(radius) => radius,
                        _ => panic!(),
                    };

                    if distance < radius {
                        force_creation_state.start.replace(*cursor_world_space);
                    }
                }
                (MouseButton::Left, MouseEventKind::Up) => {
                    if let Some(start) = &force_creation_state.start {
                        let (from, to) = (*start, *cursor_world_space);

                        let f = (to - from) * 1000.0;

                        let force = Box::new(
                            move |_state: &RigidBodySimulationState,
                                  _i: usize,
                                  _current_time: f32|
                                  -> (Newtons, Option<ContactPoint>) {
                                (f, Some(from))
                            },
                        );

                        simulation.forces.push(force);

                        force_creation_state.start.take();
                    }
                }
                _ => (),
            }
        }

        // Advance our simulation.

        simulation.tick(uptime_seconds, h, *cursor_world_space);

        simulation.forces = vec![];

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}
