extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    vec::vec3::Vec3,
};

use coordinates::screen_to_world_space;
use draw_collider::draw_collider;
use draw_wind_velocity::draw_wind_velocity;
use make_simulation::make_simulation;
use renderable::Renderable;

mod coordinates;
mod draw_collider;
mod draw_wind_velocity;
mod make_simulation;
mod point;
mod renderable;
mod simulation;
mod springy_mesh;
mod static_line_segment_collider;
mod strut;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/springy-mesh-2d".to_string(),
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

    let framebuffer_center = Vec3 {
        x: framebuffer.width as f32 / 2.0,
        y: framebuffer.height as f32 / 2.0,
        ..Default::default()
    };

    let framebuffer_rc = RefCell::new(framebuffer);

    // Set up our springy mesh simulation.

    let simulation = make_simulation();

    let simulation_rc = RefCell::new(simulation);

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   new_resolution: Option<Resolution>,
                                   canvas: &mut [u8]|
     -> Result<(), String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        let simulation = simulation_rc.borrow();

        if let Some(resolution) = &new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);
        }

        // Clear the framebuffer.

        framebuffer.clear(None);

        draw_wind_velocity(&simulation.wind, &mut framebuffer, &framebuffer_center);

        for mesh in &simulation.meshes {
            mesh.render(&mut framebuffer, &framebuffer_center);
        }

        for collider in &simulation.static_colliders {
            draw_collider(collider, &mut framebuffer, &framebuffer_center);
        }

        framebuffer.copy_to(canvas);

        Ok(())
    };

    let (mut app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    app.pause_updates();

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime_seconds = app.timing_info.uptime_seconds;

        let h = app.timing_info.seconds_since_last_update;

        let mut simulation = simulation_rc.borrow_mut();

        let cursor_world_space = screen_to_world_space(
            &Vec3 {
                x: mouse_state.position.0 as f32,
                y: mouse_state.position.1 as f32,
                z: 0.0,
            },
            &framebuffer_center,
        );

        simulation.wind = cursor_world_space * 3.0;

        simulation.tick(uptime_seconds, h);

        Ok(())
    };

    app.run(&mut update, &render_to_window_canvas)?;

    Ok(())
}
