extern crate sdl2;

use std::{cell::RefCell, env, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::{pixelbuffer::PixelBuffer, text::TextOperation, Graphics},
    time::TimingInfo,
    ui::panel::{Panel, PanelInfo},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ui-panels".to_string(),
        show_cursor: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Load a system font

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example ui-panels /path/to/your-font.fon");

        return Ok(());
    }

    let font_info = FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    };

    let mut font_cache = FontCache::new(app.context.ttf_context);

    // Set up our app

    let mut graphics = Graphics {
        buffer: PixelBuffer::new(window_info.window_width, window_info.window_height),
    };

    let root_panel = RefCell::new(Panel::new(
        PanelInfo {
            id: 0,
            title: "Root Panel".to_string(),
            x: 0,
            y: 0,
            width: window_info.window_width,
            height: window_info.window_height,
        },
        |_timing_info: &TimingInfo,
         _keyboard_state: &KeyboardState,
         _mouse_state: &MouseState,
         _game_controller_state: &GameControllerState|
         -> () {
            // @TODO(mzalla) Update panel tree in response to mouse events
        },
        |panel_graphics: &mut Graphics, info: &PanelInfo| -> Result<Vec<u32>, String> {
            let font = font_cache.load(&font_info).unwrap();

            panel_graphics.text(
                &font,
                &TextOperation {
                    text: &info.title,
                    x: 8,
                    y: 8,
                    color: color::YELLOW,
                },
            )?;

            return Ok(panel_graphics.buffer.get_pixel_data().clone());
        },
    ));

    let current_mouse_state: RwLock<MouseState> = RwLock::new(Default::default());

    root_panel.borrow_mut().split()?;

    let mut update = |timing_info: &TimingInfo,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegrate update actions to the root panel

        ((*root_panel.borrow_mut()).update)(
            timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        // Cache the mouse state (position) so that we can render a crosshair.

        current_mouse_state.write().unwrap().position = mouse_state.position;
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Clears pixel buffer
        graphics.buffer.clear(color::WHITE);

        // Delegate render call to the root panel

        let panel_pixel_data = root_panel.borrow_mut().render()?;

        let panel_info = &root_panel.borrow().info;

        // Blit panel pixels (local space) onto global pixels

        graphics.buffer.blit(
            panel_info.x,
            panel_info.y,
            panel_info.width,
            panel_info.height,
            &panel_pixel_data,
        );

        // Render a custom crosshair

        let mouse_state = current_mouse_state.read().unwrap();

        let (x, y) = (mouse_state.position.0, mouse_state.position.1);

        graphics.crosshair(x, y, 24, 2, 6, true, color::YELLOW);

        return Ok(graphics.buffer.get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
