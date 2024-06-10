use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::node::{Node, NodeLocalTraversalMethod};

pub fn visit_dfs_for_root<'a, T, C>(
    root: &Rc<RefCell<Node<'a, T>>>,
    method: &NodeLocalTraversalMethod,
    visit_action: &mut C,
) -> Result<(), String>
where
    T: Default + Clone + Serialize + Deserialize<'a>,
    C: FnMut(usize, Option<&T>, &Node<'a, T>) -> Result<(), String>,
{
    root.borrow().visit_dfs(method, 0, None, visit_action)
}

pub fn visit_dfs_mut_for_root<'a, T, C, D>(
    root: &Rc<RefCell<Node<'a, T>>>,
    method: &NodeLocalTraversalMethod,
    visit_action: &mut C,
    cleanup_action: &mut D,
) -> Result<(), String>
where
    T: Default + Clone + Serialize + Deserialize<'a>,
    C: FnMut(usize, usize, Option<&T>, &mut Node<'a, T>) -> Result<(), String>,
    D: FnMut(),
{
    root.borrow_mut()
        .visit_dfs_mut(method, 0, 0, None, visit_action, cleanup_action)
}

#[macro_export]
macro_rules! visit_dfs {
    ($name:ident, $type_param:ident, $self:ident, $root:ident) => {
        pub fn $name<C>(
            &self,
            method: &NodeLocalTraversalMethod,
            visit_action: &mut C,
        ) -> Result<(), String>
        where
            C: FnMut(usize, Option<&$type_param>, &Node<'a, $type_param>) -> Result<(), String>,
        {
            if let Some(root) = &self.$root {
                $crate::ui::tree::traversal::visit_dfs_for_root::<$type_param, C>(
                    root,
                    method,
                    visit_action,
                )
            } else {
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! visit_dfs_mut {
    ($name:ident, $type_param:ident, $self:ident, $root:ident) => {
        pub fn $name<C, D>(
            &self,
            method: &NodeLocalTraversalMethod,
            visit_action: &mut C,
            cleanup_action: &mut D,
        ) -> Result<(), String>
        where
            C: FnMut(
                usize,
                usize,
                Option<&$type_param>,
                &mut Node<'a, $type_param>,
            ) -> Result<(), String>,
            D: FnMut(),
        {
            if let Some(root) = &self.$root {
                $crate::ui::tree::traversal::visit_dfs_mut_for_root::<$type_param, C, D>(
                    root,
                    method,
                    visit_action,
                    cleanup_action,
                )
            } else {
                Ok(())
            }
        }
    };
}
