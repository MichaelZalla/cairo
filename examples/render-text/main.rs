extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{pixelbuffer::PixelBuffer, text::TextOperation, Graphics},
    time::TimingInfo,
};

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    let app = App::new("examples/render-text", WINDOW_WIDTH, ASPECT_RATIO);

    // Load a system font

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example render-text /path/to/your-font.fon");
        return Ok(());
    }

    let font_info = FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    };

    let mut font_cache = FontCache::new(app.context.ttf_context);

    // Set up our app

    let mut graphics = Graphics {
        buffer: PixelBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    };

    let now_seconds = RefCell::new(0.0);

    let mouse_x = RefCell::new(0);
    let mouse_y = RefCell::new(0);

    let mut update = |timing_info: &TimingInfo,
                      _keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> () {
        *now_seconds.borrow_mut() += timing_info.seconds_since_last_update;
        *mouse_x.borrow_mut() = mouse_state.position.0;
        *mouse_y.borrow_mut() = mouse_state.position.1;
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Clears pixel buffer

        graphics.buffer.clear(color::BLACK);

        // Render some text to our pixel buffer

        let font = font_cache.load(&font_info).unwrap();

        graphics.text(
            &font,
            &TextOperation {
                text: &(format!("Uptime: {}s", now_seconds.borrow())),
                x: 12,
                y: 12,
                color: color::WHITE,
            },
        )?;

        let x = *mouse_x.borrow();
        let y = *mouse_y.borrow();

        graphics.crosshair(x, y, 24, 2, 6, true, color::YELLOW);

        graphics.text(
            &font,
            &TextOperation {
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

        return Ok(graphics.buffer.get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
