extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/basic-window".to_string(),
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

    let framebuffer_rc = RefCell::new(framebuffer);

    let mut update = |_app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      _mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> { Ok(()) };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        // Clears pixel buffer

        framebuffer.clear(None);

        // @TODO Write some pixel data to the pixel buffer,
        //       based on some borrowed state.

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &render)?;

    Ok(())
}
