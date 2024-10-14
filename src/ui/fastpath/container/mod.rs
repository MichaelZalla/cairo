use crate::ui::{
    context::GLOBAL_UI_CONTEXT,
    ui_box::{
        key::UIKey, tree::UIBoxTree, UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

use super::{button::button, spacer::greedy_spacer, text::text};

pub fn container(
    id: String,
    layout_direction: UILayoutDirection,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
        ],
    };

    UIBox::new(id, UIBoxFeatureMask::none(), layout_direction, sizes, None)
}

pub fn greedy_container(id: String, layout_direction: UILayoutDirection) -> UIBox {
    UIBox::new(
        id,
        UIBoxFeatureMask::none(),
        layout_direction,
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
        None,
    )
}

pub fn collapsible_container<C>(
    id: String,
    label: String,
    tree: &mut UIBoxTree,
    callback: C,
) -> Result<(), String>
where
    C: FnOnce(&mut UIBoxTree) -> Result<(), String>,
{
    let container_id = format!("{}_collapsible_container", id);

    let ui_key = UIKey::from_string(container_id.clone());

    let was_expanded = GLOBAL_UI_CONTEXT.with(|ctx| {
        let cache = ctx.cache.borrow();

        match cache.get(&ui_key) {
            Some(previous_frame) => previous_frame.expanded,
            None => false,
        }
    });

    let wrapper = container(container_id.clone(), UILayoutDirection::LeftToRight, None);

    let mut was_toggled = false;

    tree.with_parent(wrapper, |tree| -> Result<(), String> {
        let left = container(
            format!("{}_left", container_id).to_string(),
            UILayoutDirection::LeftToRight,
            Some([
                UISizeWithStrictness {
                    size: UISize::Pixels(16),
                    strictness: 1.0,
                },
                UISizeWithStrictness {
                    size: UISize::Pixels(16),
                    strictness: 1.0,
                },
            ]),
        );

        let right = container(
            format!("{}_right", container_id).to_string(),
            UILayoutDirection::TopToBottom,
            None,
        );

        tree.with_parent(left, |tree| {
            // Toggle button
            let toggle_button_label = if was_expanded { "-" } else { "+" };

            let mut toggle_button = button(
                format!("{}.toggle_button", id).to_string(),
                toggle_button_label.to_string(),
                Some([
                    UISizeWithStrictness {
                        size: UISize::PercentOfParent(1.0),
                        strictness: 1.0,
                    },
                    UISizeWithStrictness {
                        size: UISize::PercentOfParent(1.0),
                        strictness: 1.0,
                    },
                ]),
            );

            toggle_button.features ^= UIBoxFeatureFlag::EmbossAndDeboss;

            tree.push(greedy_spacer())?;

            let interaction_result = tree.push(toggle_button)?;

            tree.push(greedy_spacer())?;

            if interaction_result
                .mouse_interaction_in_bounds
                .was_left_pressed
            {
                was_toggled = true;
            }

            Ok(())
        })?;

        tree.with_parent(right, |tree| {
            // Container label
            tree.push(text(format!("{}.label", id).to_string(), label.to_string()))?;

            if was_expanded {
                callback(tree)?;
            }

            Ok(())
        })?;

        Ok(())
    })?;

    if let Some(current_rc) = tree.get_current() {
        let panel_node = current_rc.borrow_mut();

        let wrapper_node_rc = &panel_node.children[panel_node.children.len() - 1];
        let mut wrapper_node = wrapper_node_rc.borrow_mut();
        let wrapper = &mut wrapper_node.data;

        wrapper.expanded = if was_toggled {
            !was_expanded
        } else {
            was_expanded
        };
    }

    Ok(())
}
