use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy)]
pub enum NodeLocalTraversalMethod {
    #[default]
    PreOrder,
    PostOrder,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Node<'a, T> {
    pub data: T,
    pub children: Vec<Rc<RefCell<Node<'a, T>>>>,
    #[serde(skip)]
    pub parent: Option<Rc<RefCell<Node<'a, T>>>>,
}

impl<'a, T> Node<'a, T>
where
    T: Default + Clone + Serialize + Deserialize<'a>,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    pub fn visit_dfs<C>(
        &self,
        method: &NodeLocalTraversalMethod,
        current_depth: usize,
        parent_data: Option<&T>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&T>, &Self) -> Result<(), String>,
    {
        match method {
            NodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, parent_data, self)?;

                for child_rc in &self.children {
                    let child = (*child_rc).borrow();

                    child.visit_dfs(method, current_depth + 1, Some(&self.data), visit_action)?;
                }

                Ok(())
            }
            NodeLocalTraversalMethod::PostOrder => {
                for child_rc in &self.children {
                    let child = (*child_rc).borrow();

                    child.visit_dfs(method, current_depth + 1, Some(&self.data), visit_action)?;
                }

                visit_action(current_depth, parent_data, self)
            }
        }
    }

    pub fn visit_dfs_mut<C>(
        &mut self,
        method: &NodeLocalTraversalMethod,
        current_depth: usize,
        parent_data: Option<&T>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&T>, &mut Self) -> Result<(), String>,
    {
        match method {
            NodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, parent_data, self)?;

                for child_rc in &mut self.children {
                    let mut child = (*child_rc).borrow_mut();

                    child.visit_dfs_mut(
                        method,
                        current_depth + 1,
                        Some(&self.data),
                        visit_action,
                    )?;
                }

                Ok(())
            }
            NodeLocalTraversalMethod::PostOrder => {
                for child_rc in &mut self.children {
                    let mut child = (*child_rc).borrow_mut();

                    child.visit_dfs_mut(
                        method,
                        current_depth + 1,
                        Some(&self.data),
                        visit_action,
                    )?;
                }

                visit_action(current_depth, parent_data, self)
            }
        }
    }
}