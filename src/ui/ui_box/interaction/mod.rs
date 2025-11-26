use std::mem;

use sdl2::mouse::MouseButton;

use crate::{
    device::mouse::{MouseDragEvent, MouseEventKind},
    graphics::{self},
    ui::context::UIInputEvents,
};

use super::{
    UIBox, UIBoxDragHandle, UIBoxFeatureFlags, UILayoutDirection,
    UI_DIVIDER_CURSOR_SNAP_EPSILON,
};

#[derive(Default, Debug, Clone)]
pub struct UIMouseInteraction {
    pub is_hovering: bool,

    // Left mouse button.
    pub was_left_pressed: bool,
    pub is_left_down: bool,
    pub was_left_released: bool,
    // pub was_left_double_clicked: bool,

    // Middle mouse button.
    pub was_middle_pressed: bool,
    pub is_middle_down: bool,
    pub was_middle_released: bool,
    // pub was_middle_double_clicked: bool,

    // Right mouse button.
    pub was_right_pressed: bool,
    pub is_right_down: bool,
    pub was_right_released: bool,
    // pub was_right_double_clicked: bool,

    // Drag event
    pub drag_event: Option<MouseDragEvent>,

    // Drag handles
    pub hot_drag_handle: Option<UIBoxDragHandle>,
    pub active_drag_handle: Option<UIBoxDragHandle>,
}

#[derive(Default, Debug, Clone)]
pub struct UIBoxInteraction {
    pub was_active: bool,
    pub was_focused: bool,
    pub was_hot: bool,
    pub mouse_interaction_in_bounds: UIMouseInteraction,
}

