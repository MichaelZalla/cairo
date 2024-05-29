use cairo::ui::{context::UIContext, tree::Tree, ui_box::UIBoxFeatureFlag};

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

                let is_leaf_panel = panel_tree_node.children.is_empty();

                let mut ui_box_tree = ui_context.tree.borrow_mut();

                let panel_box = panel.make_panel_box(ui_context)?;

                if is_leaf_panel {
                    let panel_interaction_result = ui_box_tree.push_parent(panel_box)?;

                    panel
                        .render_leaf_panel_contents(&mut ui_box_tree, &panel_interaction_result)?;

                    ui_box_tree.pop_parent()?;
                } else {
                    ui_box_tree.push_parent(panel_box)?;
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
