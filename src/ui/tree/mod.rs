use std::{cell::RefCell, rc::Rc};

use self::node::Node;

use super::UIWidget;

pub mod node;

pub struct UIWidgetTree<'a> {
    current: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    root: Rc<RefCell<Node<'a, UIWidget>>>,
}

impl<'a> UIWidgetTree<'a> {
    pub fn new(mut root: Node<'a, UIWidget>) -> Self {
        root.parent = None;
        root.children = vec![];

        let root_rc = Rc::new(RefCell::new(root));

        Self {
            root: root_rc.clone(),
            current: Some(root_rc),
        }
    }
}

impl<'a> UIWidgetTree<'a> {
    pub fn push(&mut self, widget: UIWidget) {
        let new_child_node_rc: Rc<RefCell<Node<'a, UIWidget>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            let new_child_node = Node::<'a, UIWidget>::new(widget);

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            panic!("Called UIWidgetTree::push() on a tree with no `current` value!");
        }

        self.current = Some(new_child_node_rc.clone());
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<Node<'a, UIWidget>>>> {
        let (child_node_rc, parent_node_rc) = {
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

        let removed_child_node_rc = match (&child_node_rc, &parent_node_rc) {
            (Some(child_node_rc), Some(parent_node_rc)) => {
                let child_node = (*child_node_rc).borrow_mut();
                let mut parent_node = (*parent_node_rc).borrow_mut();

                // Remove child from parent.

                if let Some(child_index) = get_child_index(&parent_node, &child_node) {
                    let removed_child_node = parent_node.children.swap_remove(child_index);

                    removed_child_node.borrow_mut().parent = None;
                    removed_child_node.borrow_mut().children = vec![];

                    removed_child_node
                } else {
                    panic!("Failed to find child index!");
                }
            }
            (Some(_child_node_rc), None) => {
                panic!("Called UIWidgetTree::pop() on the root of the tree!");
            }
            _ => {
                panic!("Called UIWidgetTree::pop() on an empty tree!");
            }
        };

        // Set `self.current` to parent.

        self.current = parent_node_rc.clone();

        // Return child.

        Some(removed_child_node_rc)
    }
}

fn get_child_index(parent: &Node<UIWidget>, child: &Node<UIWidget>) -> Option<usize> {
    let mut child_index: isize = -1;

    for (i, other_child) in parent.children.iter().enumerate() {
        let lhs = (*other_child).borrow();
        let rhs = child;

        if lhs.data.id == rhs.data.id {
            child_index = i as isize;
        }
    }

    if child_index > -1 {
        Some(child_index as usize)
    } else {
        None
    }
}
