extern crate sdl2;

use std::{cell::RefCell, env, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::{cache::FontCache, FontInfo},
    ui::{
        button::{do_button, ButtonOptions},
        panel::{Panel, PanelInfo, PANEL_TITLE_BAR_HEIGHT},
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ui-panels".to_string(),
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

    let font_cache = FontCache::new(app.context.ttf_context);

    let font_cache_rwl = Box::new(RwLock::new(font_cache));

    let font_cache_box_leaked: &'static mut RwLock<FontCache<'static>> = Box::leak(font_cache_rwl);

    // Set up our app

    let mut framebuffer = Buffer2D::new(window_info.window_width, window_info.window_height, None);

    let render_rwl = RwLock::new(
        |info: &PanelInfo,
         buffer: &mut Buffer2D,
         _app: &mut App,
         _keyboard_state: &KeyboardState,
         mouse_state: &MouseState|
         -> Result<(), String> {
            buffer.clear(None);

            let button_options = ButtonOptions {
                label: format!("Button {}", info.id).to_string(),
                x_offset: 8,
                y_offset: PANEL_TITLE_BAR_HEIGHT + 8,
                with_border: true,
                ..Default::default()
            };

            if do_button(
                info,
                buffer,
                mouse_state,
                font_cache_box_leaked,
                &font_info,
                &button_options,
            ) {
                println!("You clicked Button {}!", info.id);
            }

            Ok(())
        },
    );

    let render_rwl_option = Some(&render_rwl);

    let mut root_panel = Panel::new(
        PanelInfo {
            title: "Panel 0".to_string(),
            width: window_info.window_width,
            height: window_info.window_height,
            ..Default::default()
        },
        render_rwl_option,
    );

    root_panel.split(0.5)?;

    let root_panel_rc = RefCell::new(root_panel);

    let current_mouse_state: RwLock<MouseState> = RwLock::new(Default::default());

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegrate update actions to the root panel?

        let mut root_panel = root_panel_rc.borrow_mut();

        root_panel
            .update(app, keyboard_state, mouse_state, game_controller_state)
            .unwrap();

        // Delegate render call to the root panel

        root_panel
            .render(
                app,
                keyboard_state,
                mouse_state,
                font_cache_box_leaked,
                &font_info,
            )
            .unwrap();

        // Cache the mouse state (position) so that we can render a crosshair.

        current_mouse_state.write().unwrap().position = mouse_state.position;
    };

    let mut render = || -> Result<Vec<u32>, String> {
        let fill_value = color::WHITE.to_u32();

        // Clears pixel buffer
        framebuffer.clear(Some(fill_value));

        // Blit panel pixels (local space) onto global pixels

        let root = root_panel_rc.borrow();

        framebuffer.blit_from(root.info.x, root.info.y, &root.buffer);

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
