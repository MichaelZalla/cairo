use std::cell::RefCell;

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
    vec::vec3::Vec3,
};
use coordinates::{screen_to_world_space, world_to_screen_space};
use force::{Newtons, Point};
use make_simulation::make_simulation;
use renderable::Renderable;
use rigid_body_simulation_state::RigidBodySimulationState;
use sdl2::mouse::MouseButton;

mod coordinates;
mod force;
mod make_simulation;
mod renderable;
mod rigid_body;
mod rigid_body_simulation_state;
mod simulation;
mod state_vector;
mod transform;

#[derive(Default, Debug, Copy, Clone)]
struct ForceCreationState {
    start: Option<Vec3>,
}

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/rigid-body".to_string(),
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

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        if let Some(resolution) = &new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);
        }

        // Clear the framebuffer.

        framebuffer.clear(None);

        // Draws a circle with fill and border.

        let simulation = simulation_rc.borrow();

        for body in simulation.rigid_bodies.iter() {
            body.render(&mut framebuffer);
        }

        let cursor_world_space = cursor_world_space_rc.borrow();
        let force_creation_state = force_creation_state_rc.borrow();

        if let Some(start) = &force_creation_state.start {
            let (from, to) = (*start, cursor_world_space);

            let (from_screen_space, to_screen_space) = (
                world_to_screen_space(&from, &framebuffer),
                world_to_screen_space(&to, &framebuffer),
            );

            Graphics::line(
                &mut framebuffer,
                from_screen_space.x as i32,
                from_screen_space.y as i32,
                to_screen_space.x as i32,
                to_screen_space.y as i32,
                &color::WHITE,
            );
        }

        Ok(framebuffer.get_all().clone())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

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
                    let from = *cursor_world_space;

                    match &simulation.rigid_bodies[0].kind {
                        rigid_body::RigidBodyKind::Circle(radius) => {
                            let distance =
                                (from - *simulation.rigid_bodies[0].transform.translation()).mag();

                            if distance < *radius {
                                force_creation_state.start.replace(*cursor_world_space);
                            }
                        }
                    }
                }
                (MouseButton::Left, MouseEventKind::Up) => {
                    if let Some(start) = &force_creation_state.start {
                        let (from, to) = (*start, *cursor_world_space);

                        let f = (to - from) * 1000.0;

                        let force = Box::new(
                            move |_state: &RigidBodySimulationState,
                                  _current_time: f32|
                                  -> (Newtons, Option<Point>) {
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

    let render = |frame_index, new_resolution| -> Result<Vec<u32>, String> {
        render_scene_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}
