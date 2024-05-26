extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    debug::println_indent,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{
        tree::{
            node::{Node, NodeLocalTraversalMethod},
            UIWidgetTree,
        },
        widget::UIWidget,
        UIContext, UISize, UISizeWithStrictness,
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
        "Root__root".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(512),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
        ],
    );

    let root_widget_node = Node::<UIWidget>::new(root_widget);

    let mut widget_tree = UIWidgetTree::new(root_widget_node);

    // @TODO(mzalla) Move these push() and pop() calls into the main render loop.

    widget_tree.push(UIWidget::new(
        "RootChild1__root_child1".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(128),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(256),
                strictness: 1.0,
            },
        ],
    ))?;

    widget_tree.push(UIWidget::new(
        "RootChild1Child1__root_child1_child1".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
        ],
    ))?;

    widget_tree.pop_current()?;

    widget_tree.push(UIWidget::new(
        "RootChild1Child2__root_child1_child2".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
        ],
    ))?;

    widget_tree.pop_current()?;
    widget_tree.pop_current()?;

    widget_tree.push(UIWidget::new(
        "RootChild2__root_child2".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(128),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(256),
                strictness: 1.0,
            },
        ],
    ))?;

    widget_tree.push(UIWidget::new(
        "RootChild2Child1__root_child2_child1".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
        ],
    ))?;

    widget_tree.pop_current()?;

    widget_tree.push(UIWidget::new(
        "RootChild2Child2__root_child2_child2".to_string(),
        [
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::Pixels(1000),
                strictness: 0.0,
            },
        ],
    ))?;

    let ui_context_rc = RefCell::new(UIContext { tree: widget_tree });

    {
        let context = ui_context_rc.borrow_mut();

        context.tree.visit_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth: usize, parent_data, node| {
                println_indent(
                    depth,
                    format!(
                        "{}, parent: {}",
                        node.data.id,
                        match parent_data {
                            Some(data) => {
                                data.id.to_string()
                            }
                            None => {
                                "None".to_string()
                            }
                        },
                    ),
                );

                Ok(())
            },
        )?;
    }

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

            context.tree.render(&mut framebuffer).unwrap();
        }

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
