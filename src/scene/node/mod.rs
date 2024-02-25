use std::{collections::VecDeque, fmt::Display};

use crate::{matrix::Mat4, resource::handle::Handle, transform::Transform3D};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum SceneNodeType {
    #[default]
    Scene,
    Environment,
    AmbientLight,
    DirectionalLight,
    Camera,
    PointLight,
    SpotLight,
    Entity,
}

impl Display for SceneNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SceneNodeGlobalTraversalMethod {
    #[default]
    DepthFirst,
    BreadthFirst,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SceneNodeLocalTraversalMethod {
    #[default]
    PreOrder,
    PostOrder,
}

#[derive(Default, Debug, Clone)]
pub struct SceneNode<'a> {
    node_type: SceneNodeType,
    transform: Transform3D,
    handle: Option<Handle>,
    parent: Option<&'a SceneNode<'a>>,
    children: Option<Vec<SceneNode<'a>>>,
}

impl<'a> SceneNode<'a> {
    pub fn new(
        node_type: SceneNodeType,
        transform: Transform3D,
        handle: Option<Handle>,
        parent: Option<&'a SceneNode<'a>>,
    ) -> Self {
        Self {
            node_type,
            transform,
            handle,
            parent,
            children: None,
        }
    }

    pub fn is_type(&self, node_type: SceneNodeType) -> bool {
        self.node_type == node_type
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn is_scene_root(&self) -> bool {
        self.is_type(SceneNodeType::Scene)
    }

    pub fn get_type(&self) -> &SceneNodeType {
        &self.node_type
    }

    pub fn get_transform(&self) -> &Transform3D {
        &self.transform
    }

    pub fn get_transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    pub fn get_handle(&self) -> &Option<Handle> {
        &self.handle
    }

    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }

    pub fn children(&self) -> &Option<Vec<SceneNode<'a>>> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Option<Vec<SceneNode<'a>>> {
        &mut self.children
    }

    pub fn add_child(&mut self, node: SceneNode<'a>) -> Result<(), String> {
        match node.node_type {
            SceneNodeType::Scene => {
                return Err("Scene node must be the root node.".to_string());
            }
            SceneNodeType::Environment => {
                // Environment node may only be a child of a Scene node.
                if !self.is_scene_root() {
                    return Err("Attempted to add an Environment node as a child to a node that is not a Scene node!".to_string());
                }
            }

            SceneNodeType::AmbientLight => {
                // AmbientLight node may only be a child of an Environment node.
                if !self.is_type(SceneNodeType::Environment) {
                    return Err("Attempted to add an AmbientLight node as a child to a node that is not an Environment node!".to_string());
                }

                // Only one AmbientLight node may exist per scene (environment) at a time.
                match self.children() {
                    Some(children) => {
                        if children
                            .iter()
                            .any(|child| child.is_type(SceneNodeType::AmbientLight))
                        {
                            return Err(
                                "Cannot add multiple AmbientLight nodes to an Environment node!"
                                    .to_string(),
                            );
                        }
                    }
                    None => (),
                }
            }
            SceneNodeType::DirectionalLight => {
                // DirectionalLight node may only be a child of an Environment node.
                if !self.is_type(SceneNodeType::Environment) {
                    return Err("Attempted to add an DirectionalLight node as a child to a node that is not an Environment node!".to_string());
                }

                // Only one DirectionalLight node may exist per scene (environment) at a time.
                match self.children() {
                    Some(children) => {
                        if children
                            .iter()
                            .any(|child| child.is_type(SceneNodeType::DirectionalLight))
                        {
                            return Err(
                                "Cannot add multiple DirectionalLight nodes to an Environment node!"
                                    .to_string(),
                            );
                        }
                    }
                    None => (),
                }
            }
            _ => (),
        }

        match self.children.as_mut() {
            Some(children) => {
                children.push(node);
            }
            None => {
                self.children = Some(vec![node]);
            }
        }

        Ok(())
    }

