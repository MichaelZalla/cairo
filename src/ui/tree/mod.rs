use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    buffer::Buffer2D,
    debug::println_indent,
    debug_print,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::text::cache::cache_text,
    ui::{extent::ScreenExtent, ui_box::UILayoutDirection, UI2DAxis, UISize},
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
        seconds_since_last_update: f32,
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

                ui_box.update_hot_state(seconds_since_last_update, mouse_state);
                ui_box.update_active_state(seconds_since_last_update, mouse_state);

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
            &mut |depth, parent_data, node| {
                let ui_box: &mut UIBox = &mut node.data;
                
                let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

                for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                    let axis = UI2DAxis::from_usize(axis_index);

                    let is_horizontal_axis = match (axis, parent_layout_direction) {
                        (UI2DAxis::Primary, UILayoutDirection::LeftToRight) | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                        (UI2DAxis::Primary, UILayoutDirection::TopToBottom) | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false
                    };
    
                    let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

                    match size_with_strictness.size {
                        UISize::Pixels(pixels) => {
                            println_indent(
                                depth,
                                format!("{}: Pixel size for {} axis: {}", ui_box.id, axis, pixels),
                            );
                            ui_box.computed_size[screen_axis_index] = pixels as f32;
                        }
                        UISize::TextContent => {
                            let (texture_width, texture_height) = GLOBAL_UI_CONTEXT.with(|ctx| {
                                let font_info = ctx.font_info.borrow();
                                let mut text_cache = ctx.text_cache.borrow_mut();
                                let mut font_cache_rc = ctx.font_cache.borrow_mut();
                                let font_cache = font_cache_rc.as_mut().expect("Found a UIBox with `DrawText` feature enabled when `GLOBAL_UI_CONTEXT.font_cache` is `None`!");

                                let text_content = ui_box.text_content.as_ref().expect("Found a UIBox with `DrawText` feature enabled, with no `text_content` set!");
                
                                cache_text(font_cache, &mut text_cache, &font_info, text_content)
                            });

                            // Layout direction has no effect on the screen
                            // dimensions of the rendered text (no text rotations).

                            if is_horizontal_axis {
                                ui_box.computed_size[0] = texture_width as f32;
    
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Rendered text is {} pixels wide.",
                                        ui_box.id,
                                        texture_width
                                    ),
                                );
                            } else {
                                ui_box.computed_size[1] = texture_height as f32;
                                
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Rendered text is {} pixels tall.",
                                        ui_box.id,
                                        texture_height
                                    ),
                                );
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

        // 2. Calculate sibling-dependent sizes with pre-order traversal.

        debug_print!(">\n> (Sibling-dependent pass...)\n>");

        self.visit_root_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |_depth, parent_data, node| {
            let ui_box = &mut node.data;

            let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

            for (axis_index, _size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                let axis = UI2DAxis::from_usize(axis_index);
                
                let is_horizontal_axis = match (axis, parent_layout_direction) {
                    (UI2DAxis::Primary, UILayoutDirection::LeftToRight) | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                    (UI2DAxis::Primary, UILayoutDirection::TopToBottom) | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false
                };

                let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

                if node.children.is_empty() {
                    return Ok(());
                }

                let child_sizes_along_axis = node
                    .children
                    .iter()
                    .map(|c| c.borrow().data.computed_size[screen_axis_index]);

                let max = child_sizes_along_axis
                    .into_iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                for child in &mut node.children {
                    let child_ui_box = &mut child.borrow_mut().data;

                    let corresponding_child_axis = match (ui_box.layout_direction, is_horizontal_axis) {
                        (UILayoutDirection::TopToBottom, true) | (UILayoutDirection::LeftToRight, false) => 1,
                        (UILayoutDirection::TopToBottom, false) | (UILayoutDirection::LeftToRight, true) => 0,
                    };

                    let child_size_along_corresponding_child_axis = child_ui_box.semantic_sizes[corresponding_child_axis];
    
                    if matches!(child_size_along_corresponding_child_axis.size, UISize::MaxOfSiblings) {
                        child_ui_box.computed_size[screen_axis_index] = max;
                    }
                }
            }

            Ok(())
        })?;

        // 3. Calculate upward-dependent sizes with a pre-order traversal.

        debug_print!(">\n> (Upward-dependent sizes pass...)\n>");

        self.visit_root_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, parent_data, node| {
            let ui_box = &mut node.data;

            let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

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

            let grandparent_layout_direction = parent_data.unwrap().parent_layout_direction;

            for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                let axis = UI2DAxis::from_usize(axis_index);
                
                let is_horizontal_axis = match (axis, parent_layout_direction) {
                    (UI2DAxis::Primary, UILayoutDirection::LeftToRight) | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                    (UI2DAxis::Primary, UILayoutDirection::TopToBottom) | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false
                };

                let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

                let corresponding_parent_axis_index = match (is_horizontal_axis, grandparent_layout_direction) {
                    (true, UILayoutDirection::TopToBottom) | (false, UILayoutDirection::LeftToRight)=> 1,
                    (true, UILayoutDirection::LeftToRight) | (false, UILayoutDirection::TopToBottom) => 0,
                };

                match size_with_strictness.size {
                    UISize::PercentOfParent(percentage) => match parent_data {
                        Some(parent) => {
                            let parent_size_for_axis = parent.semantic_sizes[corresponding_parent_axis_index].size;

                            match parent_size_for_axis {
                                UISize::ChildrenSum => {
                                    panic!(
                                        "{}: Uses {} size for {} axis, but parent uses downward-dependent {} size for same axis.",
                                        ui_box.id, size_with_strictness.size, axis,
                                        parent_size_for_axis
                                    );
                                },
                                UISize::Null => panic!("{}: Parent node has `Null` size for screen axis {}!", ui_box.id, if screen_axis_index == 0 { "X" } else { "Y" }),
                                UISize::Pixels(_) | UISize::TextContent | UISize::PercentOfParent(_) | UISize::MaxOfSiblings => {
                                    ui_box.computed_size[screen_axis_index] = parent.computed_size[screen_axis_index] * percentage;

                                    println_indent(
                                        depth,
                                        format!(
                                            "{}: ({} axis) Computed size {} as {} percent of parent size {}",
                                            ui_box.id,
                                            axis,
                                            ui_box.computed_size[screen_axis_index], 
                                            percentage * 100.0,
                                            parent.computed_size[screen_axis_index]
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

        // 4. Calculate downward-dependent sizes with a post-order traversal.

        debug_print!(">\n> (Downward-dependent sizes pass...)\n>");

        self.visit_root_dfs_mut(&NodeLocalTraversalMethod::PostOrder, &mut |depth, parent_data, node| {
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

            let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

            for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                let axis = UI2DAxis::from_usize(axis_index);

                let is_horizontal_axis = match (axis, parent_layout_direction) {
                    (UI2DAxis::Primary, UILayoutDirection::LeftToRight) | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                    (UI2DAxis::Primary, UILayoutDirection::TopToBottom) | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false
                };

                let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

                match size_with_strictness.size {
                    UISize::ChildrenSum => {
                        let size_of_children_along_axis = {
                            let child_sizes_along_axis = node
                                .children
                                .iter()
                                .map(|c| c.borrow().data.computed_size[screen_axis_index]);

                            match (ui_box.layout_direction, screen_axis_index) {
                                (UILayoutDirection::LeftToRight, 0) | (UILayoutDirection::TopToBottom, 1) => {
                                    child_sizes_along_axis.into_iter().sum()
                                },
                                _ => {
                                    child_sizes_along_axis
                                    .into_iter()
                                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                                    .unwrap()
                                }
                            }
                        };

                        ui_box.computed_size[screen_axis_index] = size_of_children_along_axis;

                        println_indent(
                            depth,
                            format!(
                                "{}: ({} axis) Computed box size {} as the sum of its children's sizes.",
                                ui_box.id,
                                axis,
                                ui_box.computed_size[screen_axis_index], 
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

        // 5. Solve any violations (children extending beyond parent) with a pre-order traversal.

        debug_print!(">\n> (Violations pass...)\n>");

        self.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, parent_data, node| {
                let ui_box = &mut node.data;

                if node.children.is_empty() {
                    println_indent(depth, format!("{}: Skipping (leaf node).", ui_box.id,));

                    return Ok(());
                }

                let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

                for (axis_index, size_with_strictness) in ui_box.semantic_sizes.iter().enumerate() {
                    let axis = UI2DAxis::from_usize(axis_index);

                    let is_horizontal_axis = match (axis, parent_layout_direction) {
                        (UI2DAxis::Primary, UILayoutDirection::LeftToRight) | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                        (UI2DAxis::Primary, UILayoutDirection::TopToBottom) | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false
                    };

                    let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

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
                        UISize::Pixels(_) | UISize::PercentOfParent(_) | UISize::MaxOfSiblings => {
                            let computed_size_along_axis = ui_box.computed_size[screen_axis_index];

                            let size_of_children_along_axis = {
                                let child_sizes_along_axis = node
                                    .children
                                    .iter()
                                    .map(|c| c.borrow().data.computed_size[screen_axis_index]);

                                match (ui_box.layout_direction, screen_axis_index) {
                                    (UILayoutDirection::LeftToRight, 0) | (UILayoutDirection::TopToBottom, 1) => {
                                        child_sizes_along_axis.into_iter().sum()
                                    },
                                    _ => {
                                        child_sizes_along_axis
                                        .into_iter()
                                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                                        .unwrap()
                                    }
                                }
                            };
                            
                            if computed_size_along_axis < size_of_children_along_axis {
                                println_indent(
                                    depth,
                                    format!(
                                        "{}: Detected size violation of children ({} > {}!).",
                                        ui_box.id,
                                        size_of_children_along_axis,
                                        computed_size_along_axis,
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

                                let size_reserved_for_strict_children: f32 =  match (ui_box.layout_direction, screen_axis_index) {
                                    (UILayoutDirection::LeftToRight, 0) | (UILayoutDirection::TopToBottom, 1) => {
                                        node
                                            .children
                                            .iter()
                                            .map(|child| {
                                                let child_ui_box = &child.borrow().data;

                                                let child_strictness =
                                                    child_ui_box.semantic_sizes[0].strictness;

                                                if child_strictness == 1.0 {
                                                    child_ui_box.computed_size[screen_axis_index]
                                                } else {
                                                    0.0
                                                }
                                            })
                                            .sum()
                                    },
                                    _ => {
                                        0.0
                                    }
                                };

                                let alpha_adjusted_for_size_reserved =
                                    (computed_size_along_axis - size_reserved_for_strict_children) / (size_of_children_along_axis - size_reserved_for_strict_children);

                                for child in &node.children {
                                    let child_ui_box = &mut child.borrow_mut().data;

                                    let old_child_size = child_ui_box.computed_size[screen_axis_index];

                                    let strictness =
                                        child_ui_box.semantic_sizes[0].strictness;

                                    let new_child_size = if strictness == 1.0 {
                                        old_child_size
                                    } else if strictness == 0.0 {
                                        old_child_size * alpha_adjusted_for_size_reserved
                                    } else {
                                        panic!()
                                    };

                                    if new_child_size != old_child_size {
                                        println_indent(
                                            depth + 1,
                                            format!(
                                                "{}: ({} axis) Scaling down from {} to {} (strictness: {}).",
                                                child_ui_box.id,
                                                axis,
                                                old_child_size,
                                                new_child_size,
                                                strictness,
                                            ),
                                        );
                                    }

                                    child_ui_box.computed_size[screen_axis_index] = new_child_size;
                                }
                            }
                        }
                    }
                }

                Ok(())
            },
        )?;

        // 6. Compute the relative positions of each child with a pre-order traversal.

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

                let parent_layout_direction = if let Some(parent) = parent_data {
                    parent.layout_direction
                } else {
                    UILayoutDirection::default()
                };

                for (axis_index, _size_with_strictness) in ui_box.semantic_sizes.iter().enumerate()
                {
                    let axis = UI2DAxis::from_usize(axis_index);

                    let is_horizontal_axis = match (axis, parent_layout_direction) {
                        (UI2DAxis::Primary, UILayoutDirection::LeftToRight)
                        | (UI2DAxis::Secondary, UILayoutDirection::TopToBottom) => true,
                        (UI2DAxis::Primary, UILayoutDirection::TopToBottom)
                        | (UI2DAxis::Secondary, UILayoutDirection::LeftToRight) => false,
                    };

                    let screen_axis_index = if is_horizontal_axis { 0 } else { 1 };

                    let mut cursor = 0.0;

                    for child_node_rc in &node.children {
                        let mut child_node = (*child_node_rc).borrow_mut();
                        let child_ui_box = &mut child_node.data;

                        child_ui_box.computed_relative_position[screen_axis_index] = cursor;

                        match (ui_box.layout_direction, is_horizontal_axis) {
                            (UILayoutDirection::TopToBottom, true)
                            | (UILayoutDirection::LeftToRight, false) => (),
                            (UILayoutDirection::TopToBottom, false)
                            | (UILayoutDirection::LeftToRight, true) => {
                                cursor += child_ui_box.computed_size[screen_axis_index];
                            }
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

    pub fn push(&mut self, mut ui_box: UIBox) -> Result<(), String> {
        let new_child_node_rc: Rc<RefCell<Node<'a, UIBox>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            if let UISize::TextContent = &current_node.data.semantic_sizes[0].size {
                return Err(
                    "Called UIBoxTree::push_parent() when current node uses TextContent size!"
                        .to_string(),
                );
            }

            ui_box.parent_layout_direction = current_node.data.layout_direction;

            let mut new_child_node = Node::<'a, UIBox>::new(ui_box);

            new_child_node.parent = Some(current_node_rc.clone());

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            debug_assert!(self.root.is_none());

            ui_box.parent_layout_direction = UILayoutDirection::default();

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