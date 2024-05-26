use std::{cell::RefCell, rc::Rc};

use crate::ui::{UI2DAxis, UISize};

use self::node::{Node, NodeLocalTraversalMethod};

use super::UIWidget;

pub mod node;

fn println_indent(depth: usize, msg: String) {
    let indent = 2 * (depth + 1);

    println!("{:indent$}{}", ">", msg);
}

pub struct UIWidgetTree<'a> {
    current: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    pub root: Rc<RefCell<Node<'a, UIWidget>>>,
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

    pub fn do_autolayout_pass(&mut self) -> Result<(), String> {
        println!("\nPerforming auto-layout pass:");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, _parent_data, node| {
            let widget = &mut node.data;

            println_indent(
                depth,
                format!("Calculating standalone size for node {}.", widget.id),
            );

            for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 {
                    UI2DAxis::X
                } else {
                    UI2DAxis::Y
                };

                match size_with_strictness.size {
                    UISize::Pixels(pixels) => {
                        println_indent(depth, format!("Pixel size for axis {}: {}", axis, pixels));
                        widget.computed_size[axis_index] = pixels as f32;
                    }
                    UISize::TextContent => match axis {
                        UI2DAxis::X => {
                            println_indent(
                                depth,
                                "Assuming horizontal text size to be 50.0".to_string(),
                            );

                            widget.computed_size[axis_index] = 50.0;
                        }
                        UI2DAxis::Y => {
                            println_indent(
                                depth,
                                "Assuming vertical text size to be 18.0".to_string(),
                            );

                            widget.computed_size[axis_index] = 10.0;
                        }
                    },
                    _ => println_indent(
                        depth,
                        format!(
                            "Widget {} uses {} size for {} axis. Skipping...",
                            widget.id, size_with_strictness.size, axis
                        ),
                    ),
                }
            }

            Ok(())
        })?;

        // 2. Calculate upward-dependent sizes with a pre-order traversal.

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, parent_data, node| {
            let widget = &mut node.data;

            println_indent(
                depth,
                format!(
                    "Calculating upward-dependent size for (child) node {}.",
                    widget.id,
                ),
            );

            for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                match size_with_strictness.size {
                    UISize::PercentOfParent(percentage) => match parent_data {
                        Some(data) => {
                            let parent_size_for_axis = data.semantic_sizes[axis_index].size;

                            match parent_size_for_axis {
                                UISize::ChildrenSum => {
                                    println_indent(
                                        depth,
                                        format!(
                                            "Widget {} uses {} size for {} axis, but parent uses downward-dependent {} size for same axis. Skipping...",
                                            widget.id, size_with_strictness.size, axis,
                                            parent_size_for_axis
                                        ),
                                    );
                                },
                                UISize::Null => panic!("Parent node has {} size for axis {}!", parent_size_for_axis, axis),
                                UISize::Pixels(_) | UISize::TextContent | UISize::PercentOfParent(_) => {
                                    widget.computed_size[axis_index] = data.computed_size[axis_index] * percentage;

                                    println_indent(
                                        depth,
                                        format!(
                                            "{} axis: Computed widget size {} as {} percent of parent size {}",
                                            axis,
                                            widget.computed_size[axis_index], 
                                            percentage * 100.0,
                                            data.computed_size[axis_index]
                                        ),
                                    );
                                },
                            }
                        }
                        None => {
                            panic!(
                                "Node uses {} size for {} axis, but node has no parent!",
                                size_with_strictness.size, axis
                            );
                        }
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "Widget {} uses {} size for {} axis. Skipping...",
                                widget.id, size_with_strictness.size, axis
                            ),
                        );
                    }
                }
            }

            Ok(())
        })?;

        // 3. Calculate downward-dependent sizes with a post-order traversal.

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PostOrder, &mut |depth, _parent_data, node| {
            let widget = &mut node.data;

            println_indent(
                depth,
                format!(
                    "Calculating downward-dependent size for (parent) node {}.",
                    widget.id,
                ),
            );

            for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                match size_with_strictness.size {
                    UISize::ChildrenSum => {
                        //

                        let mut sum = 0.0;

                        for child_node in &node.children {
                            let child_widget = &child_node.borrow().data;

                            sum += child_widget.computed_size[axis_index];
                        }

                        widget.computed_size[axis_index] = sum;

                        println_indent(
                            depth,
                            format!(
                                "{} axis: Computed widget size {} as the sum of its children's sizes.",
                                axis,
                                widget.computed_size[axis_index], 
                            ),
                        );
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "Widget {} uses {} size for {} axis. Skipping...",
                                widget.id, size_with_strictness.size, axis
                            ),
                        );
                    },
                }
            }

            Ok(())
        })?;

        // 4. Solve any violations (children extending beyond parent) with a pre-order traversal.

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, _parent_data, node| {
            let widget = &mut node.data;

            println_indent(
                depth,
                format!(
                    "Solving child layout violations for (parent) node {}.",
                    widget.id,
                ),
            );

            Ok(())
        })?;

        // 5. Compute the relative positions of each child with a pre-order traversal.

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, _parent_data, node| {
            let widget = &mut node.data;

            println_indent(
                depth,
                format!(
                    "Computing the relative (in-parent) position of (child) node {}.",
                    widget.id,
                ),
            );

            Ok(())
        })
    }

    pub fn visit_dfs<C>(
        &self,
        method: &NodeLocalTraversalMethod,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&UIWidget>, &Node<'a, UIWidget>) -> Result<(), String>,
    {
        self.root
            .borrow()
            .visit_dfs(method, 0, None, visit_action)
    }

    pub fn visit_dfs_mut<C>(
        &mut self,
        method: &NodeLocalTraversalMethod,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&UIWidget>, &mut Node<'a, UIWidget>) -> Result<(), String>,
    {
        self.root
            .borrow_mut()
            .visit_dfs_mut(method, 0, None, visit_action)
    }

    pub fn push(&mut self, widget: UIWidget) {
        let new_child_node_rc: Rc<RefCell<Node<'a, UIWidget>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            let mut new_child_node = Node::<'a, UIWidget>::new(widget);

            new_child_node.parent = Some(current_node_rc.clone());

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
