extern crate sdl2;

use cairo::{
    animation::lerp,
    app::{resolution::RESOLUTION_1600_BY_900, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
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

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;

        // Recreate the UI tree.

        let mut result = Ok(());

        GLOBAL_UI_CONTEXT.with(|ctx| {
            result = ctx.fill_color(color::BLUE, || {
                ctx.border_color(color::YELLOW, || {
                    let tree = &mut ctx.tree.borrow_mut();

                    tree.clear();

                    ctx.fill_color(color::WHITE, || {
                        tree.push_parent(UIBox::new(
                            "Root__root".to_string(),
                            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::Pixels(lerp(
                                        512.0,
                                        768.0,
                                        uptime.sin() / 2.0 + 1.0,
                                    )
                                        as u32),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::Pixels(lerp(
                                        378.0,
                                        512.0,
                                        uptime.sin() / 2.0 + 1.0,
                                    )
                                        as u32),
                                    strictness: 1.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::GREEN, || {
                        tree.push_parent(UIBox::new(
                            "RootChild1__root_child1".to_string(),
                            UIBoxFeatureFlag::DrawFill
                                | UIBoxFeatureFlag::Hoverable
                                | UIBoxFeatureFlag::Clickable,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::Pixels(128),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::Pixels(128),
                                    strictness: 1.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::ORANGE, || {
                        tree.push(UIBox::new(
                            "RootChild1Child1__root_child1_child1".to_string(),
                            UIBoxFeatureFlag::DrawFill
                                | UIBoxFeatureFlag::Hoverable
                                | UIBoxFeatureFlag::Clickable,
                            UILayoutDirection::TopToBottom,
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
                        ))
                    })?;

                    ctx.fill_color(color::BLACK, || {
                        tree.push(UIBox::new(
                            "RootChild1Child1__root_child1_spacer1".to_string(),
                            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::Pixels(6),
                                    strictness: 1.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::SKY_BOX, || {
                        tree.push(UIBox::new(
                            "RootChild1Child2__root_child1_child2".to_string(),
                            UIBoxFeatureFlag::DrawFill
                                | UIBoxFeatureFlag::Hoverable
                                | UIBoxFeatureFlag::Clickable,
                            UILayoutDirection::TopToBottom,
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
                        ))
                    })?;

                    tree.pop_parent()?;

                    // `Current` is now back at the root...

                    ctx.fill_color(color::GREEN, || {
                        tree.push_parent(UIBox::new(
                            "RootChild2__root_child2".to_string(),
                            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                            UILayoutDirection::LeftToRight,
                            [
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(0.66),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::ORANGE, || {
                        tree.push(UIBox::new(
                            "RootChild2Child1__root_child2_child1".to_string(),
                            UIBoxFeatureFlag::DrawFill
                                | UIBoxFeatureFlag::Hoverable
                                | UIBoxFeatureFlag::Clickable,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::SKY_BOX, || {
                        tree.push_parent(UIBox::new(
                            "RootChild2Child2__root_child2_child2".to_string(),
                            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        ))
                    })?;

                    ctx.fill_color(color::BLACK, || {
                        ctx.border_color(color::WHITE, || {
                            let child_count = 8_usize;

                            for i in 0..child_count {
                                let node = UIBox::new(
                                    format!("RootChild2Child2__root_child2_child2_child{}", i),
                                    UIBoxFeatureFlag::DrawFill
                                        | UIBoxFeatureFlag::DrawBorder
                                        | UIBoxFeatureFlag::Hoverable
                                        | UIBoxFeatureFlag::Clickable,
                                    UILayoutDirection::TopToBottom,
                                    [
                                        UISizeWithStrictness {
                                            size: UISize::PercentOfParent(1.0),
                                            strictness: 0.0,
                                        },
                                        UISizeWithStrictness {
                                            size: UISize::PercentOfParent(1.0),
                                            strictness: 0.0,
                                        },
                                    ],
                                );

                                tree.push(node)?;
                            }

                            Ok(())
                        })
                    })?;

                    tree.pop_parent()?;
                    tree.pop_parent()?;

                    // `Current` is now back at the root...

                    tree.do_user_inputs_pass(keyboard_state, mouse_state, game_controller_state)?;

                    tree.do_autolayout_pass()
                })
            });
        });

        result
    };

    let mut render = |frame_index: u32| -> Result<Vec<u32>, String> {
        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let tree = &mut ctx.tree.borrow_mut();

            tree.render(frame_index, &mut framebuffer).unwrap();
        });

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
