use std::fmt::{Display, Error};

use super::node::{
    SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
};

pub struct SceneGraph<'a> {
    pub root: SceneNode<'a>,
}

impl<'a> SceneGraph<'a> {
    pub fn new() -> Self {
        Self {
            root: SceneNode::new(SceneNodeType::Empty, Default::default(), None, None),
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
