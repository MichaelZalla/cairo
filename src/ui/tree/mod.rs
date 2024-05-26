use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    animation::lerp,
    buffer::Buffer2D,
    debug::println_indent,
    debug_print,
    ui::{widget::ScreenExtent, UI2DAxis, UISize},
    visit_dfs, visit_dfs_mut,
};

use self::node::{Node, NodeLocalTraversalMethod};

use super::widget::{key::UIKey, UIWidget};

pub mod node;
pub mod traversal;

#[derive(Default, Debug, Clone)]
pub struct UIWidgetTree<'a> {
    current: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    pub root: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    pub dropdown_menus_root: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    pub tooltips_root: Option<Rc<RefCell<Node<'a, UIWidget>>>>,
    cache: RefCell<HashMap<UIKey, UIWidget>>,
}

impl<'a> UIWidgetTree<'a> {
    pub fn with_root(root_widget: UIWidget) -> Self {
        let root_node = Node::<UIWidget>::new(root_widget);
        let root_node_rc = Rc::new(RefCell::new(root_node));

        Self {
            current: Some(root_node_rc.clone()),
            root: Some(root_node_rc),
            dropdown_menus_root: None,
            tooltips_root: None,
            cache: Default::default(),
        }
    }

    visit_dfs!(visit_root_dfs, self, root);
    visit_dfs_mut!(visit_root_dfs_mut, self, root);

    visit_dfs!(visit_dropdown_menus_root_dfs, self, dropdown_menus_root);
    visit_dfs_mut!(visit_dropdown_menus_root_dfs_mut, self, dropdown_menus_root);

    visit_dfs!(visit_tooltips_root_dfs, self, tooltips_root);
    visit_dfs_mut!(visit_tooltips_root_dfs_mut, self, tooltips_root);

    pub fn clear(&mut self) {
        // @NOTE(mzalla) Does not drop cache entries, only tree structure!

        self.current = None;

        self.root = None;
    }

    pub fn do_autolayout_pass(&mut self) -> Result<(), String> {
        debug_print!("\nAuto-layout pass:\n");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        debug_print!(">\n> (Standalone sizes pass...)\n>");

        self.visit_root_dfs_mut(
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

        self.visit_root_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, parent_data, node| {
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

        self.visit_root_dfs_mut(&NodeLocalTraversalMethod::PostOrder, &mut |depth, _parent_data, node| {
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

        self.visit_root_dfs_mut(
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

        self.visit_root_dfs_mut(
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

        self.visit_root_dfs(
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

    pub fn render(&mut self, frame_index: u32, target: &mut Buffer2D) -> Result<(), String> {
        self.visit_root_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let widget = &node.data;

                let mut cache = self.cache.borrow_mut();

                let render_result = widget.render(depth, frame_index, target);

                // Update this node's entry in our persistent cache.

                if cache.contains_key(&widget.key) {
                    let cached_widget = cache.get_mut(&widget.key).unwrap();

                    // cached_widget.computed_size = widget.computed_size;
                    // cached_widget.computed_relative_position = widget.computed_relative_position;
                    cached_widget.global_bounds = widget.global_bounds;

                    cached_widget.hot_transition = widget.hot_transition;
                    cached_widget.active_transition = widget.active_transition;

                    cached_widget.last_read_at_frame = frame_index;
                } else {
                    cache.insert(widget.key.clone(), widget.clone());
                }

                render_result
            },
        )?;

        {
            let mut cache = self.cache.borrow_mut();

            cache.retain(|_key, widget: &mut UIWidget| widget.last_read_at_frame == frame_index);
        }

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
}
