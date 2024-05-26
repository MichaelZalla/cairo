use std::{cell::RefCell, rc::Rc};

use crate::{
    debug::println_indent,
    ui::{UI2DAxis, UISize},
};

use self::node::{Node, NodeLocalTraversalMethod};

use super::UIWidget;

pub mod node;

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
        println!("\nAuto-layout pass:\n");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        println!(">\n> (Standalone sizes pass...)\n>");

        self.visit_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &mut node.data;

                for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                    let axis = if axis_index == 0 {
                        UI2DAxis::X
                    } else {
                        UI2DAxis::Y
                    };

                    match size_with_strictness.size {
                        UISize::Pixels(pixels) => {
                            println_indent(
                                depth,
                                format!("{}: Pixel size for axis {}: {}", widget.id, axis, pixels),
                            );
                            widget.computed_size[axis_index] = pixels as f32;
                        }
                        UISize::TextContent => match axis {
                            UI2DAxis::X => {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Assuming horizontal text size to be 50.0",
                                        widget.id
                                    ),
                                );

                                widget.computed_size[axis_index] = 50.0;
                            }
                            UI2DAxis::Y => {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Assuming vertical text size to be 18.0",
                                        widget.id
                                    ),
                                );

                                widget.computed_size[axis_index] = 10.0;
                            }
                        },
                        _ => println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                widget.id, size_with_strictness.size, axis
                            ),
                        ),
                    }
                }

                Ok(())
            },
        )?;

        // 2. Calculate upward-dependent sizes with a pre-order traversal.

        println!(">\n> (Upward-dependent sizes pass...)\n>");

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, parent_data, node| {
            let widget = &mut node.data;

            if node.parent.is_none() {
                println_indent(
                    depth,
                    format!(
                        "{}: Skipping (root node).",
                        widget.id,
                    ),
                );
                
                return Ok(());
            }

            for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                match size_with_strictness.size {
                    UISize::PercentOfParent(percentage) => match parent_data {
                        Some(data) => {
                            let parent_size_for_axis = data.semantic_sizes[axis_index].size;

                            match parent_size_for_axis {
                                UISize::ChildrenSum => {
                                    panic!(
                                        "{}: Uses {} size for {} axis, but parent uses downward-dependent {} size for same axis.",
                                        widget.id, size_with_strictness.size, axis,
                                        parent_size_for_axis
                                    );
                                },
                                UISize::Null => panic!("{}: Parent node has {} size for axis {}!", widget.id, parent_size_for_axis, axis),
                                UISize::Pixels(_) | UISize::TextContent | UISize::PercentOfParent(_) => {
                                    widget.computed_size[axis_index] = data.computed_size[axis_index] * percentage;

                                    println_indent(
                                        depth,
                                        format!(
                                            "{}: ({} axis) Computed size {} as {} percent of parent size {}",
                                            widget.id,
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
                                "{}: Uses {} size for {} axis, but node has no parent!",
                                widget.id, size_with_strictness.size, axis
                            );
                        }
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                widget.id, size_with_strictness.size, axis
                            ),
                        );
                    }
                }
            }

            Ok(())
        })?;

        // 3. Calculate downward-dependent sizes with a post-order traversal.

        println!(">\n> (Downward-dependent sizes pass...)\n>");

        self.visit_dfs_mut(&NodeLocalTraversalMethod::PostOrder, &mut |depth, _parent_data, node| {
            let widget = &mut node.data;

            if node.children.is_empty() {
                println_indent(
                    depth,
                    format!(
                        "{}: Skipping (leaf node).",
                        widget.id,
                    ),
                );
                
                return Ok(());
            }

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
                                "{}: ({} axis) Computed widget size {} as the sum of its children's sizes.",
                                widget.id,
                                axis,
                                widget.computed_size[axis_index], 
                            ),
                        );
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                widget.id, size_with_strictness.size, axis
                            ),
                        );
                    },
                }
            }

            Ok(())
        })?;

        // 4. Solve any violations (children extending beyond parent) with a pre-order traversal.

        println!(">\n> (Violations pass...)\n>");

        self.visit_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &mut node.data;

                if node.children.is_empty() {
                    println_indent(
                        depth,
                        format!(
                            "{}: Skipping (leaf node).",
                            widget.id,
                        ),
                    );
                    
                    return Ok(());
                }

                println_indent(
                    depth,
                    format!("{}: Solving child layout violations.", widget.id,),
                );

                Ok(())
            },
        )?;

        // 5. Compute the relative positions of each child with a pre-order traversal.

        println!(">\n> (Relative positioning pass...)\n>");

        self.visit_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &mut node.data;

                println_indent(
                    depth,
                    format!("{}: Computing relative (in-parent) position.", widget.id,),
                );

                Ok(())
            },
        )?;

        // Check our results.

        self.debug_computed_sizes()
    }

    fn debug_computed_sizes(&self) -> Result<(), String> {

        println!("\nResults:\n");
       
        self.visit_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &node.data;

                println_indent(
                    depth,
                    format!(
                        "{}: Computed size: {}x{}.",
                        widget.id, widget.computed_size[0], widget.computed_size[1]
                    ),
                );

                Ok(())
            },
        )?;

        panic!();

        Ok(())
    }

    pub fn visit_dfs<C>(
        &self,
        method: &NodeLocalTraversalMethod,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&UIWidget>, &Node<'a, UIWidget>) -> Result<(), String>,
    {
        self.root.borrow().visit_dfs(method, 0, None, visit_action)
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

    pub fn pop_current(&mut self) -> Result<(), String> {
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

                self.current = Some(parent_node_rc.clone());

                Ok(())
            }
            (Some(_child_node_rc), None) => {
                Err("Called UIWidgetTree::pop() on the root of the tree!".to_string())
            }
            _ => Err("Called UIWidgetTree::pop() on an empty tree!".to_string()),
        }
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
