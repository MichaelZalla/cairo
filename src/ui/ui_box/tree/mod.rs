use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    buffer::Buffer2D,
    collections::tree::{
        node::{Node, NodeLocalTraversalMethod},
        Tree,
    },
    color,
    graphics::{text::cache::cache_text, Graphics},
    ui::{
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        ui_box::{UIBoxFeatureFlag, UILayoutDirection},
        UI2DAxis, UISize,
    },
};

use super::{key::UIKey, UIBox, UIBoxInteraction};

#[cfg(feature = "print_ui_layout_info")]
macro_rules! ui_debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[cfg(not(feature = "print_ui_layout_info"))]
macro_rules! ui_debug_print {
    ($( $args:expr ),*) => {};
}

#[cfg(feature = "print_ui_layout_info")]
macro_rules! ui_debug_print_indented {
    ($depth: expr, $msg: expr) => {
        let indent = 2 * ($depth + 1);

        ui_debug_print!("{:indent$}{}", ">", $msg);
    };
}

#[cfg(not(feature = "print_ui_layout_info"))]
macro_rules! ui_debug_print_indented {
    ($depth: expr, $msg: expr) => {};
}

pub type UIBoxTreeRenderCallback = Rc<dyn Fn(&mut UIBoxTree) -> Result<(), String>>;

#[derive(Default, Debug, Clone)]
pub struct FocusedTransitionInfo {
    pub transition: f32,
    pub from_rect: ScreenExtent,
    pub current_rect: ScreenExtent,
}

#[derive(Default, Debug, Clone)]
pub struct UIBoxTree<'a> {
    tree: Tree<'a, UIBox>,
    pub focused_transition: RefCell<FocusedTransitionInfo>,
    pub next_focused_key: RefCell<Option<UIKey>>,
}

impl<'a> fmt::Display for UIBoxTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tree.fmt(f)
    }
}

