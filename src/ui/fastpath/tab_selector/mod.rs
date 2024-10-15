use crate::ui::{
    context::GLOBAL_UI_CONTEXT,
    ui_box::{key::UIKey, tree::UIBoxTree, UIBox, UIBoxFeatureFlag, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

use super::{button::button, container::container, spacer::greedy_spacer, text::text};

pub fn tab_selector(id: String, tabs: Vec<&str>, tree: &mut UIBoxTree) -> Result<usize, String> {
    let container_id = format!("{}_tab_selector_container", id);

    let ui_key = UIKey::from_string(container_id.clone());

    let selected_tab_index = GLOBAL_UI_CONTEXT.with(|ctx| {
        let cache = ctx.cache.borrow();

        match cache.get(&ui_key) {
            Some(previous_frame) => previous_frame.selected_item_index,
            None => 0,
        }
    });

    let mut new_selected_tab_index = selected_tab_index;

    let wrapper = container(container_id.clone(), UILayoutDirection::LeftToRight, None);

    tree.with_parent(wrapper, |tree| -> Result<(), String> {
        for (tab_index, tab) in tabs.into_iter().enumerate() {
            let tab_button = GLOBAL_UI_CONTEXT.with(|ctx| -> UIBox {
                let theme = ctx.theme.borrow();

                let fill_color = if tab_index == selected_tab_index {
                    Some(theme.checkbox_background_selected)
                } else {
                    None
                };

                if let Some(color) = fill_color {
                    ctx.styles.borrow_mut().fill_color.push(color);
                }

                let mut tab_button: UIBox = button(
                    format!("{}.tab_{}_button", id, tab_index),
                    "".to_string(),
                    Some([
                        UISizeWithStrictness {
                            size: UISize::Pixels(75),
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(20),
                            strictness: 1.0,
                        },
                    ]),
                );

                tab_button.features ^=
                    UIBoxFeatureFlag::DrawText | UIBoxFeatureFlag::EmbossAndDeboss;

                if fill_color.is_some() {
                    ctx.styles.borrow_mut().fill_color.pop();
                }

                tab_button
            });

            let interaction_result = tree.with_parent(tab_button, |tree| {
                tree.push(greedy_spacer())?;

                tree.push(text(
                    format!("{}.tab_{}_label", id, tab_index),
                    tab.to_string(),
                ))?;

                tree.push(greedy_spacer())?;

                Ok(())
            })?;

            if interaction_result
                .mouse_interaction_in_bounds
                .was_left_pressed
            {
                new_selected_tab_index = tab_index;
            }
        }

        Ok(())
    })?;

    if let Some(current_rc) = tree.get_current() {
        let panel_node = current_rc.borrow_mut();

        let wrapper_node_rc = &panel_node.children[panel_node.children.len() - 1];
        let mut wrapper_node = wrapper_node_rc.borrow_mut();
        let wrapper = &mut wrapper_node.data;

        wrapper.selected_item_index = new_selected_tab_index;
    }

    Ok(new_selected_tab_index)
}
