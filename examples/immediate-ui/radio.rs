use cairo::ui::{
    context::GLOBAL_UI_CONTEXT,
    ui_box::{
        tree::UIBoxTree,
        utils::{container, spacer, text},
        UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

use crate::stack::stack;

pub struct RadioOption {
    pub label: String,
}

pub fn radio_group(
    id: String,
    options: &[RadioOption],
    selected_index: usize,
    tree: &mut UIBoxTree,
) -> Result<Option<usize>, String> {
    let mut result: Option<usize> = None;

    tree.with_parent(
        container(
            format!("{}.radio_group.container", id).to_string(),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| -> Result<(), String> {
            stack(
                options,
                |index: usize, option: &RadioOption, tree: &mut UIBoxTree| -> Result<(), String> {
                    if let Some(new_selected_index) =
                        radio(&id, index, option, index == selected_index, tree)?
                    {
                        if new_selected_index != selected_index {
                            result.replace(new_selected_index);
                        }
                    }

                    Ok(())
                },
                4,
                tree,
            )
        },
    )?;

    Ok(result)
}

pub fn radio(
    id: &String,
    index: usize,
    option: &RadioOption,
    is_selected: bool,
    tree: &mut UIBoxTree,
) -> Result<Option<usize>, String> {
    let radio_option_container = {
        let mut ui_box = container(
            format!("{}.radio_option_{}_container", id, index).to_string(),
            UILayoutDirection::LeftToRight,
            None,
        );

        ui_box.features |= UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable;

        ui_box
    };

    let was_radio_option_clicked = tree
        .with_parent(radio_option_container, |tree| {
            let radio_option_selected_indicator = {
                GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBox, String> {
                    let theme = ctx.theme.borrow();

                    let fill_color = if is_selected {
                        theme.checkbox_background_selected
                    } else {
                        theme.checkbox_background
                    };

                    ctx.fill_color(fill_color, || -> Result<UIBox, String> {
                        Ok(UIBox::new(
                            format!("{}.radio_option_{}_selected", id, index).to_string(),
                            UIBoxFeatureMask::from(UIBoxFeatureFlag::DrawFill),
                            UILayoutDirection::LeftToRight,
                            [
                                UISizeWithStrictness {
                                    size: UISize::Pixels(14),
                                    strictness: 1.0,
                                },
                                UISizeWithStrictness {
                                    size: UISize::MaxOfSiblings,
                                    strictness: 1.0,
                                },
                            ],
                            None,
                        ))
                    })
                })
            }?;

            tree.push(radio_option_selected_indicator)?;

            let radio_option_label = text(
                format!("{}.radio_option_{}_label", id, index).to_string(),
                option.label.to_string(),
            );

            tree.push(spacer(6))?;

            tree.push(radio_option_label)?;

            Ok(())
        })?
        .mouse_interaction_in_bounds
        .was_left_pressed;

    Ok(if was_radio_option_clicked {
        Some(index)
    } else {
        None
    })
}
