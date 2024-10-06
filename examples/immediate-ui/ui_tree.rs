use cairo::{
    app::resolution::Resolution,
    color,
    ui::{
        context::UIContext,
        ui_box::{
            tree::UIBoxTree,
            utils::{button_box, text_box},
            UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
        },
        UISize, UISizeWithStrictness,
    },
};

pub(crate) fn build_ui_tree(
    ctx: &UIContext<'static>,
    tree: &mut UIBoxTree,
    resolution: Resolution,
) -> Result<(), String> {
    ctx.fill_color(color::WHITE, || {
        tree.push_parent(UIBox::new(
            "Root".to_string(),
            UIBoxFeatureMask::none() | UIBoxFeatureFlag::DrawFill,
            UILayoutDirection::TopToBottom,
            [
                UISizeWithStrictness {
                    size: UISize::Pixels(resolution.width),
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::Pixels(resolution.height),
                    strictness: 1.0,
                },
            ],
            None,
        ))
        .unwrap();

        Ok(())
    })?;

    // Sample text.

    tree.push(text_box("Text_0".to_string(), "Sample label".to_string()))?;

    // Sample button.

    tree.push(button_box(
        "Button_0".to_string(),
        "Sample button".to_string(),
        None,
    ))?;

    Ok(())
}
