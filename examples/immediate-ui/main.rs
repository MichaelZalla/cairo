extern crate sdl2;

use std::{cell::RefCell, env};

use cairo::{
    animation::lerp,
    app::{
        resolution::{Resolution, RESOLUTION_1600_BY_900},
        App, AppWindowInfo,
    },
    buffer::Buffer2D,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::cache::FontCache,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{
            tree::UIBoxTree, utils::text_box, UIBox, UIBoxFeatureFlag, UIBoxFeatureMask,
            UILayoutDirection,
        },
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

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example immediate-ui /path/to/your-font.fon");
        return Ok(());
    }

    let font_filepath = args[1].to_string();

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = ctx.font_info.borrow_mut();

            font_info.filepath = font_filepath;
            font_info.point_size = 14;
        }
    });

    // Set up our app

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let framebuffer_rc = RefCell::new(framebuffer);

    let ui_box_tree_rc = RefCell::new(UIBoxTree::default());

    let mut update = |app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      _mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;
        let _seconds_since_last_update = app.timing_info.seconds_since_last_update;

        // Recreate the UI tree.

        let mut result = Ok(());

        GLOBAL_UI_CONTEXT.with(|ctx| {
            result = ctx.fill_color(color::BLUE, || {
                ctx.border_color(color::YELLOW, || {
                    let mut tree = ui_box_tree_rc.borrow_mut();

                    tree.clear();

                    let alpha_x = uptime.sin() / 2.0 + 0.5;
                    let alpha_y = uptime.cos() / 2.0 + 0.5;

                    ctx.fill_color(color::WHITE, || {
                        tree.push_parent(UIBox::new(
                            "Root__root".to_string(),
                            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::Pixels(lerp(
                                        window_info.window_resolution.width as f32 * 0.66,
                                        window_info.window_resolution.width as f32,
                                        alpha_x,
                                    )
                                        as u32),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::Pixels(lerp(
                                        window_info.window_resolution.height as f32 * 0.66,
                                        window_info.window_resolution.height as f32,
                                        alpha_y,
                                    )
                                        as u32),
                                    strictness: 1.0,
                                },
                            ],
                        )).unwrap();

                        Ok(())
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
                        )).unwrap();

                        Ok(())
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
                        )).unwrap();

                        Ok(())
                    })?;

                    ctx.fill_color(color::BLACK, || {
                        tree.push(UIBox::new(
                            "RootChild1Spacer1__root_child1_spacer1".to_string(),
                            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::Pixels(6),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 1.0,
                                },
                            ],
                        )).unwrap();

                        Ok(())
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
                        )).unwrap();

                        Ok(())
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
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        )).unwrap();

                        Ok(())
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
                                    size: UISize::PercentOfParent(1.0 / 3.0),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        )).unwrap();

                        Ok(())
                    })?;

                    ctx.fill_color(color::SKY_BOX, || {
                        tree.push_parent(UIBox::new(
                            "RootChild2Child2__root_child2_child2".to_string(),
                            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable,
                            UILayoutDirection::TopToBottom,
                            [
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(2.0 / 3.0),
                                    strictness: 0.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::PercentOfParent(1.0),
                                    strictness: 0.0,
                                },
                            ],
                        )).unwrap();

                        Ok(())
                    })?;

                    ctx.fill_color(color::BLACK, || {
                        ctx.border_color(color::WHITE, || {
                            let child_count = 8_usize;

                            for i in 0..child_count {
                                tree.push_parent(UIBox::new(
                                    format!("RootChild2Child2Child{}__root_child2_child2_child{}", i, i),
                                    UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable,
                                    UILayoutDirection::LeftToRight,
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
                                ))?;

                                tree.push(UIBox::new(
                                    format!("RootChild2Child2Child{}SpacerBefore__root_child2_child2_child{}_spacer_before", i, i),
                                    UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                                    UILayoutDirection::TopToBottom,
                                    [
                                        UISizeWithStrictness {
                                            size: UISize::PercentOfParent(1.0),
                                            strictness: 0.0,
                                        },
                                        UISizeWithStrictness {
                                            size: UISize::Pixels(1),
                                            strictness: 1.0,
                                        },
                                    ],
                                ))?;

                                tree.push(text_box(format!("RootChild2Child2Child{}Text__root_child2_child2_child{}_text", i, i), format!("Label {}", i)))?;

                                tree.push(UIBox::new(
                                    format!("RootChild2Child2Child{}SpacerAfter__root_child2_child2_child{}_spacer_after", i, i),
                                    UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                                    UILayoutDirection::TopToBottom,
                                    [
                                        UISizeWithStrictness {
                                            size: UISize::PercentOfParent(1.0),
                                            strictness: 0.0,
                                        },
                                        UISizeWithStrictness {
                                            size: UISize::Pixels(1),
                                            strictness: 1.0,
                                        },
                                    ],
                                ))?;

                                tree.pop_parent()?;
                            }

                            Ok(())
                        })
                    })?;

                    tree.pop_parent()?;
                    tree.pop_parent()?;

                    // `Current` is now back at the root...

                    tree.do_active_focused_pass()?;

                    tree.do_autolayout_pass()
                })
            });
        });

        result
    };

    let render = |frame_index: Option<u32>, _new_resolution| -> Result<Vec<u32>, String> {
        let mut framebuffer = framebuffer_rc.borrow_mut();

        let fill_value = color::BLACK.to_u32();

        framebuffer.clear(Some(fill_value));

        let mut tree = ui_box_tree_rc.borrow_mut();

        tree.render_frame(frame_index.unwrap(), &mut framebuffer)
            .unwrap();

        Ok(framebuffer.get_all().clone())
    };

    app.run(&mut update, &render)?;

    Ok(())
}
