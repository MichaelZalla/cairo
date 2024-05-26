extern crate sdl2;

use cairo::{
    app::{resolution::RESOLUTION_1600_BY_900, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBox, UIBoxFeatureFlag},
        UISize, UISizeWithStrictness,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/immediate-ui".to_string(),
        window_resolution: RESOLUTION_1600_BY_900,
        canvas_resolution: RESOLUTION_1600_BY_900,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Set up our app

    let mut framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let mut update = |_app: &mut App,
                      _keyboard_state: &KeyboardState,
                      _mouse_state: &MouseState,
                      _game_controller_state: &GameControllerState|
     -> Result<(), String> { Ok(()) };

    let mut render = |frame_index: u32| -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        // Recreate the UI tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            ctx.fill_color(color::BLUE, || {
                ctx.border_color(color::YELLOW, || {
                    let tree = &mut ctx.tree.borrow_mut();

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

                    tree.do_autolayout_pass()
                })
            })
            .unwrap();
        });

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let tree = &mut ctx.tree.borrow_mut();

            tree.render(frame_index, &mut framebuffer).unwrap();
        });

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
