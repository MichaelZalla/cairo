extern crate sdl2;

use std::{cell::RefCell, env, path::Path};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::{Graphics, PixelBuffer, TextOperation},
    ui::panel::{Panel, PanelInfo},
};

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 1920;
static CANVAS_HEIGHT: u32 = (CANVAS_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    match sdl2::ttf::init() {
        Ok(ttf_context) => {
            println!("Initialized TTF font subsystem.");

            // Load a system font

            let args: Vec<String> = env::args().collect();

            if args.len() < 2 {
                println!("Usage: cargo run --example ui-panels /path/to/your-font.fon");

                return Ok(());
            }

            let font_path = Path::new(&args[1]);

            let font = ttf_context.load_font(font_path, 16)?;

            // Set up our app

            let mut graphics = Graphics {
                buffer: PixelBuffer::new(CANVAS_WIDTH, CANVAS_HEIGHT),
            };

            let root_panel = RefCell::new(Panel::new(
                PanelInfo {
                    id: 0,
                    title: "Root Panel".to_string(),
                    x: 0,
                    y: 0,
                    width: CANVAS_WIDTH,
                    height: CANVAS_HEIGHT,
                },
                |_keyboard_state: &KeyboardState,
                 _mouse_state: &MouseState,
                 _game_controller_state: &GameControllerState,
                 _seconds_since_last_update: f32|
                 -> () {
                    // @TODO(mzalla) Update panel tree in response to mouse events
                },
                |panel_graphics: &mut Graphics, info: &PanelInfo| -> Result<Vec<u32>, String> {
                    panel_graphics.text(
                        &font,
                        TextOperation {
                            text: &info.title,
                            x: 8,
                            y: 8,
                            color: color::YELLOW,
                        },
                    )?;

                    return Ok(panel_graphics.get_pixel_data().clone());
                },
            ));

            root_panel.borrow_mut().split()?;

            let mut update = |keyboard_state: &KeyboardState,
                              mouse_state: &MouseState,
                              game_controller_state: &GameControllerState,
                              seconds_since_last_update: f32|
             -> () {
                // Delegrate update actions to the root panel

                ((*root_panel.borrow_mut()).update)(
                    keyboard_state,
                    mouse_state,
                    game_controller_state,
                    seconds_since_last_update,
                )
            };

            let mut render = || -> Result<Vec<u32>, String> {
                // Clears pixel buffer
                graphics.buffer.clear(color::WHITE);

                // Delegate render call to the root panel

                let panel_pixel_data = root_panel.borrow_mut().render()?;

                let panel_info = &root_panel.borrow().info;

                // Blit panel pixels (local space) onto global pixels

                graphics.blit(
                    panel_info.x,
                    panel_info.y,
                    panel_info.width,
                    panel_info.height,
                    &panel_pixel_data,
                );

                return Ok(graphics.get_pixel_data().clone());
            };

            let app = App::new("examples/ui-panels", CANVAS_WIDTH, ASPECT_RATIO);

            app.run(&mut update, &mut render)?;
        }
        Err(e) => {
            println!("Error initializing ttf font subsystem: '{}'", e);
        }
    }

    Ok(())
}
