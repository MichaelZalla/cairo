use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    graphics::Graphics,
    vec::vec3::Vec3,
};

mod state_vector;

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

    let last_mouse_coordinates_rc =
        RefCell::new((framebuffer_center.x as u32, framebuffer_center.y as u32));

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        let last_mouse_coordinates = last_mouse_coordinates_rc.borrow();

        let mut framebuffer = framebuffer_rc.borrow_mut();

        if let Some(resolution) = &new_resolution {
            // Resize our framebuffer to match the window's new resolution.

            framebuffer.resize(resolution.width, resolution.height);
        }

        // Clear the framebuffer.

        framebuffer.clear(None);

        // Draws a circle with fill and border.

        Graphics::circle(
            &mut framebuffer,
            last_mouse_coordinates.0,
            last_mouse_coordinates.1,
            80,
            Some(&color::BLUE),
            Some(&color::YELLOW),
        );

        Ok(framebuffer.get_all().clone())
    };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let mut update = |_app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut last_mouse_coordinates = last_mouse_coordinates_rc.borrow_mut();

        last_mouse_coordinates.0 = mouse_state.position.0 as u32;
        last_mouse_coordinates.1 = mouse_state.position.1 as u32;

        Ok(())
    };

    let render = |frame_index, new_resolution| -> Result<Vec<u32>, String> {
        render_scene_to_framebuffer(frame_index, new_resolution)
    };

    app.run(&mut update, &render)?;

    Ok(())
}
