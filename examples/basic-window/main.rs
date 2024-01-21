extern crate sdl2;

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::{pixelbuffer::PixelBuffer, Graphics},
    time::TimingInfo,
};

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Set up our app

    let mut graphics = Graphics {
        buffer: PixelBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    };

    let mut update = |_timing_info: &TimingInfo,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> () {
        // @TODO Update any borrowed state here.
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Clears pixel buffer
        graphics.buffer.clear(color::BLACK);

        // @TODO Write some pixel data to the pixel buffer,
        //       based on some borrowed state.

        return Ok(graphics.buffer.get_pixel_data().clone());
    };

    let app = App::new("examples/basic-window", WINDOW_WIDTH, ASPECT_RATIO);

    app.run(&mut update, &mut render)?;

    Ok(())
}
