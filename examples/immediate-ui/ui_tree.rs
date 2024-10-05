use cairo::{
    animation::lerp,
    app::AppWindowInfo,
    color,
    ui::{
        context::UIContext,
        ui_box::{
            tree::UIBoxTree, utils::text_box, UIBox, UIBoxFeatureFlag, UIBoxFeatureMask,
            UILayoutDirection,
        },
        UISize, UISizeWithStrictness,
    },
};

pub(crate) fn build_ui_tree(
    ctx: &UIContext<'static>,
    tree: &mut UIBoxTree,
    window_info: &AppWindowInfo,
    uptime: f32,
) -> Result<(), String> {
    let alpha_x = uptime.sin() / 2.0 + 0.5;
    let alpha_y = uptime.cos() / 2.0 + 0.5;

    ctx.fill_color(color::WHITE, || {
        tree.push_parent(UIBox::new(
            "Root".to_string(),
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::GREEN, || {
        tree.push_parent(UIBox::new(
            "RootChild1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::ORANGE, || {
        tree.push(UIBox::new(
            "RootChild1Child1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::BLACK, || {
        tree.push(UIBox::new(
            "RootChild1Spacer1".to_string(),
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::SKY_BOX, || {
        tree.push(UIBox::new(
            "RootChild1Child2".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    tree.pop_parent()?;

    // `Current` is now back at the root...

    ctx.fill_color(color::GREEN, || {
        tree.push_parent(UIBox::new(
            "RootChild2".to_string(),
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::ORANGE, || {
        tree.push(UIBox::new(
            "RootChild2Child1".to_string(),
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable,
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::SKY_BOX, || {
        tree.push_parent(UIBox::new(
            "RootChild2Child2".to_string(),
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
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    ctx.fill_color(color::BLACK, || {
        ctx.border_color(color::WHITE, || {
            let child_count = 8_usize;

            for i in 0..child_count {
                tree.push_parent(UIBox::new(
                    format!("RootChild2Child2Child{}", i),
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
                    None,
                ))?;

                tree.push(UIBox::new(
                    format!("RootChild2Child2Child{}SpacerBefore", i),
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
                    None,
                ))?;

                tree.push(text_box(
                    format!("RootChild2Child2Child{}Text", i),
                    format!("Label {}", i),
                ))?;

                tree.push(UIBox::new(
                    format!("RootChild2Child2Child{}SpacerAfter", i),
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
                    None,
                ))?;

                tree.pop_parent()?;
            }

            Ok(())
        })
    })?;

    tree.pop_parent()?;
    tree.pop_parent()?;

    // `current` is now back at the root of our tree.

    Ok(())
}
