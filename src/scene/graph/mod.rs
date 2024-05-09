use std::fmt::{Display, Error};

use serde::{Deserialize, Serialize};

use crate::{matrix::Mat4, pipeline::Pipeline, resource::handle::Handle, serde::PostDeserialize};

use super::{
    node::{
        SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
    },
    resources::SceneResources,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SceneGraph {
    pub root: SceneNode,
}

impl PostDeserialize for SceneGraph {
    fn post_deserialize(&mut self) {
        self.root.post_deserialize();
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            root: SceneNode::new(SceneNodeType::Scene, Default::default(), None),
        }
    }

    pub fn render(
        &self,
        resources: &SceneResources,
        pipeline: &mut Pipeline,
    ) -> Result<(), String> {
        // Begin frame

        pipeline.begin_frame();

        // Render scene.

        let mut active_camera_handle: Option<Handle> = None;
        let mut active_skybox_handle: Option<Handle> = None;

        let mut render_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        active_camera_handle = Some(*handle);

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `Camera` node with no resource handle!")
                    }
                },
                SceneNodeType::Skybox => {
                    match handle {
                        Some(handle) => {
                            active_skybox_handle = Some(*handle);
                        }
                        None => {
                            panic!("Encountered a `Skybox` node with no resource handle!")
                        }
                    }

                    Ok(())
                }
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mesh_arena = resources.mesh.borrow();
                        let entity_arena = resources.entity.borrow();

                        match entity_arena.get(handle) {
                            Ok(entry) => {
                                let entity = &entry.item;

                                pipeline.render_entity(
                                    entity,
                                    &current_world_transform,
                                    &mesh_arena,
                                );

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get Entity from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `Entity` node with no resource handle!")
                    }
                },
                _ => Ok(()),
            }
        };

        // Traverse the scene graph and render its nodes.

        self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_scene_graph_node,
        )?;

        // End frame

        if let (Some(camera_handle), Some(skybox_handle)) =
            (active_camera_handle, active_skybox_handle)
        {
            if let (Ok(camera_entry), Ok(skybox_entry)) = (
                resources.camera.borrow().get(&camera_handle),
                resources.cubemap_vec3.borrow().get(&skybox_handle),
            ) {
                let camera = &camera_entry.item;
                let skybox_hdr = &skybox_entry.item;

                pipeline.render_skybox_hdr(skybox_hdr, camera);
            }
        }

        pipeline.end_frame();

        Ok(())
    }
}

impl Display for SceneGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_node_to_formatter = |current_depth: usize,
                                           _world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            match write!(
                f,
                "{}{}",
                "   ".repeat((current_depth as i8 - 1).max(0) as usize),
                if current_depth > 0 { "|- " } else { "" }
            ) {
                Ok(()) => (),
                Err(err) => return Err(err.to_string()),
            }

            match writeln!(f, "{}", node) {
                Ok(()) => Ok(()),
                Err(err) => Err(err.to_string()),
            }
        };

        match self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PreOrder),
            &mut write_node_to_formatter,
        ) {
            Ok(()) => Ok(()),
            Err(_err) => Err(Error),
        }
    }
}
