use crate::{matrix::Mat4, resource::handle::Handle};

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

#[derive(Default, Debug, Clone)]
pub struct SceneNode<'a> {
    node_type: SceneNodeType,
    transform: Mat4,
    handle: Option<Handle>,
    parent: Option<&'a SceneNode<'a>>,
    children: Option<Vec<SceneNode<'a>>>,
}

impl<'a> SceneNode<'a> {
    pub fn new(
        node_type: SceneNodeType,
        transform: Option<Mat4>,
        handle: Option<Handle>,
        parent: Option<&'a SceneNode<'a>>,
    ) -> Self {
        Self {
            node_type,
            transform: if transform.is_some() {
                transform.unwrap()
            } else {
                Mat4::identity()
            },
            handle,
            parent,
            children: None,
        }
    }

    pub fn get_type(&self) -> &SceneNodeType {
        &self.node_type
    }

    pub fn get_transform(&self) -> &Mat4 {
        &self.transform
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
}

pub struct SceneGraph<'a> {
    pub root: SceneNode<'a>,
}

impl<'a> SceneGraph<'a> {
    pub fn new() -> Self {
        Self {
            root: SceneNode {
                node_type: SceneNodeType::Empty,
                transform: Mat4::identity(),
                handle: None,
                parent: None,
                children: Some(vec![]),
            },
        }
    }
}
