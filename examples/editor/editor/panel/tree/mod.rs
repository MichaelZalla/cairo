use cairo::{
    color,
    ui::{
        context::UIContext,
        tree::Tree,
        ui_box::{
            interaction::UIBoxInteraction, tree::UIBoxTree, utils::text_box, UIBox,
            UIBoxFeatureFlag,
        },
    },
};

use super::EditorPanel;

#[derive(Default, Debug, Clone)]
pub struct EditorPanelTree<'a> {
    tree: Tree<'a, EditorPanel>,
}

impl<'a> EditorPanelTree<'a> {
    pub fn with_root(root_panel: EditorPanel) -> Self {
        Self {
            tree: Tree::<'a, EditorPanel>::with_root(root_panel),
        }
    }

    pub fn push(&mut self, id: &str, mut panel: EditorPanel) -> Result<(), String> {
        if let Some(current_node_rc) = self.tree.get_current() {
            let current_node = &current_node_rc.borrow();
            let current_panel = &current_node.data;

            panel.path = format!("{} {}", current_panel.path, id);
        } else {
            panel.path = "Root".to_string();
        }

        self.tree.push(panel)?;

        Ok(())
    }

    pub fn push_parent(&mut self, id: &str, panel: EditorPanel) -> Result<(), String> {
        // println!("Pushing parent {}.", panel.path);

        self.push(id, panel)?;

        self.tree.push_parent_post();

        Ok(())
    }

    pub fn pop_parent(&mut self) -> Result<(), String> {
        self.tree.pop_parent()
    }

    pub fn render(&mut self, ui_context: &UIContext<'static>) -> Result<(), String> {
        self.tree.visit_root_dfs_mut(
            &cairo::ui::tree::node::NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, _parent_data, panel_tree_node| {
                let panel = &panel_tree_node.data;

                let is_leaf = panel_tree_node.children.is_empty();

                let mut panel_ui_box: UIBox = Default::default();

                ui_context.fill_color(color::WHITE, || {
                    ui_context.border_color(color::BLACK, || {
                        panel_ui_box = panel.render();

                        Ok(())
                    })
                })?;

                let mut ui_box_tree = ui_context.tree.borrow_mut();

                if is_leaf {
                    let id = panel_ui_box.id.clone();

                    let interaction_result = ui_box_tree.push_parent(panel_ui_box)?;

                    render_debug_interaction_result(&mut ui_box_tree, id, &interaction_result)?;

                    ui_box_tree.pop_parent()?;
                } else {
                    ui_box_tree.push_parent(panel_ui_box)?;
                };

                Ok(())
            },
            &mut || {
                let mut ui_box_tree = ui_context.tree.borrow_mut();

                if let Some(rc) = ui_box_tree.get_current() {
                    let mut ui_box_node = (*rc).borrow_mut();

                    if !ui_box_node.children.is_empty() {
                        ui_box_node.data.features |= UIBoxFeatureFlag::DrawChildDividers;
                    }
                }

                ui_box_tree.pop_parent().unwrap();
            },
        )
    }
}

fn render_debug_interaction_result(
    ui_box_tree: &mut UIBoxTree,
    id: String,
    interaction_result: &UIBoxInteraction,
) -> Result<(), String> {
    // Push some text describing this leaf panel's interaction.

    let mouse_result = &interaction_result.mouse_interaction_in_bounds;

    ui_box_tree.push(text_box(
        format!("{}_is_hovering", id),
        format!("is_hovering: {}", mouse_result.is_hovering),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_left_pressed", id),
        format!("was_left_pressed: {}", mouse_result.was_left_pressed),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_is_left_down", id),
        format!("is_left_down: {}", mouse_result.is_left_down),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_left_released", id),
        format!("was_left_released: {}", mouse_result.was_left_released),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_middle_pressed", id),
        format!("was_middle_pressed: {}", mouse_result.was_middle_pressed),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_is_middle_down", id),
        format!("is_middle_down: {}", mouse_result.is_middle_down),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_middle_released", id),
        format!("was_middle_released: {}", mouse_result.was_middle_released),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_right_pressed", id),
        format!("was_right_pressed: {}", mouse_result.was_right_pressed),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_is_right_down", id),
        format!("is_right_down: {}", mouse_result.is_right_down),
    ))?;

    ui_box_tree.push(text_box(
        format!("{}_was_right_released", id),
        format!("was_right_released: {}", mouse_result.was_right_released),
    ))?;

    Ok(())
}
