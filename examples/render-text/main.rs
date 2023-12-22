extern crate sdl2;

use std::{cell::RefCell, env, path::Path};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::{Graphics, PixelBuffer, TextOperation},
};

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    match sdl2::ttf::init() {
        Ok(ttf_context) => {
            println!("Initialized TTF font subsystem.");

            // Load a system font

            let args: Vec<String> = env::args().collect();

            if args.len() < 2 {
                println!("Usage: cargo run --example render-text /path/to/your-font.fon");
                return Ok(());
            }

            let font_path = Path::new(&args[1]);

            let mut font = ttf_context.load_font(font_path, 128)?;

            // Set up our app

            let mut graphics = Graphics {
                buffer: PixelBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            };

            let now_seconds = RefCell::new(0.0);
            let mouse_x = RefCell::new(0);
            let mouse_y = RefCell::new(0);

            let mut update = |_keyboard_state: &KeyboardState,
                              mouse_state: &MouseState,
                              _game_controller_state: &GameControllerState,
                              seconds_since_last_update: f32|
             -> () {
                *now_seconds.borrow_mut() += seconds_since_last_update;
                *mouse_x.borrow_mut() = mouse_state.position.0;
                *mouse_y.borrow_mut() = mouse_state.position.1;
            };

            let mut render = || -> Result<Vec<u32>, String> {
                // Clears pixel buffer

                graphics.buffer.clear(color::BLACK);

                // Render some text to our pixel buffer

                font.set_style(sdl2::ttf::FontStyle::NORMAL);

                graphics.text(
                    &font,
                    TextOperation {
                        text: &(format!("Uptime: {}s", now_seconds.borrow())),
                        x: 12,
                        y: 12,
                        color: color::WHITE,
                    },
                )?;

                graphics.text(
                    &font,
                    TextOperation {
                        text: &(format!(
                            "Mouse position: ({},{})",
                            mouse_x.borrow(),
                            mouse_y.borrow()
                        )),
                        x: 12,
                        y: graphics.buffer.height - 12 - 16,
                        color: color::YELLOW,
                    },
                )?;

                // @TODO(mzalla) Return reference to a captured variable???
                return Ok(graphics.get_pixel_data().clone());
            };

            let app = App::new("examples/render-text", WINDOW_WIDTH, ASPECT_RATIO);

            app.run(&mut update, &mut render)?;
        }
        Err(e) => {
            println!("Error initializing ttf font subsystem: '{}'", e);
        }
    }

    Ok(())
}
