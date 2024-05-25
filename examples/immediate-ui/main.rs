extern crate sdl2;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{UISize, UISizeWithStrictness, UIWidget, UIWidgetNode, UIWidgetTree},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/immediate-ui".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Set up our app

    let mut framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    // UI widget tree

    let root_widget = UIWidget::new(
        "root".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(window_info.window_resolution.width),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(window_info.window_resolution.height),
                strictness: 1.0,
            },
        ],
    );

    let root_widget_node = UIWidgetNode::new(root_widget);

    let _widget_tree = UIWidgetTree::new(root_widget_node);

    let mut update = |_app: &mut App,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> Result<(), String> { Ok(()) };

    let mut render = || -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        // Clears pixel buffer
        framebuffer.clear(Some(fill_value));

        // @TODO Write some pixel data to the pixel buffer,
        //       based on some borrowed state.

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
