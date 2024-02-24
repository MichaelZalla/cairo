use std::{
    collections::VecDeque,
    fmt::{Display, Error},
};

use crate::{resource::handle::Handle, transform::Transform3D};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum SceneNodeType {
    #[default]
    Empty,
    Entity,
    Camera,
    DirectionalLight,
    PointLight,
    SpotLight,
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

    pub fn is_type(&self, node_type: SceneNodeType) -> bool {
        self.node_type == node_type
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
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

    pub fn add_child(&mut self, node: SceneNode<'a>) {
        match self.children.as_mut() {
            Some(children) => {
                children.push(node);
            }
            None => {
                self.children = Some(vec![node]);
            }
        }
    }

    pub fn visit<C>(
        &self,
        global_method: SceneNodeGlobalTraversalMethod,
        local_method: Option<SceneNodeLocalTraversalMethod>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, &Self) -> Result<(), String>,
    {
        let local = match local_method {
            Some(method) => method,
            None => Default::default(),
        };

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => self.visit_dfs(&local, 0, visit_action),
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
        C: FnMut(usize, &mut Self) -> Result<(), String>,
    {
        let local = match local_method {
            Some(method) => method,
            None => Default::default(),
        };

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => {
                self.visit_dfs_mut(&local, 0, visit_action)
            }
            SceneNodeGlobalTraversalMethod::BreadthFirst => self.visit_bfs_mut(visit_action),
        }
    }

    fn visit_dfs<C>(
        &self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, &Self) -> Result<(), String>,
    {
        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, self)?;

                match &self.children {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs(local_method, current_depth + 1, visit_action)?;
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
                            child.visit_dfs(local_method, current_depth + 1, visit_action)?;
                        }
                    }
                    None => (),
                }

                visit_action(current_depth, self)
            }
        }
    }

    fn visit_dfs_mut<C>(
        &mut self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, &mut Self) -> Result<(), String>,
    {
        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, self)?;

                match self.children.as_mut() {
                    Some(children) => {
                        for child in children {
                            child.visit_dfs_mut(local_method, current_depth + 1, visit_action)?;
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
                            child.visit_dfs_mut(local_method, current_depth + 1, visit_action)?;
                        }
                    }
                    None => (),
                }

                visit_action(current_depth, self)
            }
        }
    }

    fn visit_bfs<C>(&self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, &Self) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, &Self)> = VecDeque::new();

        frontier.push_front((0, self));

        while frontier.len() > 0 {
            let (current_depth, current_node) = frontier.pop_front().unwrap();

            visit_action(current_depth, current_node)?;

            match &current_node.children {
                Some(children) => {
                    for child in children {
                        frontier.push_back((current_depth + 1, child));
                    }
                }
                None => (),
            }
        }

        Ok(())
    }

    fn visit_bfs_mut<C>(&mut self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, &mut Self) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, &mut Self)> = VecDeque::new();

        frontier.push_front((0, self));

        while frontier.len() > 0 {
            let (current_depth, current_node) = frontier.pop_front().unwrap();

            visit_action(current_depth, current_node)?;

            match current_node.children.as_mut() {
                Some(children) => {
                    for child in children {
                        frontier.push_back((current_depth + 1, child));
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
        let uuid_suffix: String = match &self.handle {
            Some(handle) => format!(" (UUID {})", handle.uuid),
            None => "".to_string(),
        };

        let children_suffix: String = match &self.children {
            Some(children) => format!(" ({} children)", children.len()),
            None => "".to_string(),
        };

        write!(f, "{}{}{}", self.node_type, uuid_suffix, children_suffix)?;

        Ok(())
    }
}

pub struct SceneGraph<'a> {
    pub root: SceneNode<'a>,
}

impl<'a> SceneGraph<'a> {
    pub fn new() -> Self {
        Self {
            root: SceneNode {
                node_type: SceneNodeType::Empty,
                transform: Default::default(),
                handle: None,
                parent: None,
                children: Some(vec![]),
            },
        }
    }
}

impl<'a> Display for SceneGraph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_node_to_formatter =
            |current_depth: usize, node: &SceneNode| -> Result<(), String> {
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