impl<'a> UIBoxTree<'a> {
    pub fn get_current(&self) -> Option<&Rc<RefCell<Node<'a, UIBox>>>> {
        self.tree.get_current()
    }

    pub fn clear(&mut self) {
        self.tree.clear();

        // Resets "next" focused key for this UI tree.

        self.next_focused_key.borrow_mut().take();
    }

    pub fn push(&mut self, mut ui_box: UIBox) -> Result<UIBoxInteraction, String> {
        // Validations.

        if let Some(current_node_rc) = self.tree.get_current() {
            let current_node = &current_node_rc.borrow();
            let current_ui_box = &current_node.data;

            if let UISize::TextContent = &current_ui_box.semantic_sizes[0].size {
                return Err(
                    "Called UIBoxTree::push_parent() when current node uses TextContent size!"
                        .to_string(),
                );
            }

            ui_box.parent_layout_direction = current_ui_box.layout_direction;
        } else {
            ui_box.parent_layout_direction = UILayoutDirection::default();
        }

        let interaction_result = GLOBAL_UI_CONTEXT.with(|ctx| {
            let cache = ctx.cache.borrow();

            let input_events = ctx.input_events.borrow();

            let seconds_since_last_update = *ctx.seconds_since_last_update.borrow();

            let interaction_result = match cache.get(&ui_box.key) {
                Some(ui_box_previous_frame) => UIBoxInteraction::from_user_inputs(
                    &ui_box.features,
                    Some(ui_box_previous_frame),
                    &input_events,
                ),
                None => UIBoxInteraction::from_user_inputs(&ui_box.features, None, &input_events),
            };

            ui_box.hot_drag_handle = interaction_result
                .mouse_interaction_in_bounds
                .hot_drag_handle;

            ui_box.active_drag_handle = interaction_result
                .mouse_interaction_in_bounds
                .active_drag_handle;

            // Updates hot state for this node, based on the node's previous
            // layout (from the prior frame).

            if ui_box.features.contains(UIBoxFeatureFlag::Hoverable) {
                ui_box.update_hot_state(seconds_since_last_update, &interaction_result);
            }

            interaction_result
        });

        self.tree.push(ui_box)?;

        Ok(interaction_result)
    }

    pub fn push_parent(&mut self, ui_box: UIBox) -> Result<UIBoxInteraction, String> {
        let interaction_result = self.push(ui_box)?;

        self.tree.push_parent_post();

        Ok(interaction_result)
    }

    pub fn pop_parent(&mut self) -> Result<(), String> {
        self.tree.pop_parent()
    }

    pub fn do_active_focused_pass(&mut self) -> Result<(), String> {
        // @TODO This stuff needs to happen in MakeWidget() calls! Immediately
        // return the user interaction result from each call, as we're building
        // this frame's UI tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            ui_debug_print!("\nUser inputs pass:\n");

            let mut input_events = ctx.input_events.borrow_mut();

            let seconds_since_last_update = *ctx.seconds_since_last_update.borrow();

            let mut next_focused_key = self.next_focused_key.borrow_mut();

            self.tree
                .visit_root_dfs_mut(
                    &NodeLocalTraversalMethod::PostOrder,
                    &mut |_depth, _sibling_index, _parent_data, node| {
                        let ui_box: &mut UIBox = &mut node.data;

                        if !ui_box.features.contains(UIBoxFeatureFlag::Clickable) {
                            return Ok(());
                        }

                        // Apply the latest user inputs, based on this node's previous layout
                        // (from the previous frame).

                        let was_just_focused = ui_box.update_active_state(
                            seconds_since_last_update,
                            &mut input_events.mouse,
                        ) && !ui_box.focused;

                        if was_just_focused {
                            next_focused_key.replace(ui_box.key.clone());
                        }

                        Ok(())
                    },
                    &mut |_node| {},
                )
                .unwrap();

            self.tree
                .visit_root_dfs_mut(
                    &NodeLocalTraversalMethod::PreOrder,
                    &mut |_depth, _sibling_index, _parent, node| {
                        let ui_box = &mut node.data;

                        if !ui_box.features.contains(UIBoxFeatureFlag::Clickable) {
                            return Ok(());
                        }

                        let mut focused_transition_info = self.focused_transition.borrow_mut();

                        ui_box.update_focused_state(
                            &next_focused_key,
                            &mut focused_transition_info,
                            seconds_since_last_update,
                        );

                        Ok(())
                    },
                    &mut |_node| {},
                )
                .unwrap();
        });

        Ok(())
    }

    pub fn do_autolayout_pass(&mut self) -> Result<(), String> {
        ui_debug_print!("\nAuto-layout pass:\n");

        // For each axis...

        // 1. Calculate "standalone" sizes.

        ui_debug_print!(">\n> (Standalone sizes pass...)\n>");

        #[allow(unused)]
        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _sibling_index, parent_data, node| {
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
                            ui_debug_print_indented!(
                                depth,
                                format!("{}: Pixel size for {} axis: {}", ui_box.id, axis, pixels)
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

                                ui_debug_print_indented!(
                                    depth,
                                    format!(
                                        "{}: Rendered text is {} pixels wide.",
                                        ui_box.id,
                                        texture_width
                                    )
                                );
                            } else {
                                ui_box.computed_size[1] = texture_height as f32;

                                ui_debug_print_indented!(
                                    depth,
                                    format!(
                                        "{}: Rendered text is {} pixels tall.",
                                        ui_box.id,
                                        texture_height
                                    )
                                );
                            }
                        },
                        _ => {
                            ui_debug_print_indented!(
                                depth,
                                format!(
                                    "{}: Uses {} size for {} axis. Skipping.",
                                    ui_box.id, size_with_strictness.size, axis
                                )
                            );
                        },
                    }
                }

                Ok(())
            },
            &mut |_node| {}
        )?;

        // 2. Calculate sibling-dependent sizes with pre-order traversal.

        ui_debug_print!(">\n> (Sibling-dependent pass...)\n>");

        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, _sibling_index, parent_data, node| {
                let ui_box: &mut UIBox = &mut node.data;

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

                        let corresponding_child_axis =
                            match (ui_box.layout_direction, is_horizontal_axis) {
                                (UILayoutDirection::TopToBottom, true)
                                | (UILayoutDirection::LeftToRight, false) => 1,
                                (UILayoutDirection::TopToBottom, false)
                                | (UILayoutDirection::LeftToRight, true) => 0,
                            };

                        let child_size_along_corresponding_child_axis =
                            child_ui_box.semantic_sizes[corresponding_child_axis];

                        if matches!(
                            child_size_along_corresponding_child_axis.size,
                            UISize::MaxOfSiblings
                        ) {
                            // println!("Box {} has child {} using MaxOfSiblings! (max = {}).", ui_box.id, child_ui_box.id, max);

                            child_ui_box.computed_size[screen_axis_index] = max;
                        }
                    }
                }

                Ok(())
            },
            &mut |_node| {},
        )?;

        // 3. Calculate upward-dependent sizes with a pre-order traversal.

        ui_debug_print!(">\n> (Upward-dependent sizes pass...)\n>");

        #[allow(unused)]
        self.tree.visit_root_dfs_mut(&NodeLocalTraversalMethod::PreOrder, &mut |depth, _sibling_index, parent_data, node| {
            let ui_box: &mut UIBox = &mut node.data;

            let parent_layout_direction = if let Some(parent) = parent_data { parent.layout_direction } else { UILayoutDirection::default() };

            if node.parent.is_none() {
                ui_debug_print_indented!(
                    depth,
                    format!(
                        "{}: Skipping (root node).",
                        ui_box.id,
                    )
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
                    (true, UILayoutDirection::TopToBottom) | (false, UILayoutDirection::LeftToRight) => 1,
                    (true, UILayoutDirection::LeftToRight) | (false, UILayoutDirection::TopToBottom) => 0,
                };

                if let UISize::PercentOfParent(percentage) = size_with_strictness.size {
                    match parent_data {
                    Some(parent) => {

                        let parent_size_for_axis = parent.semantic_sizes[corresponding_parent_axis_index].size;

                        match parent_size_for_axis {
                            UISize::ChildrenSum => {
                                panic!(
                                    "{}: Uses {} size for {} axis, but parent {} uses downward-dependent {} size for same axis.",
                                    ui_box.id, size_with_strictness.size, axis,
                                    parent.id,
                                    parent_size_for_axis
                                );
                            },
                            UISize::Null => panic!("{}: Parent node has `Null` size for screen axis {}!", ui_box.id, if screen_axis_index == 0 { "X" } else { "Y" }),
                            UISize::Pixels(_) | UISize::TextContent | UISize::PercentOfParent(_) | UISize::MaxOfSiblings => {
                                ui_box.computed_size[screen_axis_index] = parent.computed_size[screen_axis_index] * percentage;

                                ui_debug_print_indented!(
                                    depth,
                                    format!(
                                        "{}: ({} axis) Computed size {} as {} percent of parent size {}",
                                        ui_box.id,
                                        axis,
                                        ui_box.computed_size[screen_axis_index],
                                        percentage * 100.0,
                                        parent.computed_size[screen_axis_index]
                                    )
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
                }
                } else {
                    ui_debug_print_indented!(
                        depth,
                        format!(
                            "{}: Uses {} size for {} axis. Skipping.",
                            ui_box.id, size_with_strictness.size, axis
                        )
                    );
                }
            }

            Ok(())
        }, &mut |_node| {})?;

        // 4. Calculate downward-dependent sizes with a post-order traversal.

        ui_debug_print!(">\n> (Downward-dependent sizes pass...)\n>");

        #[allow(unused)]
        self.tree.visit_root_dfs_mut(&NodeLocalTraversalMethod::PostOrder, &mut |depth, _sibling_index, parent_data, node| {
            let ui_box: &mut UIBox = &mut node.data;

            if node.children.is_empty() {
                ui_debug_print_indented!(
                    depth,
                    format!(
                        "{}: Skipping (leaf node).",
                        ui_box.id,
                    )
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

                if let UISize::ChildrenSum = size_with_strictness.size {
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

                    ui_debug_print_indented!(
                        depth,
                        format!(
                            "{}: ({} axis) Computed box size {} as the sum of its children's sizes.",
                            ui_box.id,
                            axis,
                            ui_box.computed_size[screen_axis_index],
                        )
                    );
                } else {
                    ui_debug_print_indented!(
                        depth,
                        format!(
                            "{}: Uses {} size for {} axis. Skipping.",
                            ui_box.id, size_with_strictness.size, axis
                        )
                    );
                }
            }

            Ok(())
        }, &mut |_node| {})?;

        // 5. Solve any violations (children extending beyond parent) with a pre-order traversal.

        ui_debug_print!(">\n> (Violations pass...)\n>");

        #[allow(unused)]
        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _sibling_index, parent_data, node| {
                let ui_box: &mut UIBox = &mut node.data;

                if node.children.is_empty() {
                    ui_debug_print_indented!(depth, format!("{}: Skipping (leaf node).", ui_box.id,));

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

                    ui_debug_print_indented!(
                        depth + 1,
                        String::new()
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("id: {}", ui_box.id)
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("parent_layout: {}", ui_box.parent_layout_direction)
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("axis: {}", axis)
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("layout: {}", ui_box.layout_direction)
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("screen_axis_index: {}", screen_axis_index)
                    );

                    ui_debug_print_indented!(
                        depth + 1,
                        format!("is_horizontal_axis: {}", is_horizontal_axis)
                    );

                    match size_with_strictness.size {
                        UISize::Null | UISize::TextContent => panic!(),
                        UISize::ChildrenSum => {
                            ui_debug_print_indented!(
                                depth,
                                format!(
                                    "{}: Uses {} size. Skipping.",
                                    ui_box.id, ui_box.semantic_sizes[axis_index].size,
                                )
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

                            ui_debug_print_indented!(
                                depth + 1,
                                format!("computed_size_along_axis: {}", computed_size_along_axis)
                            );

                            ui_debug_print_indented!(
                                depth + 1,
                                format!("size_of_children_along_axis: {}", size_of_children_along_axis)
                            );

                            if computed_size_along_axis < size_of_children_along_axis {
                                ui_debug_print_indented!(
                                    depth,
                                    format!(
                                        "{}: Detected size violation of children ({} > {}!).",
                                        ui_box.id,
                                        size_of_children_along_axis,
                                        computed_size_along_axis,
                                    )
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

                                ui_debug_print_indented!(
                                    depth + 1,
                                    format!("size_reserved_for_strict_children: {}", size_reserved_for_strict_children)
                                );

                                ui_debug_print_indented!(
                                    depth + 1,
                                    format!("alpha_adjusted_for_size_reserved: {}", alpha_adjusted_for_size_reserved)
                                );

                                for child in &node.children {
                                    let child_ui_box = &mut child.borrow_mut().data;

                                    let old_child_size = child_ui_box.computed_size[screen_axis_index];

                                    ui_debug_print_indented!(
                                        depth + 1,
                                        format!("old_child_size for {}: {}", child_ui_box.id, old_child_size)
                                    );

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
                                        ui_debug_print_indented!(
                                            depth + 1,
                                            format!(
                                                "{}: ({} axis) Scaling down from {} to {} (strictness: {}).",
                                                child_ui_box.id,
                                                axis,
                                                old_child_size,
                                                new_child_size,
                                                strictness,
                                            )
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
            &mut |_node| {}
        )?;

        // 6. Compute the relative positions of each child with a pre-order traversal.

        ui_debug_print!(">\n> (Relative positioning pass...)\n>");

        self.tree.visit_root_dfs_mut(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, _sibling_index, parent_data, node| {
                let ui_box: &mut UIBox = &mut node.data;

                let mut global_bounds = ScreenExtent {
                    left: ui_box.computed_relative_position[0] as u32,
                    top: ui_box.computed_relative_position[1] as u32,
                    ..Default::default()
                };

                if let Some(parent) = parent_data {
                    global_bounds.left += parent.global_bounds.left;
                    global_bounds.top += parent.global_bounds.top;
                } else {
                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        let global_offset = ctx.global_offset.borrow();

                        global_bounds.left += global_offset.0;
                        global_bounds.top += global_offset.1;
                    });
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
            &mut |_node| {},
        )?;

        // Check our results.

        self.debug_computed_sizes()
    }

    fn debug_computed_sizes(&self) -> Result<(), String> {
        ui_debug_print!("\nResults:\n");

        #[allow(unused)]
        self.tree.visit_root_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |depth, _parent_data, node| {
                #[cfg(feature = "print_ui_layout_info")] {
                    let ui_box: &UIBox = &node.data;

                    let rel_position = ui_box.computed_relative_position;
                    let global_position = ui_box.global_bounds;
                    let size = ui_box.computed_size;

                    ui_debug_print_indented!(
                        depth,
                        format!(
                            "{}: Relative position: ({},{}) | Global position: ({},{}) | Computed size: {}x{}.",
                            ui_box.id, rel_position[0], rel_position[1], global_position.left, global_position.top, size[0], size[1],
                        )
                    );
                }

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn commit_frame(&mut self) -> Result<(), String> {
        self.do_active_focused_pass()?;

        self.do_autolayout_pass()
    }

    pub fn render_frame(&mut self, frame_index: u32, target: &mut Buffer2D) -> Result<(), String> {
        self.tree.visit_root_dfs(
            &NodeLocalTraversalMethod::PreOrder,
            &mut |_depth, _parent_data, node| {
                let ui_box: &UIBox = &node.data;

                // Render this node for the current frame (preorder).

                ui_box.render_preorder(target)?;

                if ui_box.features.contains(UIBoxFeatureFlag::DrawCustomRender) {
                    if let Some((panel_handle, render)) = &ui_box.custom_render_callback {
                        return render(panel_handle, &ui_box.global_bounds, target);
                    }
                }

                Ok(())
            },
        )?;

        self.tree.visit_root_dfs(
            &NodeLocalTraversalMethod::PostOrder,
            &mut |_depth, _parent_data, node| {
                let ui_box: &UIBox = &node.data;
                let children = &node.children;

                // 2. Render this node for the current frame.

                let render_result = ui_box.render_postorder(children, target);

                // 3. Update this node's cache entry (prepare for rendering the next frame).

                GLOBAL_UI_CONTEXT.with(|ctx| {
                    let mut cache = ctx.cache.borrow_mut();

                    update_cache_entry(&mut cache, ui_box, frame_index);
                });

                // Return the rendering result.

                render_result
            },
        )?;

        let focused_rect = self.focused_transition.borrow().current_rect;

        if focused_rect.left != focused_rect.right {
            let (x, y, width, height) = (
                focused_rect.left,
                focused_rect.top,
                focused_rect.right - focused_rect.left,
                focused_rect.bottom - focused_rect.top,
            );

            Graphics::rectangle(target, x, y, width, height, None, Some(&color::RED));

            Graphics::rectangle(
                target,
                x + 1,
                y + 1,
                width - 2,
                height - 2,
                None,
                Some(&color::RED),
            );
        }

        Ok(())
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

        cached_ui_box.focused = ui_box.focused;

        cached_ui_box.hot_drag_handle = ui_box.hot_drag_handle;
        cached_ui_box.active_drag_handle = ui_box.active_drag_handle;

        cached_ui_box.last_read_at_frame = frame_index;
    } else if !ui_box.key.is_null() {
        cache.insert(ui_box.key.clone(), ui_box.clone());
    }
}
