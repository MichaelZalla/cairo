use std::{cell::RefCell, collections::HashMap, rc::Rc};

use sdl2::mouse::MouseButton;

use crate::{
    buffer::Buffer2D,
    debug::println_indent,
    debug_print,
    device::{GameControllerState, KeyboardState, MouseEventKind, MouseState},
    ui::{
        extent::ScreenExtent,
        ui_box::{UIBoxFeatureFlag, UILayoutDirection},
        UI2DAxis, UISize,
    },
    visit_dfs, visit_dfs_mut,
};

use super::{
    context::GLOBAL_UI_CONTEXT,
    ui_box::{self, key::UIKey, UIBox},
};

use self::node::{Node, NodeLocalTraversalMethod};

pub mod node;
pub mod traversal;

#[derive(Default, Debug, Clone)]
pub struct UIBoxTree<'a> {
    current: Option<Rc<RefCell<Node<'a, UIBox>>>>,
    pub root: Option<Rc<RefCell<Node<'a, UIBox>>>>,
}

impl<'a> UIBoxTree<'a> {
    pub fn with_root(root_ui_box: UIBox) -> Self {
        let root_node = Node::<UIBox>::new(root_ui_box);
        let root_node_rc = Rc::new(RefCell::new(root_node));

        Self {
            current: Some(root_node_rc.clone()),
            root: Some(root_node_rc),
        }
    }

    visit_dfs!(visit_root_dfs, self, root);
    visit_dfs_mut!(visit_root_dfs_mut, self, root);

    // visit_dfs!(visit_dropdown_menus_root_dfs, self, dropdown_menus_root);
    // visit_dfs_mut!(visit_dropdown_menus_root_dfs_mut, self, dropdown_menus_root);

    // visit_dfs!(visit_tooltips_root_dfs, self, tooltips_root);
    // visit_dfs_mut!(visit_tooltips_root_dfs_mut, self, tooltips_root);

    pub fn clear(&mut self) {
        // @NOTE(mzalla) Does not drop cache entries, only tree structure!

        self.current = None;

        self.root = None;
    }

    pub fn do_user_inputs_pass(
        &mut self,
        _keyboard_state: &mut KeyboardState,
        mouse_state: &mut MouseState,
        _game_controller_state: &mut GameControllerState,
    ) -> Result<(), String> {
        debug_print!("\nUser inputs pass:\n");

        self.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PostOrder,
            &mut |_depth, _parent_data, node| {
                let ui_box = &mut node.data;

                // Apply the latest user inputs, based on this node's previous layout
                // (from the previous frame).

                ui_box.hot = if ui_box.features.contains(UIBoxFeatureFlag::Hoverable)
                    && !ui_box.key.is_null()
                {
                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        let cache = ctx.cache.borrow();

                        if let Some(ui_box_previous_frame) = cache.get(&ui_box.key) {
                            // Check if our global mouse coordinates overlap this node's bounds.

                            ui_box_previous_frame.global_bounds.contains(
                                mouse_state.position.0 as u32,
                                mouse_state.position.1 as u32,
                            )
                        } else {
                            // We weren't rendered in previous frames, so we can't be hot yet.

                            false
                        }
                    })
                } else {
                    // This node has no key (e.g., spacer, etc). Can't be hot.

                    false
                };

