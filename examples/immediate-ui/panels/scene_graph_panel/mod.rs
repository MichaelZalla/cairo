use std::fmt::Debug;

use cairo::{
    mem::linked_list::LinkedList,
    scene::{graph::SceneGraph, node::SceneNode, resources::SceneResources},
    serde::PostDeserialize,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        fastpath::{
            container::{collapsible_container, container},
            text::text,
        },
        ui_box::{UIBoxFeatureFlags, UILayoutDirection, tree::UIBoxTree},
    },
};
use uuid::Uuid;

use crate::{COMMAND_BUFFER, SCENE_CONTEXT, command::PendingCommand};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct SceneGraphPanel {
    id: String,
    scene_index: usize,
    selected_node: Option<Uuid>,
}

impl Debug for SceneGraphPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneGraphPanel")
            .field("id", &self.id)
            .field("scene_index", &self.scene_index)
            .finish()
    }
}

impl PostDeserialize for SceneGraphPanel {
    fn post_deserialize(&mut self) {}
}

impl SceneGraphPanel {
    pub fn new(id: &str, scene_index: usize) -> Self {
        Self {
            id: id.to_string(),
            scene_index,
            selected_node: None,
        }
    }
}

fn render_subtree(
    node: &SceneNode,
    selected_node: &mut Option<Uuid>,
    tree: &mut UIBoxTree,
) -> Result<(), String> {
    let container_id = format!("node_{}_container", node.get_uuid());

    GLOBAL_UI_CONTEXT.with(|ctx| {
        let theme = ctx.theme.borrow();

        let text_color = match selected_node {
            Some(selected) => {
                if *selected == *node.get_uuid() {
                    Some(theme.background_selected)
                } else {
                    None
                }
            }
            None => None,
        };

        let label_box = {
            if let Some(color) = text_color {
                ctx.styles.borrow_mut().text_color.push(color);
            }

            let mut label_box = text(
                format!("node_{}_type_label", node.get_uuid()).to_string(),
                node.get_type().to_string(),
            );

            label_box.features |= UIBoxFeatureFlags::HOVERABLE | UIBoxFeatureFlags::CLICKABLE;

            if text_color.is_some() {
                ctx.styles.borrow_mut().text_color.pop();
            }

            label_box
        };

        tree.with_parent(
            container(
                container_id.to_string(),
                UILayoutDirection::LeftToRight,
                None,
            ),
            |tree| {
                let interaction_result = if node.has_children() {
                    collapsible_container(
                        format!("node_{}_container", node.get_uuid()),
                        label_box,
                        tree,
                        |tree| -> Result<(), String> {
                            if let Some(children) = node.children() {
                                for child in children {
                                    render_subtree(child, selected_node, tree)?;
                                }
                            }

                            Ok(())
                        },
                    )
                } else {
                    let node_container_box = container(
                        format!("node_{}_container", node.get_uuid()),
                        UILayoutDirection::LeftToRight,
                        None,
                    );

                    tree.with_parent(node_container_box, |tree| {
                        tree.push(label_box)?;

                        Ok(())
                    })
                }?;

                if interaction_result
                    .mouse_interaction_in_bounds
                    .was_left_pressed
                {
                    println!("UUID: {}", node.get_uuid());

                    selected_node.replace(*node.get_uuid());
                }

                Ok(())
            },
        )
        .unwrap();
    });

    Ok(())
}

impl SceneGraphPanel {
    pub fn render_for_scene(
        &mut self,
        scene: &SceneGraph,
        _resources: &SceneResources,
        tree: &mut UIBoxTree,
        _pending_queue: &mut LinkedList<PendingCommand>,
    ) -> Result<(), String> {
        render_subtree(&scene.root, &mut self.selected_node, tree)?;

        Ok(())
    }
}

impl PanelInstance for SceneGraphPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SCENE_CONTEXT.with(|ctx| -> Result<(), String> {
            let resources = &ctx.resources;
            let scenes = ctx.scenes.borrow();

            if self.scene_index < scenes.len() {
                let scene = &scenes[self.scene_index];

                COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                    let mut pending_queue = buffer.pending_commands.borrow_mut();

                    self.render_for_scene(scene, resources, tree, &mut pending_queue)
                })?;
            } else {
                panic!(
                    "Invalid scene index {} assigned to SceneGraphPanel {}!",
                    self.scene_index, self.id
                );
            }

            Ok(())
        })
    }
}
