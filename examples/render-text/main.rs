extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color::{self, Color},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{
        text::{cache::TextCache, TextOperation},
        Graphics,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/render-text".to_string(),
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

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

    let font_cache_rc = RefCell::new(FontCache::new(app.context.ttf_context));

    let _text_cache_rc = RefCell::new(TextCache::new());

    // Set up our app

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let framebuffer_rc = RefCell::new(framebuffer);

    let now_seconds = RefCell::new(0.0);

    let mouse_x = RefCell::new(0);
    let mouse_y = RefCell::new(0);

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        *now_seconds.borrow_mut() += app.timing_info.seconds_since_last_update;

        *mouse_x.borrow_mut() = mouse_state.position.0;
        *mouse_y.borrow_mut() = mouse_state.position.1;

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();
        let mut font_cache = font_cache_rc.borrow_mut();

        // Clears pixel buffer

        framebuffer.clear(None);

        // Render some text to our pixel buffer

        Graphics::text(
            &mut framebuffer,
            &mut font_cache,
            None,
            &font_info,
            &TextOperation {
                text: &(format!("Uptime: {}s", now_seconds.borrow())),
                x: 12,
                y: 12,
                color: color::WHITE,
            },
        )?;

        let x = *mouse_x.borrow();
        let y = *mouse_y.borrow();

        crosshair(&mut framebuffer, x, y, 24, 2, 6, true, color::YELLOW);

        let framebuffer_height = framebuffer.height;

        Graphics::text(
            &mut framebuffer,
            &mut font_cache,
            None,
            &font_info,
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

        framebuffer.copy_to(canvas);

        Ok(())
    };

    app.run(&mut update, &render)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn crosshair(
    buffer: &mut Buffer2D,
    x: i32,
    y: i32,
    length: u16,
    thickness: u16,
    mut gap: u16,
    center_dot: bool,
    color: Color,
) {
    gap = gap.min((length as f32 / 2.0).ceil() as u16);

    let color_u32 = color.to_u32();

    for i in 0..thickness {
        let offset_x = x - (thickness as f32 / 2.0).ceil() as i32 + i as i32;
        let offset_y = y - (thickness as f32 / 2.0).ceil() as i32 + i as i32;

        // Horizontal segments
        Graphics::line(
            buffer,
            x - (length as f32 / 2.0).ceil() as i32,
            offset_y,
            x - gap as i32,
            offset_y,
            color_u32,
        );

        Graphics::line(
            buffer,
            x + gap as i32,
            offset_y,
            x + (length as f32 / 2.0).ceil() as i32,
            offset_y,
            color_u32,
        );

        // Vertical segments

        Graphics::line(
            buffer,
            offset_x,
            y - (length as f32 / 2.0).ceil() as i32,
            offset_x,
            y - gap as i32,
            color_u32,
        );

        Graphics::line(
            buffer,
            offset_x,
            y + gap as i32,
            offset_x,
            y + (length as f32 / 2.0).ceil() as i32,
            color_u32,
        );

        // Center dot

        if center_dot {
            Graphics::line(
                buffer,
                x - (thickness as f32 / 2.0).ceil() as i32,
                offset_y,
                x + (thickness as f32 / 2.0).ceil() as i32,
                offset_y,
                color_u32,
            );
        }
    }
}
