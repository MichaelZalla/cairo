use sdl2::mouse::MouseButton;

use crate::{device::mouse::MouseEventKind, ui::context::UIInputEvents};

use super::UIBox;

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

    // pub is_dragging: bool,
}

#[derive(Default, Debug, Clone)]
pub struct UIBoxInteraction {
    // ui_box: Rc<RefCell<Node<'a, UIBox>>>,
    pub mouse_interaction_in_bounds: UIMouseInteraction,
}

impl UIBoxInteraction {
    pub fn from_user_inputs(
        ui_box_previous_frame: Option<&UIBox>,
        input_events: &UIInputEvents,
    ) -> Self {
        let is_hovering = match ui_box_previous_frame {
            Some(ui_box_prev) => ui_box_prev.global_bounds.contains(
                input_events.mouse.position.0 as u32,
                input_events.mouse.position.1 as u32,
            ),
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

        Self {
            mouse_interaction_in_bounds,
        }
    }
}
