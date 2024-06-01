use core::fmt;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use serde::{self, Deserialize, Serialize};

use crate::{visit_dfs, visit_dfs_mut};

use self::node::{Node, NodeLocalTraversalMethod};

pub mod node;
pub mod traversal;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Tree<'a, T> {
    #[serde(flatten)]
    pub root: Option<Rc<RefCell<Node<'a, T>>>>,
    #[serde(skip)]
    current: Option<Rc<RefCell<Node<'a, T>>>>,
}

impl<'a, T: Default + Clone + Display + Serialize + Deserialize<'a>> fmt::Display for Tree<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.visit_root_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let indent = 2 * (depth + 1);

                writeln!(f, "{:indent$}{}", ">", node.data).unwrap();

                Ok(())
            },
        ) {
            Ok(()) => fmt::Result::Ok(()),
            Err(_s) => fmt::Result::Err(std::fmt::Error {}),
        }
    }
}

impl<'a, T: Default + Clone + Display + Serialize + Deserialize<'a>> Tree<'a, T> {
    pub fn get_current(&self) -> Option<&Rc<RefCell<Node<'a, T>>>> {
        self.current.as_ref()
    }

    pub fn with_root(root_t: T) -> Self {
        let root_node = Node::<T>::new(root_t);
        let root_node_rc = Rc::new(RefCell::new(root_node));

        Self {
            current: Some(root_node_rc.clone()),
            root: Some(root_node_rc),
        }
    }

    visit_dfs!(visit_root_dfs, T, self, root);
    visit_dfs_mut!(visit_root_dfs_mut, T, self, root);

    pub fn clear(&mut self) {
        self.current = None;

        self.root = None;
    }

    pub fn push(&mut self, data: T) -> Result<(), String> {
        let new_child_node_rc: Rc<RefCell<Node<'a, T>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            let mut new_child_node = Node::<'a, T>::new(data);

            new_child_node.parent = Some(current_node_rc.clone());

            // println!("Pushing node '{}'.", new_child_node.data.id);

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            debug_assert!(self.root.is_none());

            let root_node = Node::<T>::new(data);
            new_child_node_rc = Rc::new(RefCell::new(root_node));

            self.root = Some(new_child_node_rc.clone());

            // println!("Setting `current` to node '{}'.", new_child_node_rc.borrow().data.id);

            self.current = Some(new_child_node_rc);
        }

        Ok(())
    }

    pub fn push_parent(&mut self, data: T) -> Result<(), String> {
        self.push(data)?;

        self.push_parent_post();

        Ok(())
    }

    pub fn push_parent_post(&mut self) {
        debug_assert!(self.current.is_some());

        let new_current;

        {
            let current_node_rc = self.current.as_ref().unwrap();

            let current_node = current_node_rc.borrow();

            if current_node.children.is_empty() {
                // We just added the root node (and set `self.current`).
                new_current = self.root.clone();
            } else {
                // We just added some other node as a child of `self.current`.
                let new_child_rc = current_node.children.last().unwrap();

                new_current = Some(new_child_rc.clone());
            }
        }

        // println!("Setting `current` to {}.", match &new_current {
        //     Some(rc) => format!("node '{}'", (*rc).borrow().data.id),
        //     None => "None".to_string(),
        // });

        self.current = new_current;
    }

    pub fn pop_parent(&mut self) -> Result<(), String> {
        let (current_node_rc, parent_node_rc) = {
            match &self.current {
                Some(current_node_rc) => {
                    let current_node = current_node_rc.borrow();

                    match &current_node.parent {
                        Some(parent_node_rc) => {
                            (Some(current_node_rc.clone()), Some(parent_node_rc.clone()))
                        }
                        None => (Some(current_node_rc.clone()), None),
                    }
                }
                None => (None, None),
            }
        };

        match (&current_node_rc, &parent_node_rc) {
            (Some(_current_node_rc), Some(parent_node_rc)) => {
                // Set `self.current` to parent.

                // println!("Setting `current` to node '{}'.", parent_node_rc.borrow().data.id);

                self.current = Some(parent_node_rc.clone());

                Ok(())
            }
            (Some(_child_node_rc), None) => {
                Err("Called Tree::pop_parent() on the root of the tree!".to_string())
            }
            _ => Err("Called Tree::pop_parent() on an empty tree!".to_string()),
        }
    }
}
