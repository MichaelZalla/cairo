use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    vec::vec3::Vec3,
};
use coordinates::screen_to_world_space;
use make_simulation::make_simulation;
use renderable::Renderable;

mod coordinates;
mod force;
mod make_simulation;
mod quaternion;
mod renderable;
mod rigid_body;
mod simulation;
mod state_vector;
mod transform;

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

        let mut simulation = simulation_rc.borrow_mut();

        simulation.tick(uptime_seconds, h, *cursor_world_space);

        Ok(())
    };

    let render = |frame_index, new_resolution| -> Result<Vec<u32>, String> {
        render_scene_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}
