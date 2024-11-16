use core::fmt;

use serde::{Deserialize, Serialize};

use sdl2::mouse::MouseButton;

use crate::{
    collections::tree::{node::NodeLocalTraversalMethod, Tree},
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{UIBoxDragHandle, UIBoxFeatureFlag},
        window::Window,
    },
};

use super::Panel;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PanelTree<'a> {
    #[serde(flatten)]
    tree: Tree<'a, Panel>,
}

impl<'a> fmt::Display for PanelTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

impl<'a> PanelTree<'a> {
    pub fn from_id(id: &String) -> Self {
        Self::with_root(Panel {
            path: format!("{}_panel_tree_root", id),
            ..Default::default()
        })
    }

    pub fn with_root(root_panel: Panel) -> Self {
        Self {
            tree: Tree::<'a, Panel>::with_root(root_panel),
        }
    }

    pub fn push(&mut self, id: &str, mut panel: Panel) -> Result<(), String> {
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

    pub fn push_parent(&mut self, id: &str, panel: Panel) -> Result<(), String> {
        self.push(id, panel)?;

        self.tree.push_parent_post();

        Ok(())
    }

    pub fn pop_parent(&mut self) -> Result<(), String> {
        self.tree.pop_parent()
    }

    pub fn render(&mut self, window: &Window) -> Result<(), String> {
        let base_tree = &window.ui_trees.base;

        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, sibling_index, parent_data, panel_tree_node| {
                let panel = &mut panel_tree_node.data;

                let is_leaf_panel = panel_tree_node.children.is_empty();

                let mut ui_box_tree = base_tree.borrow_mut();

                let mut panel_box = panel.make_panel_box(window)?;

                if let Some(parent) = parent_data {
                    if parent.resizable && sibling_index != 0 {
                        panel_box.features |= UIBoxFeatureFlag::ResizableMinExtentOnPrimaryAxis;
                    }
                }

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
            &mut |node| {
                let mut ui_box_tree = base_tree.borrow_mut();

                if let Some(rc) = ui_box_tree.get_current() {
                    let mut ui_box_node = (*rc).borrow_mut();

                    if !ui_box_node.children.is_empty() {
                        // Enable child dividers.
                        ui_box_node.data.features |= UIBoxFeatureFlag::DrawChildDividers;

                        // Check each child for an active drag handle.
                        let mut resized_child: Option<(usize, UIBoxDragHandle)> = None;

                        for (index, child) in ui_box_node.children.iter().enumerate() {
                            let child_panel = &(*child).borrow().data;

                            if let Some(handle) = &child_panel.active_drag_handle {
                                resized_child.replace((index, *handle));
                            }
                        }

                        if let Some((resized_child_index, drag_handle)) = resized_child {
                            GLOBAL_UI_CONTEXT.with(|ctx| {
                                let mouse = &ctx.input_events.borrow().mouse;
                                let cache = &ctx.cache.borrow();

                                if let Some(drag_event) = mouse.drag_events.get(&MouseButton::Left)
                                {
                                    let pixel_delta = drag_event.delta;

                                    let prev_frame_parent =
                                        cache.get(&ui_box_node.data.key).unwrap();

                                    let prev_frame_parent_size = match drag_handle {
                                        UIBoxDragHandle::Left | UIBoxDragHandle::Right => {
                                            prev_frame_parent.global_bounds.right
                                                - prev_frame_parent.global_bounds.left
                                        }
                                        UIBoxDragHandle::Top | UIBoxDragHandle::Bottom => {
                                            prev_frame_parent.global_bounds.bottom
                                                - prev_frame_parent.global_bounds.top
                                        }
                                    };

                                    let drag_pixels = match drag_handle {
                                        UIBoxDragHandle::Left | UIBoxDragHandle::Right => {
                                            pixel_delta.0 as f32
                                        }
                                        UIBoxDragHandle::Top | UIBoxDragHandle::Bottom => {
                                            pixel_delta.1 as f32
                                        }
                                    };

                                    let drag_alpha = drag_pixels / prev_frame_parent_size as f32;

                                    let (take, give) =
                                        (resized_child_index, resized_child_index - 1);

                                    (*node.children[give]).borrow_mut().data.alpha_split +=
                                        drag_alpha;

                                    (*node.children[take]).borrow_mut().data.alpha_split -=
                                        drag_alpha;
                                }
                            });
                        }
                    }
                }

                ui_box_tree.pop_parent().unwrap();
            },
        )
    }
}
