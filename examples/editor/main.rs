extern crate sdl2;

use std::env;

use cairo::{
    animation::lerp,
    app::{resolution::RESOLUTION_1600_BY_900, App, AppWindowInfo},
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    font::cache::FontCache,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
        UISize, UISizeWithStrictness,
    },
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/editor".to_string(),
        window_resolution: RESOLUTION_1600_BY_900,
        canvas_resolution: RESOLUTION_1600_BY_900,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example editor /path/to/your-font.fon");
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
        let seconds_since_last_update = app.timing_info.seconds_since_last_update;

        // Recreate the UI tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let tree = &mut ctx.tree.borrow_mut();

            tree.clear();

            println!("\nRebuilding tree...\n");

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
                            ) as u32),
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(lerp(
                                window_info.window_resolution.height as f32 * 0.66,
                                window_info.window_resolution.height as f32,
                                alpha_y,
                            ) as u32),
                            strictness: 1.0,
                        },
                    ],
                ))?;

                tree.push_parent(UIBox::new(
                    "MainMenuBar__main_menu_bar".to_string(),
                    UIBoxFeatureMask::none(),
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::ChildrenSum,
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 1.0,
                        },
                    ],
                ))?;

                // Icon

                tree.push(UIBox::new(
                    "MainMenuBarLogo__main_menu_bar_logo".to_string(),
                    UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::Pixels(36),
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(36),
                            strictness: 1.0,
                        },
                    ],
                ))?;

                tree.push_parent(UIBox::new(
                    "MainMenuBarMenu__main_menu_bar_menu".to_string(),
                    UIBoxFeatureMask::none(),
                    UILayoutDirection::TopToBottom,
                    [
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 0.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::MaxOfSiblings,
                            strictness: 1.0,
                        },
                    ],
                ))?;

                // Top spacer

                tree.push(UIBox::new(
                    "MainMenuBarMenuTopSpacer__main_menu_bar_menu_top_spacer".to_string(),
                    UIBoxFeatureMask::none(),
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

                // Menu item list

                tree.push_parent(UIBox::new(
                    "MainMenuBarMenuItemList__main_menu_bar_menu_item_list".to_string(),
                    UIBoxFeatureMask::none(),
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::ChildrenSum,
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::ChildrenSum,
                            strictness: 1.0,
                        },
                    ],
                ))?;

                static MENU_BAR_ITEMS: [&str; 6] =
                    ["Project", "Scene", "Edit", "Debug", "Help", "About"];

                for (item_index, item_label) in MENU_BAR_ITEMS.iter().enumerate() {
                    // Inter-item spacer.

                    tree.push(UIBox::new(
                        "MainMenuBarMenuItemList_Spacer__menu_bar_menu_spacer".to_string(),
                        UIBoxFeatureMask::none(),
                        UILayoutDirection::LeftToRight,
                        [
                            UISizeWithStrictness {
                                size: UISize::Pixels(8),
                                strictness: 1.0,
                            },
                            UISizeWithStrictness {
                                size: UISize::MaxOfSiblings,
                                strictness: 1.0,
                            },
                        ],
                    ))?;

                    // Menu bar item (container)

                    tree.push_parent(UIBox::new(
                        format!(
                            "MainMenuBarMenuItemList_MenuItem{}__menu_bar_menu_item_{}",
                            item_index, item_index
                        ),
                        UIBoxFeatureFlag::DrawFill
                            | UIBoxFeatureFlag::Hoverable
                            | UIBoxFeatureFlag::Clickable,
                        UILayoutDirection::LeftToRight,
                        [
                            UISizeWithStrictness {
                                size: UISize::ChildrenSum,
                                strictness: 1.0,
                            },
                            UISizeWithStrictness {
                                size: UISize::ChildrenSum,
                                strictness: 1.0,
                            },
                        ],
                    ))?;

                    // Menu bar text

                    let mut text_ui_box = UIBox::new(
                        format!(
                            "MainMenuBarMenuItemList_MenuItem{}_Text__menu_bar_menu_item_{}_text",
                            item_index, item_index
                        ),
                        UIBoxFeatureFlag::DrawFill
                            | UIBoxFeatureFlag::DrawText
                            | UIBoxFeatureFlag::Hoverable
                            | UIBoxFeatureFlag::Clickable,
                        UILayoutDirection::LeftToRight,
                        [
                            UISizeWithStrictness {
                                size: UISize::TextContent,
                                strictness: 1.0,
                            },
                            UISizeWithStrictness {
                                size: UISize::TextContent,
                                strictness: 1.0,
                            },
                        ],
                    );

                    text_ui_box.text_content = Some(item_label.to_string());

                    tree.push(text_ui_box)?;

                    tree.pop_parent()?;
                }

                tree.pop_parent()?;

                // Bottom spacer

                tree.push(UIBox::new(
                    "MainMenuBarMenuBottomSpacer__main_menu_bar_menu_bottomspacer".to_string(),
                    UIBoxFeatureMask::none(),
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 0.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 1.0,
                        },
                    ],
                ))?;

                // Back to 'MainMenuBar'.

                tree.pop_parent()?;

                // Back to 'Root'.

                tree.pop_parent()
            })?;

            // `Current` is now back at the root...

            tree.do_user_inputs_pass(
                seconds_since_last_update,
                keyboard_state,
                mouse_state,
                game_controller_state,
            )?;

            tree.do_autolayout_pass()
        })
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
