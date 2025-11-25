use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use sdl2::keyboard::{Keycode, Mod};

use crate::{
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    time::TimingInfo,
    transform::{look_vector::LookVector, quaternion::Quaternion},
    vec::{vec2::Vec2, vec3},
};

use super::LookVectorController;

static MOVEMENT_MODIFIER_FACTOR: f32 = 4.0;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct FirstPersonLookVectorController {
    pitch: f32,
    yaw: f32,
    mouse_look_sensitivity: f32,
    joystick_look_sensitivity: f32,
}

impl Default for FirstPersonLookVectorController {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            mouse_look_sensitivity: 1.0 / 100.0,
            joystick_look_sensitivity: 1.0 / 64.0,
        }
    }
}

impl LookVectorController for FirstPersonLookVectorController {
    fn update(
        &mut self,
        look_vector: &mut LookVector,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: Option<&MouseState>,
        game_controller_state: &GameControllerState,
        movement_speed: f32,
    ) {
        if let Some(mouse_state) = mouse_state {
            // Apply camera movement based on mouse input.

            self.apply_mouse_input(look_vector, mouse_state);
        }

        let camera_movement_step = movement_speed
            * if timing_info.seconds_since_last_update == 0.0 {
                0.01
            } else {
                timing_info.seconds_since_last_update
            };

        // Apply camera movement based on keyboard input.

        self.apply_keyboard_input(look_vector, keyboard_state, camera_movement_step);

        // Apply camera movement based on gamepad input.

        self.apply_game_controller_input(look_vector, game_controller_state, camera_movement_step);
    }
}

impl FirstPersonLookVectorController {
    fn apply_pitch_and_yaw_deltas(
        &mut self,
        look_vector: &mut LookVector,
        pitch_delta: f32,
        yaw_delta: f32,
    ) {
        self.yaw += yaw_delta;

        self.pitch += pitch_delta;

        // Update the look vector's target position, based on the new pitch and yaw.

        let rotation =
            Quaternion::new(vec3::RIGHT, -self.pitch) * Quaternion::new(vec3::UP, -self.yaw);

        let target = vec3::FORWARD * *rotation.mat();

        look_vector.set_target(look_vector.get_position() + target);
    }

    fn apply_mouse_input(&mut self, look_vector: &mut LookVector, mouse_state: &MouseState) {
        // Apply camera movement based on mouse input.

        let yaw_delta = mouse_state.relative_motion.0 as f32 * self.mouse_look_sensitivity;

        let pitch_delta = mouse_state.relative_motion.1 as f32 * self.mouse_look_sensitivity;

        self.apply_pitch_and_yaw_deltas(look_vector, pitch_delta, yaw_delta);
    }

    fn apply_keyboard_input(
        &mut self,
        look_vector: &mut LookVector,
        keyboard_state: &KeyboardState,
        camera_movement_step: f32,
    ) {
        // Apply camera movement based on keyboard input.

        let is_left_shift_pressed = keyboard_state.modifiers.contains(Mod::LSHIFTMOD);

        let movement_step = camera_movement_step
            * if is_left_shift_pressed {
                MOVEMENT_MODIFIER_FACTOR
            } else {
                1.0
            };

        for keycode in keyboard_state.pressed_keycodes.iter() {
            match *keycode {
                Keycode::Up | Keycode::W => {
                    look_vector.set_position(
                        look_vector.position + look_vector.get_forward() * movement_step,
                    );
                }
                Keycode::Down | Keycode::S => {
                    look_vector.set_position(
                        look_vector.position - look_vector.get_forward() * movement_step,
                    );
                }
                Keycode::Left | Keycode::A => {
                    look_vector.set_position(
                        look_vector.position - look_vector.get_right() * movement_step,
                    );
                }
                Keycode::Right | Keycode::D => {
                    look_vector.set_position(
                        look_vector.position + look_vector.get_right() * movement_step,
                    );
                }
                Keycode::Q => {
                    look_vector
                        .set_position(look_vector.position - look_vector.get_up() * movement_step);
                }
                Keycode::E => {
                    look_vector
                        .set_position(look_vector.position + look_vector.get_up() * movement_step);
                }
                _ => {}
            }
        }
    }

    fn apply_game_controller_input(
        &mut self,
        look_vector: &mut LookVector,
        game_controller_state: &GameControllerState,
        camera_movement_step: f32,
    ) {
        // D-pad inputs.

        if game_controller_state.buttons.dpad_up {
            look_vector.set_position(
                look_vector.position + look_vector.get_forward() * camera_movement_step,
            );
        } else if game_controller_state.buttons.dpad_down {
            look_vector.set_position(
                look_vector.position - look_vector.get_forward() * camera_movement_step,
            );
        } else if game_controller_state.buttons.dpad_left {
            look_vector.set_position(
                look_vector.position - look_vector.get_right() * camera_movement_step,
            );
        } else if game_controller_state.buttons.dpad_right {
            look_vector.set_position(
                look_vector.position + look_vector.get_right() * camera_movement_step,
            );
        }

        // Bumpers.

        if game_controller_state.buttons.left_shoulder {
            look_vector
                .set_position(look_vector.position + -look_vector.get_up() * camera_movement_step);
        }

        if game_controller_state.buttons.right_shoulder {
            look_vector
                .set_position(look_vector.position + look_vector.get_up() * camera_movement_step);
        }

        // Triggers.

        let mut movement_step = camera_movement_step;

        if game_controller_state.triggers.left.activation > 0 {
            let activation = game_controller_state.triggers.left.activation;

            if activation > 0 {
                let activation_alpha = activation as f32 / (i16::MAX as f32);

                movement_step *= MOVEMENT_MODIFIER_FACTOR * activation_alpha;
            }
        }

        // Left joystick.

        let left_joystick_position = &game_controller_state.joysticks.left.position;

        let left_joystick_position_normalized = Vec2 {
            x: left_joystick_position.x as f32 / i16::MAX as f32,
            y: left_joystick_position.y as f32 / i16::MAX as f32,
            z: 0.0,
        };

        if left_joystick_position_normalized.x > 0.5 {
            look_vector
                .set_position(look_vector.position + look_vector.get_right() * movement_step);
        } else if left_joystick_position_normalized.x < -0.5 {
            look_vector
                .set_position(look_vector.position - look_vector.get_right() * movement_step);
        }

        if left_joystick_position_normalized.y > 0.5 {
            look_vector
                .set_position(look_vector.position - look_vector.get_forward() * movement_step);
        } else if left_joystick_position_normalized.y < -0.5 {
            look_vector
                .set_position(look_vector.position + look_vector.get_forward() * movement_step);
        }

        // Right joystick.

        let right_joystick_position = &game_controller_state.joysticks.right.position;

        let right_joystick_position_normalized = Vec2 {
            x: right_joystick_position.x as f32 / i16::MAX as f32,
            y: right_joystick_position.y as f32 / i16::MAX as f32,
            z: 0.0,
        };

        let yaw_delta =
            right_joystick_position_normalized.x * (PI * self.joystick_look_sensitivity);

        let pitch_delta =
            right_joystick_position_normalized.y * (PI * self.joystick_look_sensitivity);

        self.apply_pitch_and_yaw_deltas(look_vector, pitch_delta, yaw_delta);
    }
}
