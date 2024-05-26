use std::{cell::RefCell, rc::Rc};

use crate::{
    animation::lerp, buffer::Buffer2D, debug::println_indent, debug_print, ui::{widget::ScreenExtent, UI2DAxis, UISize},
};

use self::node::{Node, NodeLocalTraversalMethod};

use super::widget::UIWidget;

pub mod node;

#[derive(Default, Debug, Clone)]
pub struct UIWidgetTree<'a> {
    current: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    pub root: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
}

impl<'a> UIWidgetTree<'a> {
    pub fn with_root(root_widget: UIWidget) -> Self {
        let root_node = Node::<UIWidget>::new(root_widget);
        let root_node_rc = Rc::new(RefCell::new(root_node));

        Self {
            root: Some(root_node_rc.clone()),
            current: Some(root_node_rc),
        }
    }

    pub fn do_autolayout_pass(&mut self) -> Result<(), String> {
        debug_print!("\nAuto-layout pass:\n");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        debug_print!(">\n> (Standalone sizes pass...)\n>");

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

        debug_print!(">\n> (Upward-dependent sizes pass...)\n>");

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

        debug_print!(">\n> (Downward-dependent sizes pass...)\n>");

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
                        let sum = {
                            let mut sum = 0.0;
                            
                            for child_node in &node.children {
                                let child_widget = &child_node.borrow().data;
    
                                sum += child_widget.computed_size[axis_index];
                            }

                            sum
                        };

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

        debug_print!(">\n> (Violations pass...)\n>");

        self.visit_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &mut node.data;

                if node.children.is_empty() {
                    println_indent(depth, format!("{}: Skipping (leaf node).", widget.id,));

                    return Ok(());
                }

                for (axis_index, size_with_strictness) in widget.semantic_sizes.iter().enumerate() {
                    // let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                    match size_with_strictness.size {
                        UISize::Null | UISize::TextContent => panic!(),
                        UISize::ChildrenSum => {
                            println_indent(
                                depth,
                                format!(
                                    "{}: Uses {} size. Skipping.",
                                    widget.id, widget.semantic_sizes[axis_index].size,
                                ),
                            );
                        }
                        UISize::Pixels(_) | UISize::PercentOfParent(_) => {
                            let computed_size_along_axis = widget.computed_size[axis_index];

                            let sum_of_child_sizes_along_axis = {
                                let mut sum = 0.0;

                                for child_node in &node.children {
                                    let child_widget = &child_node.borrow().data;

                                    sum += child_widget.computed_size[axis_index];
                                }

                                sum
                            };

                            if computed_size_along_axis < sum_of_child_sizes_along_axis {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Detected size violation of children ({} < {}).",
                                        widget.id,
                                        computed_size_along_axis,
                                        sum_of_child_sizes_along_axis
                                    ),
                                );

                                // Scale down each of this widget's children,
                                // according to the severity of the violation,
                                // as well as each child widget's strictness
                                // parameter.

                                let alpha =
                                    computed_size_along_axis / sum_of_child_sizes_along_axis;

                                for child in &node.children {
                                    let child_widget = &mut child.borrow_mut().data;

                                    let strictness =
                                        child_widget.semantic_sizes[axis_index].strictness;
                                    let old_child_size = child_widget.computed_size[axis_index];
                                    let new_child_size =
                                        old_child_size * lerp(alpha, 1.0, strictness);

                                    println_indent(
                                        depth + 1,
                                        format!(
                                            "{}: Scaling down from {} to {} (strictness: {}).",
                                            child_widget.id,
                                            old_child_size,
                                            new_child_size,
                                            strictness,
                                        ),
                                    );

                                    child_widget.computed_size[axis_index] = new_child_size;
                                }
                            }
                        }
                    }
                }

                Ok(())
            },
        )?;

        // 5. Compute the relative positions of each child with a pre-order traversal.

        debug_print!(">\n> (Relative positioning pass...)\n>");

        self.visit_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, parent_data, node| {
                let widget = &mut node.data;

                let mut global_bounds = ScreenExtent {
                    left: widget.computed_relative_position[0] as u32,
                    top: widget.computed_relative_position[1] as u32,
                    ..Default::default()
                };

                if let Some(parent) = parent_data {
                    global_bounds.left += parent.global_bounds.left;
                    global_bounds.top += parent.global_bounds.top;
                }

                global_bounds.right = global_bounds.left + widget.computed_size[0] as u32;
                global_bounds.bottom = global_bounds.top + widget.computed_size[1] as u32;

                widget.global_bounds = global_bounds;

                if node.children.is_empty() {
                    return Ok(());
                }

                for (axis_index, _size_with_strictness) in widget.semantic_sizes.iter().enumerate()
                {
                    let mut cursor = 0.0;

                    for child_node_rc in &node.children {
                        let mut child_node = (*child_node_rc).borrow_mut();
                        let child_widget = &mut child_node.data;

                        child_widget.computed_relative_position[axis_index] = cursor;

                        if axis_index == 1 {
                            cursor += child_widget.computed_size[axis_index];
                        }
                    }
                }

                Ok(())
            },
        )?;

        // Check our results.

        self.debug_computed_sizes()
    }

    fn debug_computed_sizes(&self) -> Result<(), String> {
        debug_print!("\nResults:\n");

        self.visit_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &node.data;

                let rel_position = widget.computed_relative_position;
                let global_position = widget.global_bounds;
                let size = widget.computed_size;

                println_indent(
                    depth,
                    format!(
                        "{}: Relative position: ({},{}) | Global position: ({},{}) | Computed size: {}x{}.",
                        widget.id, rel_position[0], rel_position[1], global_position.left, global_position.top, size[0], size[1],
                    ),
                );

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn render(&self, frame_index: u32, target: &mut Buffer2D) -> Result<(), String> {
        self.visit_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &node.data;

                widget.render(depth, frame_index, target)
            },
        )
    }

    pub fn visit_dfs<C>(
        &self,
        method: &NodeLocalTraversalMethod,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&UIWidget>, &Node<'a, UIWidget>) -> Result<(), String>,
    {
        if let Some(root) = &self.root {
            root.borrow().visit_dfs(method, 0, None, visit_action)
        } else {
            Ok(())
        }
    }

    pub fn visit_dfs_mut<C>(
        &mut self,
        method: &NodeLocalTraversalMethod,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Option<&UIWidget>, &mut Node<'a, UIWidget>) -> Result<(), String>,
    {
        if let Some(root) = &self.root {
            root
            .borrow_mut()
            .visit_dfs_mut(method, 0, None, visit_action)
        } else {
            Ok(())
        }
    }

    pub fn push_parent(&mut self, widget: UIWidget) -> Result<(), String> {
        self.push(widget)?;

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

        self.current = new_current;

        Ok(())
    }

    pub fn push(&mut self, widget: UIWidget) -> Result<(), String> {
        let new_child_node_rc: Rc<RefCell<Node<'a, UIWidget>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            if let UISize::TextContent = &current_node.data.semantic_sizes[0].size {
                return Err(
                    "Called UIWidgetTree::push_parent() when current node uses TextContent size!"
                        .to_string(),
                );
            }

            if let UISize::ChildrenSum = &current_node.data.semantic_sizes[0].size {
                if let UISize::ChildrenSum = &current_node.data.semantic_sizes[1].size {
                    if !current_node.children.is_empty() {
                        return Err(
                            "Called UIWidgetTree::push_parent() when current node uses ChildrenSum size on both axes, and already has a child!"
                        .to_string(),
                        );
                    }
                }
            }

            let mut new_child_node = Node::<'a, UIWidget>::new(widget);

            new_child_node.parent = Some(current_node_rc.clone());

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            debug_assert!(self.root.is_none());

            let root_node = Node::<UIWidget>::new(widget);
            new_child_node_rc = Rc::new(RefCell::new(root_node));

            self.root = Some(new_child_node_rc.clone());
            self.current = Some(new_child_node_rc);
        }

        Ok(())
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

                self.current = Some(parent_node_rc.clone());

                Ok(())
            }
            (Some(_child_node_rc), None) => {
                Err("Called UIWidgetTree::pop_parent() on the root of the tree!".to_string())
            }
            _ => Err("Called UIWidgetTree::pop_parent() on an empty tree!".to_string()),
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
