extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    app::{App, AppWindowInfo},
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{pixelbuffer::PixelBuffer, text::TextOperation, Graphics},
    time::TimingInfo,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/render-text".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

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

    let mut framebuffer = PixelBuffer::new(window_info.window_width, window_info.window_height);

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

        framebuffer.clear(color::BLACK);

        // Render some text to our pixel buffer

        let font = font_cache.load(&font_info).unwrap();

        Graphics::text(
            &mut framebuffer,
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

        Graphics::crosshair(&mut framebuffer, x, y, 24, 2, 6, true, color::YELLOW);

        let framebuffer_height = framebuffer.height;

        Graphics::text(
            &mut framebuffer,
            &font,
            &TextOperation {
                text: &(format!(
                    "Mouse position: ({},{})",
                    mouse_x.borrow(),
                    mouse_y.borrow()
                )),
                x: 12,
                y: framebuffer_height - 12 - 16,
                color: color::YELLOW,
            },
        )?;

        return Ok(framebuffer.get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
