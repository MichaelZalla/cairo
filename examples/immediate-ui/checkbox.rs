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

pub(crate) struct Checkbox {
    pub value: String,
    pub label: String,
    pub is_checked: bool,
}

impl Checkbox {
    pub fn new(value: &str, label: &str, is_checked: bool) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            is_checked,
        }
    }
}

pub fn checkbox_group(
    id: String,
    items: &[Checkbox],
    tree: &mut UIBoxTree,
) -> Result<Vec<usize>, String> {
    let mut toggled_indices: Vec<usize> = vec![];

    tree.with_parent(
        container(
            format!("{}.checkbox_group.container", id).to_string(),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| -> Result<(), String> {
            stack(
                items,
                |index: usize, item: &Checkbox, tree: &mut UIBoxTree| -> Result<(), String> {
                    if checkbox(&id, index, item, tree)? {
                        toggled_indices.push(index);
                    }

                    Ok(())
                },
                4,
                tree,
            )
        },
    )?;

    Ok(toggled_indices)
}

pub fn checkbox(
    id: &String,
    index: usize,
    item: &Checkbox,
    tree: &mut UIBoxTree,
) -> Result<bool, String> {
    let checkbox_option_container = {
        let mut ui_box = container(
            format!("{}.checkbox_{}_container", id, index).to_string(),
            UILayoutDirection::LeftToRight,
            None,
        );

        ui_box.features |= UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable;

        ui_box
    };

    let was_checkbox_toggled = tree
        .with_parent(checkbox_option_container, |tree| {
            let checkbox_checked_indicator = {
                GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBox, String> {
                    let theme = ctx.theme.borrow();

                    let fill_color = if item.is_checked {
                        theme.checkbox_background_selected
                    } else {
                        theme.checkbox_background
                    };

                    ctx.fill_color(fill_color, || -> Result<UIBox, String> {
                        Ok(UIBox::new(
                            format!("{}.checkbox_{}_checked", id, index).to_string(),
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

            tree.push(checkbox_checked_indicator)?;

            let checkbox_label = text(
                format!("{}.checkbox_{}_label", id, index).to_string(),
                item.label.to_string(),
            );

            tree.push(spacer(6))?;

            tree.push(checkbox_label)?;

            Ok(())
        })?
        .mouse_interaction_in_bounds
        .was_left_pressed;

    Ok(was_checkbox_toggled)
}