impl UIBoxInteraction {
    pub fn from_user_inputs(
        features: &UIBoxFeatureFlags,
        ui_box_previous_frame: Option<&UIBox>,
        input_events: &UIInputEvents,
    ) -> Self {
        let mouse = &input_events.mouse;

        let is_hovering = match ui_box_previous_frame {
            Some(ui_box_prev) => {
                ui_box_prev.contains_screen_pixel(mouse.position.0, mouse.position.1)
            }
            None => false,
        };

        let mut mouse_interaction_in_bounds = UIMouseInteraction {
            is_hovering,
            ..Default::default()
        };

        if is_hovering {
            let mouse = &input_events.mouse;

            if mouse.buttons_down.contains(&MouseButton::Left) {
                mouse_interaction_in_bounds.is_left_down = true;
            }

            if mouse.buttons_down.contains(&MouseButton::Middle) {
                mouse_interaction_in_bounds.is_middle_down = true;
            }

            if mouse.buttons_down.contains(&MouseButton::Right) {
                mouse_interaction_in_bounds.is_right_down = true;
            }

            if let Some(event) = mouse.button_event {
                match (event.button, event.kind) {
                    (MouseButton::Left, MouseEventKind::Down) => {
                        mouse_interaction_in_bounds.was_left_pressed = true;
                    }
                    (MouseButton::Left, MouseEventKind::Up) => {
                        mouse_interaction_in_bounds.was_left_released = true;
                    }
                    (MouseButton::Middle, MouseEventKind::Down) => {
                        mouse_interaction_in_bounds.was_middle_pressed = true;
                    }
                    (MouseButton::Middle, MouseEventKind::Up) => {
                        mouse_interaction_in_bounds.was_middle_released = true;
                    }
                    (MouseButton::Right, MouseEventKind::Down) => {
                        mouse_interaction_in_bounds.was_right_pressed = true;
                    }
                    (MouseButton::Right, MouseEventKind::Up) => {
                        mouse_interaction_in_bounds.was_right_released = true;
                    }
                    _ => (),
                }
            }
        }

        // Drag events

        {
            let mouse = &input_events.mouse;

            if let (Some(_), Some(ui_box_prev)) = (
                &mouse.drag_events.get(&MouseButton::Left),
                ui_box_previous_frame,
            ) {
                // Check that the drag event originated from inside the UIBox's extent.

                if ui_box_prev.contains_screen_pixel(mouse.prev_position.0, mouse.prev_position.1) {
                    let drag_start_relative = (
                        mouse.prev_position.0 - ui_box_prev.global_bounds.left as i32,
                        mouse.prev_position.1 - ui_box_prev.global_bounds.top as i32,
                    );

                    let drag_end_relative = (
                        mouse.position.0 - ui_box_prev.global_bounds.left as i32,
                        mouse.position.1 - ui_box_prev.global_bounds.top as i32,
                    );

                    // Clip the drag delta to this UIBox's screen extent.

                    let container_width = ui_box_prev.computed_size[0] as u32;
                    let container_height = ui_box_prev.computed_size[1] as u32;

                    if let Some(result) = graphics::line::clip_line(
                        container_width,
                        container_height,
                        drag_start_relative.0,
                        drag_start_relative.1,
                        drag_end_relative.0,
                        drag_end_relative.1,
                    ) {
                        let mut clipped_start = result.left;
                        let mut clipped_end = result.right;

                        if result.did_swap {
                            mem::swap(&mut clipped_start, &mut clipped_end);
                        }

                        let clipped_drag_delta = (
                            clipped_end.0 - clipped_start.0,
                            clipped_end.1 - clipped_start.1,
                        );

                        mouse_interaction_in_bounds.drag_event = Some(MouseDragEvent {
                            delta: clipped_drag_delta,
                        })
                    }
                }
            }
        }

        // Drag handles

        mouse_interaction_in_bounds.hot_drag_handle = match (
            ui_box_previous_frame,
            features.intersects(
                UIBoxFeatureFlags::RESIZABLE_MIN_EXTENT_ON_PRIMARY_AXIS
                    | UIBoxFeatureFlags::RESIZABLE_MAX_EXTENT_ON_PRIMARY_AXIS
                    | UIBoxFeatureFlags::RESIZABLE_MIN_EXTENT_ON_SECONDARY_AXIS
                    | UIBoxFeatureFlags::RESIZABLE_MAX_EXTENT_ON_SECONDARY_AXIS,
            ),
        ) {
            (None, _) | (Some(_), false) => None,
            (Some(previous_frame), true) => {
                // Set drag cursor if it's within epislon.

                let mouse_position = input_events.mouse.position;

                let (mouse_x, mouse_y) = (mouse_position.0, mouse_position.1);

                let (min_primary, max_primary, min_secondary, max_secondary) =
                    match previous_frame.parent_layout_direction {
                        UILayoutDirection::TopToBottom => (
                            previous_frame.global_bounds.top as i32,
                            previous_frame.global_bounds.bottom as i32,
                            previous_frame.global_bounds.left as i32,
                            previous_frame.global_bounds.right as i32,
                        ),
                        UILayoutDirection::LeftToRight => (
                            previous_frame.global_bounds.left as i32,
                            previous_frame.global_bounds.right as i32,
                            previous_frame.global_bounds.top as i32,
                            previous_frame.global_bounds.bottom as i32,
                        ),
                    };

                let (mouse_primary, mouse_secondary) = match previous_frame.parent_layout_direction
                {
                    UILayoutDirection::TopToBottom => (mouse_y, mouse_x),
                    UILayoutDirection::LeftToRight => (mouse_x, mouse_y),
                };

                let (
                    drag_handle_min_primary,
                    drag_handle_max_primary,
                    drag_handle_min_secondary,
                    drag_handle_max_secondary,
                ) = match previous_frame.parent_layout_direction {
                    UILayoutDirection::TopToBottom => (
                        UIBoxDragHandle::Top,
                        UIBoxDragHandle::Bottom,
                        UIBoxDragHandle::Left,
                        UIBoxDragHandle::Right,
                    ),
                    UILayoutDirection::LeftToRight => (
                        UIBoxDragHandle::Left,
                        UIBoxDragHandle::Right,
                        UIBoxDragHandle::Top,
                        UIBoxDragHandle::Bottom,
                    ),
                };

                if features.contains(UIBoxFeatureFlags::RESIZABLE_MIN_EXTENT_ON_PRIMARY_AXIS)
                    && within_epsilon(mouse_primary, min_primary)
                    && (min_secondary..max_secondary + 1).contains(&mouse_secondary)
                {
                    Some(drag_handle_min_primary)
                } else if features.contains(UIBoxFeatureFlags::RESIZABLE_MAX_EXTENT_ON_PRIMARY_AXIS)
                    && within_epsilon(mouse_primary, max_primary)
                    && (min_secondary..max_secondary + 1).contains(&mouse_secondary)
                {
                    Some(drag_handle_max_primary)
                } else if features.contains(UIBoxFeatureFlags::RESIZABLE_MIN_EXTENT_ON_SECONDARY_AXIS)
                    && within_epsilon(mouse_secondary, min_secondary)
                    && (min_primary..max_primary + 1).contains(&mouse_primary)
                {
                    Some(drag_handle_min_secondary)
                } else if features.contains(UIBoxFeatureFlags::RESIZABLE_MAX_EXTENT_ON_SECONDARY_AXIS)
                    && within_epsilon(mouse_secondary, max_secondary)
                    && (min_primary..max_primary + 1).contains(&mouse_primary)
                {
                    Some(drag_handle_max_secondary)
                } else {
                    None
                }
            }
        };

        mouse_interaction_in_bounds.active_drag_handle = match ui_box_previous_frame {
            // Check if we had an active drag handle in the previous frame.
            Some(prev_frame) => match &prev_frame.active_drag_handle {
                Some(prev_active_handle) => {
                    match &input_events.mouse.button_event {
                        Some(event) => {
                            // Check if we've released the mouse this frame.

                            if matches!(
                                (event.button, event.kind),
                                (MouseButton::Left, MouseEventKind::Up)
                            ) {
                                None
                            } else {
                                Some(*prev_active_handle)
                            }
                        }
                        None => {
                            // Otherwise, keep the handle active.

                            Some(*prev_active_handle)
                        }
                    }
                }
                None => match &mouse_interaction_in_bounds.hot_drag_handle {
                    Some(hot_handle) => match &input_events.mouse.button_event {
                        Some(event) => {
                            if matches!(
                                (event.button, event.kind),
                                (MouseButton::Left, MouseEventKind::Down)
                            ) {
                                Some(*hot_handle)
                            } else {
                                None
                            }
                        }
                        None => None,
                    },
                    None => None,
                },
            },
            None => None,
        };

        let (was_active, was_focused, was_hot) = if let Some(prev_frame) = ui_box_previous_frame {
            (prev_frame.active, prev_frame.focused, prev_frame.hot)
        } else {
            (false, false, false)
        };

        Self {
            was_active,
            was_focused,
            was_hot,
            mouse_interaction_in_bounds,
        }
    }
}

fn within_epsilon(mouse_along_axis: i32, target_along_axis: i32) -> bool {
    (mouse_along_axis - target_along_axis).abs() < UI_DIVIDER_CURSOR_SNAP_EPSILON
}
