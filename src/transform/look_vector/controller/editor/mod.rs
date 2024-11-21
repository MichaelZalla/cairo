use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};

use sdl2::{keyboard::Keycode, mouse::MouseButton};

use crate::{
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    time::TimingInfo,
    transform::{look_vector::LookVector, quaternion::Quaternion},
    vec::vec3,
};

use super::LookVectorController;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct EditorLookVectorController {
    mouse_pan_sensitivity: f32,
    mouse_orbit_sensitivity: f32,
}

impl Default for EditorLookVectorController {
    fn default() -> Self {
        Self {
            mouse_pan_sensitivity: 0.2,
            mouse_orbit_sensitivity: 0.0075,
        }
    }
}

impl LookVectorController for EditorLookVectorController {
    fn update(
        &mut self,
        look_vector: &mut LookVector,
        _timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: Option<&MouseState>,
        _game_controller_state: &GameControllerState,
        _movement_speed: f32,
    ) {
        if let Some(mouse_state) = mouse_state {
            // Moves camera forward or backward using mouse wheel.

            let is_shift_pressed = keyboard_state.pressed_keycodes.contains(&Keycode::LShift)
                || keyboard_state.pressed_keycodes.contains(&Keycode::RShift);

            if !is_shift_pressed {
                if let Some(wheel_event) = mouse_state.wheel_event.as_ref() {
                    let delta = wheel_event.delta as f32;

                    look_vector
                        .set_position(look_vector.position + look_vector.get_forward() * delta);
                }
            }

            if let Some(drag_event) = mouse_state.drag_events.get(&MouseButton::Middle) {
                let delta_x = drag_event.delta.0 as f32;
                let delta_y = drag_event.delta.1 as f32;

                if is_shift_pressed {
                    // Pans camera left or right using Shift and middle mouse drag.

                    look_vector.set_position(
                        look_vector.position
                            + -look_vector.get_right() * delta_x * self.mouse_pan_sensitivity,
                    );

                    look_vector.set_position(
                        look_vector.position
                            + look_vector.get_up() * delta_y * self.mouse_pan_sensitivity,
                    );
                } else {
                    // Rotates (orbits) camera around target using middle mouse drag.

                    let rotation = {
                        let rotation_around_y = Quaternion::new(
                            vec3::UP,
                            TAU * -delta_x * self.mouse_orbit_sensitivity,
                        );

                        let rotation_around_x = Quaternion::new(
                            vec3::RIGHT,
                            TAU * -delta_y * self.mouse_orbit_sensitivity,
                        );

                        rotation_around_y * rotation_around_x
                    };

                    let global_to_target_local = -look_vector.get_target();

                    let position_local = look_vector.position + global_to_target_local;

                    let position_local_rotated = position_local * *rotation.mat();

                    let position_global_rotated = position_local_rotated - global_to_target_local;

                    look_vector.set_position(position_global_rotated);

                    look_vector.set_target(look_vector.get_target());
                }
            }
        }
    }
}
