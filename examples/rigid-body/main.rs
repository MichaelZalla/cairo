use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    vec::vec3::{self, Vec3},
};
use coordinates::screen_to_world_space;
use quaternion::Quaternion;
use renderable::Renderable;
use rigid_body::RigidBody;

mod coordinates;
mod quaternion;
mod renderable;
mod rigid_body;
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

    let framebuffer_center = Vec3 {
        x: framebuffer.width as f32 / 2.0,
        y: framebuffer.height as f32 / 2.0,
        z: 0.0,
    };

    let framebuffer_rc = RefCell::new(framebuffer);

    let rigid_bodies = vec![RigidBody::circle(Default::default(), 5.0, 2.5)];

    let rigid_bodies_rc = RefCell::new(rigid_bodies);

    let last_mouse_coordinates_rc =
        RefCell::new((framebuffer_center.x as u32, framebuffer_center.y as u32));

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

        let rigid_bodies = rigid_bodies_rc.borrow();

        for body in rigid_bodies.iter() {
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
        let uptime = app.timing_info.uptime_seconds;

        let framebuffer = framebuffer_rc.borrow();

        let mut cursor_screen_space = last_mouse_coordinates_rc.borrow_mut();

        cursor_screen_space.0 = mouse_state.position.0 as u32;
        cursor_screen_space.1 = mouse_state.position.1 as u32;

        let cursor_world_space = screen_to_world_space(
            &Vec3::from_x_y(cursor_screen_space.0 as f32, cursor_screen_space.1 as f32),
            &framebuffer,
        );

        let mut rigid_bodies = rigid_bodies_rc.borrow_mut();

        for body in rigid_bodies.iter_mut() {
            let position = *body.transform.translation();

            if cursor_world_space.x == position.x && cursor_world_space.y == position.y {
                continue;
            }

            let body_to_cursor = cursor_world_space - position;

            let local_body_cursor_theta = body_to_cursor.as_normal().dot(vec3::RIGHT).acos();

            body.transform.set_translation(Vec3 {
                x: uptime.cos() * 5.0,
                y: uptime.sin() * 5.0,
                z: 0.0,
            });

            body.transform.set_orientation(Quaternion::new_2d(
                if cursor_world_space.y < position.y {
                    -local_body_cursor_theta
                } else {
                    local_body_cursor_theta
                },
            ));
        }

        Ok(())
    };

    let render = |frame_index, new_resolution| -> Result<Vec<u32>, String> {
        render_scene_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}
