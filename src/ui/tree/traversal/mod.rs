use std::{cell::RefCell, rc::Rc};

use super::{
    node::{Node, NodeLocalTraversalMethod},
    ui_box::UIBox,
};

pub(crate) fn visit_dfs_for_root<'a, C>(
    root: &Rc<RefCell<Node<'a, UIBox>>>,
    method: &NodeLocalTraversalMethod,
    visit_action: &mut C,
) -> Result<(), String>
where
    C: FnMut(usize, Option<&UIBox>, &Node<'a, UIBox>) -> Result<(), String>,
{
    root.borrow().visit_dfs(method, 0, None, visit_action)
}

pub(crate) fn visit_dfs_mut_for_root<'a, C>(
    root: &Rc<RefCell<Node<'a, UIBox>>>,
    method: &NodeLocalTraversalMethod,
    visit_action: &mut C,
) -> Result<(), String>
where
    C: FnMut(usize, Option<&UIBox>, &mut Node<'a, UIBox>) -> Result<(), String>,
{
    root.borrow_mut()
        .visit_dfs_mut(method, 0, None, visit_action)
}

#[macro_export]
macro_rules! visit_dfs {
    ($name:ident, $self:ident, $root:ident) => {
        pub fn $name<C>(
            &self,
            method: &NodeLocalTraversalMethod,
            visit_action: &mut C,
        ) -> Result<(), String>
        where
            C: FnMut(usize, Option<&UIBox>, &Node<'a, UIBox>) -> Result<(), String>,
        {
            if let Some(root) = &self.$root {
                $crate::ui::tree::traversal::visit_dfs_for_root(root, method, visit_action)
            } else {
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! visit_dfs_mut {
    ($name:ident, $self:ident, $root:ident) => {
        pub fn $name<C>(
            &self,
            method: &NodeLocalTraversalMethod,
            visit_action: &mut C,
        ) -> Result<(), String>
        where
            C: FnMut(usize, Option<&UIBox>, &mut Node<'a, UIBox>) -> Result<(), String>,
        {
            if let Some(root) = &self.$root {
                $crate::ui::tree::traversal::visit_dfs_mut_for_root(root, method, visit_action)
            } else {
                Ok(())
            }
        }
    };
}
