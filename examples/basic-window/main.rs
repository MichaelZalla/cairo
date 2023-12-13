extern crate sdl2;

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::{Graphics, PixelBuffer},
};

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Set up our app

    let mut graphics = Graphics {
        buffer: PixelBuffer {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            width_over_height: ASPECT_RATIO,
            height_over_width: 1.0 / ASPECT_RATIO,
            pixels: vec![0 as u32; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize],
        },
    };

    let update = |_keyboard_state: &KeyboardState,
                  _mouse_state: &MouseState,
                  _game_controller_state: &GameControllerState,
                  _delta_t_seconds: f32|
     -> () {
        // @TODO Update any borrowed state here.
    };

    let render = || -> Result<Vec<u32>, String> {
        // Clears pixel buffer
        graphics.buffer.clear(color::BLACK);

        // @TODO Write some pixel data to the pixel buffer,
        //       based on some borrowed state.

        // @TODO Return reference to a captured variable?
        return Ok(graphics.get_pixel_data().clone());
    };

    let app = App::new(
        "examples/basic-window",
        WINDOW_WIDTH,
        ASPECT_RATIO,
        update,
        render,
    );

    app.run()?;

    Ok(())
}
