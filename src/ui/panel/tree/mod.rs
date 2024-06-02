use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::ui::{
    context::UIContext,
    tree::{node::NodeLocalTraversalMethod, Tree},
    ui_box::UIBoxFeatureFlag,
    window::Window,
};

use super::Panel;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PanelTree<'a, T: Clone + Default + std::fmt::Debug + fmt::Display> {
    #[serde(flatten)]
    tree: Tree<'a, Panel<T>>,
}

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> fmt::Display
    for PanelTree<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> PanelTree<'a, T> {
    pub fn with_root(root_panel: Panel<T>) -> Self {
        Self {
            tree: Tree::<'a, Panel<T>>::with_root(root_panel),
        }
    }

    pub fn push(&mut self, id: &str, mut panel: Panel<T>) -> Result<(), String> {
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

    pub fn push_parent(&mut self, id: &str, panel: Panel<T>) -> Result<(), String> {
        self.push(id, panel)?;

        self.tree.push_parent_post();

        Ok(())
    }

    pub fn pop_parent(&mut self) -> Result<(), String> {
        self.tree.pop_parent()
    }

    pub fn render(
        &mut self,
        ui_context: &UIContext<'static>,
        window: &Window<T>,
    ) -> Result<(), String> {
        let base_tree = &window.ui_trees.base;

        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, _parent_data, panel_tree_node| {
                let panel = &panel_tree_node.data;

                let is_leaf_panel = panel_tree_node.children.is_empty();

                let mut ui_box_tree = base_tree.borrow_mut();

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
                let mut ui_box_tree = base_tree.borrow_mut();

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
