use cairo::ui::{context::UIContext, tree::Tree};

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
            panel.path = "Main".to_string();
        }

        self.tree.push(panel)?;

        // Reconcile child widths.

        if let Some(current_node_rc) = self.tree.get_current() {
            let current_node = &mut current_node_rc.borrow_mut();
            let current_node_child_count = current_node.children.len();

            for child in &mut current_node.children {
                child.borrow_mut().data.alpha_split = 1.0 / current_node_child_count as f32;
            }
        }

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
            &mut |_depth, _parent_data, node| {
                let panel = &node.data;

                let panel_ui_box = panel.render();

                let mut ui_box_tree = ui_context.tree.borrow_mut();

                if node.children.is_empty() {
                    ui_box_tree.push(panel_ui_box)
                } else {
                    ui_box_tree.push_parent(panel_ui_box)
                }
            },
            &mut || {
                let mut ui_box_tree = ui_context.tree.borrow_mut();

                ui_box_tree.pop_parent().unwrap();
            },
        )
    }
}
