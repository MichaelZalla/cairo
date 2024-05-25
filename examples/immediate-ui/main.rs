extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{
        tree::{node::Node, UIWidgetTree},
        UIContext, UISize, UISizeWithStrictness, UIWidget,
    },
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

    let root_widget_node = Node::<UIWidget>::new(root_widget);

    let mut widget_tree = UIWidgetTree::new(root_widget_node);

    widget_tree.push(UIWidget::new(
        "root_child1".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(120),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(200),
                strictness: 1.0,
            },
        ],
    ));

    widget_tree.push(UIWidget::new(
        "root_child1_child1".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(80),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(50),
                strictness: 1.0,
            },
        ],
    ));

    let ui_context_rc = RefCell::new(UIContext { tree: widget_tree });

    let mut update = |_app: &mut App,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> Result<(), String> { Ok(()) };

    let mut render = || -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        {
            let mut context = ui_context_rc.borrow_mut();

            context.tree.do_autolayout_pass().unwrap();
        }

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