                ui_box.active = if ui_box.features.contains(UIBoxFeatureFlag::Clickable)
                    && !ui_box.key.is_null()
                {
                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        let cache = ctx.cache.borrow();

                        if let Some(ui_box_previous_frame) = cache.get(&ui_box.key) {
                            if ui_box_previous_frame.active {
                                mouse_state.buttons_down.contains(&MouseButton::Left)
                            } else if ui_box.hot {
                                if let Some(event) = mouse_state.button_event {
                                    let matches = matches!(
                                        (event.button, event.kind),
                                        (MouseButton::Left, MouseEventKind::Down)
                                    );

                                    if matches {
                                        mouse_state.button_event.take();
                                    }

                                    matches
                                } else {
                                    false
                                }
                            } else {
                                // We weren't previously active, and we aren't hot.

                                false
                            }
                        } else {
                            // We weren't rendered in previous frames, so we can't be active yet.

                            false
                        }
                    })
                } else {
                    // This node has no key (e.g., spacer, etc). Can't be active.

                    false
                };

                Ok(())
            },
        )
    }

    pub fn do_autolayout_pass(&mut self) -> Result<(), String> {
        debug_print!("\nAuto-layout pass:\n");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        debug_print!(">\n> (Standalone sizes pass...)\n>");

        self.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                let ui_box = &mut node.data;

                for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                    let axis = if axis_index == 0 {
                        UI2DAxis::X
                    } else {
                        UI2DAxis::Y
                    };

                    match size_with_strictness.size {
                        UISize::Pixels(pixels) => {
                            println_indent(
                                depth,
                                format!("{}: Pixel size for axis {}: {}", ui_box.id, axis, pixels),
                            );
                            ui_box.computed_size[axis_index] = pixels as f32;
                        }
                        UISize::TextContent => match axis {
                            UI2DAxis::X => {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Assuming horizontal text size to be 50.0",
                                        ui_box.id
                                    ),
                                );

                                ui_box.computed_size[axis_index] = 50.0;
                            }
                            UI2DAxis::Y => {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Assuming vertical text size to be 18.0",
                                        ui_box.id
                                    ),
                                );

                                ui_box.computed_size[axis_index] = 10.0;
                            }
                        },
                        _ => println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                ui_box.id, size_with_strictness.size, axis
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
            let ui_box = &mut node.data;

            if node.parent.is_none() {
                println_indent(
                    depth,
                    format!(
                        "{}: Skipping (root node).",
                        ui_box.id,
                    ),
                );
                
                return Ok(());
            }

            for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                match size_with_strictness.size {
                    UISize::PercentOfParent(percentage) => match parent_data {
                        Some(data) => {
                            let parent_size_for_axis = data.semantic_sizes[axis_index].size;

                            match parent_size_for_axis {
                                UISize::ChildrenSum => {
                                    panic!(
                                        "{}: Uses {} size for {} axis, but parent uses downward-dependent {} size for same axis.",
                                        ui_box.id, size_with_strictness.size, axis,
                                        parent_size_for_axis
                                    );
                                },
                                UISize::Null => panic!("{}: Parent node has {} size for axis {}!", ui_box.id, parent_size_for_axis, axis),
                                UISize::Pixels(_) | UISize::TextContent | UISize::PercentOfParent(_) => {
                                    ui_box.computed_size[axis_index] = data.computed_size[axis_index] * percentage;

                                    println_indent(
                                        depth,
                                        format!(
                                            "{}: ({} axis) Computed size {} as {} percent of parent size {}",
                                            ui_box.id,
                                            axis,
                                            ui_box.computed_size[axis_index], 
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
                                ui_box.id, size_with_strictness.size, axis
                            );
                        }
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                ui_box.id, size_with_strictness.size, axis
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
            let ui_box = &mut node.data;

            if node.children.is_empty() {
                println_indent(
                    depth,
                    format!(
                        "{}: Skipping (leaf node).",
                        ui_box.id,
                    ),
                );
                
                return Ok(());
            }

            for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                let axis = if axis_index == 0 { UI2DAxis::X } else { UI2DAxis::Y };

                match size_with_strictness.size {
                    UISize::ChildrenSum => {
                        let sum = {
                            let mut sum = 0.0;
                            
                            for child_node in &node.children {
                                let child_ui_box = &child_node.borrow().data;
    
                                sum += child_ui_box.computed_size[axis_index];
                            }

                            sum
                        };

                        ui_box.computed_size[axis_index] = sum;

                        println_indent(
                            depth,
                            format!(
                                "{}: ({} axis) Computed box size {} as the sum of its children's sizes.",
                                ui_box.id,
                                axis,
                                ui_box.computed_size[axis_index], 
                            ),
                        );
                    },
                    _ => {
                        println_indent(
                            depth,
                            format!(
                                "{}: Uses {} size for {} axis. Skipping.",
                                ui_box.id, size_with_strictness.size, axis
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
                let ui_box = &mut node.data;

                if node.children.is_empty() {
                    println_indent(depth, format!("{}: Skipping (leaf node).", ui_box.id,));

                    return Ok(());
                }

                for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                    match size_with_strictness.size {
                        UISize::Null | UISize::TextContent => panic!(),
                        UISize::ChildrenSum => {
                            println_indent(
                                depth,
                                format!(
                                    "{}: Uses {} size. Skipping.",
                                    ui_box.id, ui_box.semantic_sizes[axis_index].size,
                                ),
                            );
                        }
                        UISize::Pixels(_) | UISize::PercentOfParent(_) => {
                            let computed_size_along_axis = ui_box.computed_size[axis_index];

                            let size_of_children_along_axis = {
                                let child_sizes_along_axis = node
                                    .children
                                    .iter()
                                    .map(|c| c.borrow().data.computed_size[axis_index]);

                                match (ui_box.layout_direction, UI2DAxis::from_usize(axis_index)) {
                                    (UILayoutDirection::LeftToRight, UI2DAxis::X)
                                    | (UILayoutDirection::TopToBottom, UI2DAxis::Y) => {
                                        child_sizes_along_axis.into_iter().sum()
                                    }
                                    _ => child_sizes_along_axis
                                        .into_iter()
                                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                                        .unwrap(),
                                }
                            };

                            if computed_size_along_axis < size_of_children_along_axis {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Detected size violation of children ({} < {}).",
                                        ui_box.id,
                                        computed_size_along_axis,
                                        size_of_children_along_axis
                                    ),
                                );

                                // Need to account for strictness of children
                                // relative to each other, i.e., if one child's
                                // size is fixed in pixels, it will not give up
                                // it's "fair share" of space; its siblings must
                                // overcompensate, then.

                                // Scale down each of this box's children,
                                // according to the severity of the violation,
                                // as well as each child box's strictness
                                // parameter.

                                let size_reserved_for_strict_children: f32 = match (ui_box.layout_direction, UI2DAxis::from_usize(axis_index)) {
                                    (UILayoutDirection::LeftToRight, UI2DAxis::X)
                                    | (UILayoutDirection::TopToBottom, UI2DAxis::Y) => {
                                        node
                                    .children
                                    .iter()
                                    .map(|child| {
                                        let child_ui_box = &child.borrow().data;

                                        let child_strictness =
                                            child_ui_box.semantic_sizes[axis_index].strictness;

                                        if child_strictness == 1.0 {
                                            child_ui_box.computed_size[axis_index]
                                        } else {
                                            0.0
                                        }
                                    })
                                    .sum()
                                    }
                                    _ => {
                                        0.0
                                    },
                                };  

                                let alpha_adjusted_for_size_reserved =
                                    (computed_size_along_axis - size_reserved_for_strict_children) / (size_of_children_along_axis - size_reserved_for_strict_children);

                                for child in &node.children {
                                    let child_ui_box = &mut child.borrow_mut().data;

                                    let old_child_size = child_ui_box.computed_size[axis_index];

                                    let strictness =
                                        child_ui_box.semantic_sizes[axis_index].strictness;

                                    let new_child_size = if strictness == 1.0 {
                                        old_child_size
                                    } else if strictness == 0.0 {
                                        old_child_size * alpha_adjusted_for_size_reserved
                                    } else {
                                        panic!()
                                    };

                                    println_indent(
                                        depth + 1,
                                        format!(
                                            "{}: ({} axis) Scaling down from {} to {} (strictness: {}).",
                                            child_ui_box.id,
                                            UI2DAxis::from_usize(axis_index),
                                            old_child_size,
                                            new_child_size,
                                            strictness,
                                        ),
                                    );

                                    child_ui_box.computed_size[axis_index] = new_child_size;
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
                let ui_box = &mut node.data;

                let mut global_bounds = ScreenExtent {
                    left: ui_box.computed_relative_position[0] as u32,
                    top: ui_box.computed_relative_position[1] as u32,
                    ..Default::default()
                };

                if let Some(parent) = parent_data {
                    global_bounds.left += parent.global_bounds.left;
                    global_bounds.top += parent.global_bounds.top;
                }

                global_bounds.right = global_bounds.left + ui_box.computed_size[0] as u32;
                global_bounds.bottom = global_bounds.top + ui_box.computed_size[1] as u32;

                ui_box.global_bounds = global_bounds;

                if node.children.is_empty() {
                    return Ok(());
                }

                for (axis_index, _size_with_strictness) in ui_box.semantic_sizes.iter().enumerate()
                {
                    let mut cursor = 0.0;

                    for child_node_rc in &node.children {
                        let mut child_node = (*child_node_rc).borrow_mut();
                        let child_ui_box = &mut child_node.data;

                        child_ui_box.computed_relative_position[axis_index] = cursor;

                        match (ui_box.layout_direction, UI2DAxis::from_usize(axis_index)) {
                            (UILayoutDirection::LeftToRight, UI2DAxis::X)
                            | (UILayoutDirection::TopToBottom, UI2DAxis::Y) => {
                                cursor += child_ui_box.computed_size[axis_index];
                            }
                            _ => {}
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
                let ui_box = &node.data;

                let rel_position = ui_box.computed_relative_position;
                let global_position = ui_box.global_bounds;
                let size = ui_box.computed_size;

                println_indent(
                    depth,
                    format!(
                        "{}: Relative position: ({},{}) | Global position: ({},{}) | Computed size: {}x{}.",
                        ui_box.id, rel_position[0], rel_position[1], global_position.left, global_position.top, size[0], size[1],
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
            &mut |_depth, _parent_data, node| {
                let ui_box = &node.data;

                // 2. Render this node for the current frame.

                let render_result = ui_box.render(target);

                // 3. Update this node's cache entry (prepare for rendering the next frame).

                GLOBAL_UI_CONTEXT.with(|ctx| {
                    let mut cache = ctx.cache.borrow_mut();

                    update_cache_entry(&mut cache, ui_box, frame_index);
                });

                // Return the rendering result.

                render_result
            },
        )?;

        // Prune old entries from our UI cache.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let mut cache = ctx.cache.borrow_mut();

            cache.retain(|_key, ui_box: &mut UIBox| ui_box.last_read_at_frame == frame_index);
        });

        Ok(())
    }

    pub fn push(&mut self, ui_box: UIBox) -> Result<(), String> {
        let new_child_node_rc: Rc<RefCell<Node<'a, UIBox>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            if let UISize::TextContent = &current_node.data.semantic_sizes[0].size {
                return Err(
                    "Called UIBoxTree::push_parent() when current node uses TextContent size!"
                        .to_string(),
                );
            }

            if let UISize::ChildrenSum = &current_node.data.semantic_sizes[0].size {
                if let UISize::ChildrenSum = &current_node.data.semantic_sizes[1].size {
                    if !current_node.children.is_empty() {
                        return Err(
                            "Called UIBoxTree::push_parent() when current node uses ChildrenSum size on both axes, and already has a child!"
                        .to_string(),
                        );
                    }
                }
            }

            let mut new_child_node = Node::<'a, UIBox>::new(ui_box);

            new_child_node.parent = Some(current_node_rc.clone());

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            debug_assert!(self.root.is_none());

            let root_node = Node::<UIBox>::new(ui_box);
            new_child_node_rc = Rc::new(RefCell::new(root_node));

            self.root = Some(new_child_node_rc.clone());
            self.current = Some(new_child_node_rc);
        }

        Ok(())
    }

    pub fn push_parent(&mut self, ui_box: UIBox) -> Result<(), String> {
        self.push(ui_box)?;

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
                Err("Called UIBoxTree::pop_parent() on the root of the tree!".to_string())
            }
            _ => Err("Called UIBoxTree::pop_parent() on an empty tree!".to_string()),
        }
    }
}

fn update_cache_entry(cache: &mut HashMap<UIKey, UIBox>, ui_box: &UIBox, frame_index: u32) {
    if cache.contains_key(&ui_box.key) {
        let cached_ui_box = cache.get_mut(&ui_box.key).unwrap();

        cached_ui_box.global_bounds = ui_box.global_bounds;

        cached_ui_box.hot = ui_box.hot;
        cached_ui_box.hot_transition = ui_box.hot_transition;

        cached_ui_box.active = ui_box.active;
        cached_ui_box.active_transition = ui_box.active_transition;

        cached_ui_box.last_read_at_frame = frame_index;
    } else if !ui_box.key.is_null() {
        cache.insert(ui_box.key.clone(), ui_box.clone());
    }
}
