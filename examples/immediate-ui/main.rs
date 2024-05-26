extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{
        tree::UIBoxTree,
        ui_box::{UIBox, UIBoxFeatureFlag},
        UISize, UISizeWithStrictness,
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

    let ui_box_tree_rc: RefCell<UIBoxTree> = Default::default();

    let mut update = |_app: &mut App,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> Result<(), String> { Ok(()) };

    let mut render = |frame_index: u32| -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        // Recreate the UI tree.

        let mut tree = ui_box_tree_rc.borrow_mut();

        tree.clear();

        tree.push_parent(UIBox::new(
            "Root__root".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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
        ))?;

        tree.push_parent(UIBox::new(
            "RootChild1__root_child1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.push(UIBox::new(
            "RootChild1Child1__root_child1_child1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.push(UIBox::new(
            "RootChild1Child2__root_child1_child2".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.pop_parent()?;

        // `Current` is now back at the root...

        tree.push_parent(UIBox::new(
            "RootChild2__root_child2".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.push(UIBox::new(
            "RootChild2Child1__root_child2_child1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.push(UIBox::new(
            "RootChild2Child2__root_child2_child2".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::DrawBorder,
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

        tree.pop_parent()?;

        // `Current` is now back at the root...

        tree.do_autolayout_pass().unwrap();

        tree.render(frame_index, &mut framebuffer).unwrap();

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