    pub fn visit<C>(
        &self,
        global_method: SceneNodeGlobalTraversalMethod,
        local_method: Option<SceneNodeLocalTraversalMethod>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let local = match local_method {
            Some(method) => method,
            None => Default::default(),
        };

        let current_depth: usize = 0;
        let parent_world_transform = Mat4::identity();

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => {
                self.visit_dfs(&local, current_depth, parent_world_transform, visit_action)
            }
            SceneNodeGlobalTraversalMethod::BreadthFirst => self.visit_bfs(visit_action),
        }
    }

    pub fn visit_mut<C>(
        &mut self,
        global_method: SceneNodeGlobalTraversalMethod,
        local_method: Option<SceneNodeLocalTraversalMethod>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut SceneNode) -> Result<(), String>,
    {
        let local = match local_method {
            Some(method) => method,
            None => Default::default(),
        };

        let current_depth: usize = 0;
        let parent_world_transform = Mat4::identity();

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => {
                self.visit_dfs_mut(&local, current_depth, parent_world_transform, visit_action)
            }
            SceneNodeGlobalTraversalMethod::BreadthFirst => self.visit_bfs_mut(visit_action),
        }
    }

    fn visit_dfs<C>(
        &self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        parent_world_transform: Mat4,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let current_world_transform = *(self.transform.mat()) * parent_world_transform;

        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, current_world_transform, self)?;

                match &self.children {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs(
                                local_method,
                                current_depth + 1,
                                current_world_transform,
                                visit_action,
                            )?;
                        }
                    }
                    None => (),
                }

                Ok(())
            }
            SceneNodeLocalTraversalMethod::PostOrder => {
                match &self.children {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs(
                                local_method,
                                current_depth + 1,
                                current_world_transform,
                                visit_action,
                            )?;
                        }
                    }
                    None => (),
                }

                visit_action(current_depth, current_world_transform, self)
            }
        }
    }

    fn visit_dfs_mut<C>(
        &mut self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        parent_world_transform: Mat4,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut Self) -> Result<(), String>,
    {
        let current_world_transform = *(self.transform.mat()) * parent_world_transform;

        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, current_world_transform, self)?;

                match self.children.as_mut() {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs_mut(
                                local_method,
                                current_depth + 1,
                                current_world_transform,
                                visit_action,
                            )?;
                        }
                    }
                    None => (),
                }

                Ok(())
            }
            SceneNodeLocalTraversalMethod::PostOrder => {
                match self.children.as_mut() {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs_mut(
                                local_method,
                                current_depth + 1,
                                current_world_transform,
                                visit_action,
                            )?;
                        }
                    }
                    None => (),
                }

                visit_action(current_depth, current_world_transform, self)
            }
        }
    }

    fn visit_bfs<C>(&self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, Mat4, &SceneNode)> = VecDeque::new();

        let current_depth: usize = 0;
        let parent_world_transform = *self.transform.mat();

        frontier.push_front((current_depth, parent_world_transform, self));

        while frontier.len() > 0 {
            let (current_depth, parent_world_transform, current_node) =
                frontier.pop_front().unwrap();

            let current_world_transform = *(current_node.transform.mat()) * parent_world_transform;

            visit_action(current_depth, current_world_transform, current_node)?;

            match &current_node.children {
                Some(children) => {
                    for child in children {
                        frontier.push_back((current_depth + 1, current_world_transform, child));
                    }
                }
                None => (),
            }
        }

        Ok(())
    }

    fn visit_bfs_mut<C>(&mut self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut Self) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, Mat4, &mut Self)> = VecDeque::new();

        let current_depth: usize = 0;
        let parent_world_transform = *self.transform.mat();

        frontier.push_front((current_depth, parent_world_transform, self));

        while frontier.len() > 0 {
            let (current_depth, parent_world_transform, current_node) =
                frontier.pop_front().unwrap();

            let current_world_transform = *(current_node.transform.mat()) * parent_world_transform;

            visit_action(current_depth, current_world_transform, current_node)?;

            match current_node.children.as_mut() {
                Some(children) => {
                    for child in children {
                        frontier.push_back((current_depth + 1, current_world_transform, child));
                    }
                }
                None => (),
            }
        }

        Ok(())
    }
}

impl<'a> Display for SceneNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uuid_suffix = match &self.handle {
            Some(handle) => format!(" | {}", handle.uuid),
            None => "".to_string(),
        };

        let children_suffix = match &self.children {
            Some(children) => {
                if children.len() > 1 {
                    format!(" ({} children)", children.len())
                } else {
                    " (1 child)".to_string()
                }
            }
            None => "".to_string(),
        };

        write!(f, "{}{}{}", self.node_type, uuid_suffix, children_suffix)?;

        Ok(())
    }
}
