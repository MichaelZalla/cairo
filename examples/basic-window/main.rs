extern crate sdl2;

use cairo::{
    app::{App, AppWindowInfo},
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::pixelbuffer::PixelBuffer,
    time::TimingInfo,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/basic-window".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Set up our app

    let mut framebuffer = PixelBuffer::new(window_info.window_width, window_info.window_height);

    let mut update = |_timing_info: &TimingInfo,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> () {
        // @TODO Update any borrowed state here.
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Clears pixel buffer
        framebuffer.clear(color::BLACK);

        // @TODO Write some pixel data to the pixel buffer,
        //       based on some borrowed state.

        return Ok(framebuffer.get_pixels_u32().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
