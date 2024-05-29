use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use sdl2::keyboard::Keycode;

use crate::{
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    serde::PostDeserialize,
    time::TimingInfo,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LookVector {
    position: Vec3,
    target: Vec3,
    forward: Vec3,
    up: Vec3,
    right: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
}

impl PostDeserialize for LookVector {
    fn post_deserialize(&mut self) {
        self.set_target_position(self.target);
    }
}

impl LookVector {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        let mut vector = Self {
            position,
            target,
            forward: vec3::FORWARD,
            up: vec3::UP,
            right: vec3::RIGHT,
            pitch: 0.0,
            yaw: PI / 2.0,
            roll: 0.0,
        };

        vector.post_deserialize();

        vector
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_target(&self) -> Vec3 {
        self.target
    }

    pub fn set_target_position(&mut self, target: Vec3) {
        let world_up = vec3::UP;

        self.forward = (target - self.position).as_normal();

        self.right = world_up.cross(self.forward).as_normal();

        self.up = self.forward.cross(self.right).as_normal();

        self.target = target;
    }

    pub fn get_direction(&self) -> Vec3 {
        Vec3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        }
    }

    fn look_in_direction(&mut self) {
        self.set_target_position(self.position + self.get_direction())
    }

    pub fn get_forward(&self) -> Vec3 {
        self.forward
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn get_right(&self) -> Vec3 {
        self.right
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.max(-PI / 2.0 * 0.999).min(PI / 2.0 * 0.999);

        self.look_in_direction();
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;

        self.look_in_direction();
    }

    pub fn get_roll(&self) -> f32 {
        self.roll
    }

    pub fn set_roll(&mut self, _roll: f32) {
        unimplemented!()
    }

    pub fn update(
        &mut self,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: Option<&MouseState>,
        game_controller_state: &GameControllerState,
        movement_speed: f32,
    ) {
        // Apply camera movement based on mouse input.

        if let Some(mouse_state) = mouse_state {
            // Translate relative mouse movements to NDC values (in the
            // range [0, 1]).

            let mouse_x_delta = mouse_state.relative_motion.0 as f32 / 400.0;
            let mouse_y_delta = mouse_state.relative_motion.1 as f32 / 400.0;

            // Update camera pitch and yaw, based on mouse position deltas.

            if mouse_x_delta != 0.0 {
                self.set_yaw(self.yaw - mouse_x_delta * 2.0 * PI);
            }

            if mouse_y_delta != 0.0 {
                self.set_pitch(self.pitch - mouse_y_delta * 2.0 * PI);
            }
        }

        // Apply camera movement based on keyboard input.

        let camera_movement_step = movement_speed * timing_info.seconds_since_last_update;

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Up | Keycode::W { .. } => {
                    self.set_position(self.position + self.get_forward() * camera_movement_step);
                }
                Keycode::Down | Keycode::S { .. } => {
                    self.set_position(self.position - self.get_forward() * camera_movement_step);
                }
                Keycode::Left | Keycode::A { .. } => {
                    self.set_position(self.position - self.get_right() * camera_movement_step);
                }
                Keycode::Right | Keycode::D { .. } => {
                    self.set_position(self.position + self.get_right() * camera_movement_step);
                }
                Keycode::Q { .. } => {
                    self.set_position(self.position - self.get_up() * camera_movement_step);
                }
                Keycode::E { .. } => {
                    self.set_position(self.position + self.get_up() * camera_movement_step);
                }
                _ => {}
            }
        }

        // Apply camera movement based on gamepad input.

        if game_controller_state.buttons.dpad_up {
            self.set_position(self.position + self.get_forward() * camera_movement_step);
        } else if game_controller_state.buttons.dpad_down {
            self.set_position(self.position - self.get_forward() * camera_movement_step);
        }

        let left_joystick_position_normalized = Vec2 {
            x: game_controller_state.joysticks.left.position.x as f32 / std::i16::MAX as f32,
            y: game_controller_state.joysticks.left.position.y as f32 / std::i16::MAX as f32,
            z: 1.0,
        };

        if left_joystick_position_normalized.x > 0.5 {
            self.set_position(self.position + self.get_right() * camera_movement_step);
        } else if left_joystick_position_normalized.x < -0.5 {
            self.set_position(self.position - self.get_right() * camera_movement_step);
        }

        if left_joystick_position_normalized.y > 0.5 {
            self.set_position(self.position - self.get_forward() * camera_movement_step);
        } else if left_joystick_position_normalized.y < -0.5 {
            self.set_position(self.position + self.get_forward() * camera_movement_step);
        }

        let right_joystick_position_normalized = Vec2 {
            x: game_controller_state.joysticks.right.position.x as f32 / std::i16::MAX as f32,
            y: game_controller_state.joysticks.right.position.y as f32 / std::i16::MAX as f32,
            z: 1.0,
        };

        let yaw_delta = right_joystick_position_normalized.x * (PI / 64.0);
        let pitch_delta = right_joystick_position_normalized.y * (PI / 64.0);
        let _roll_delta = -yaw_delta * 0.5;

        if pitch_delta != 0.0 {
            self.set_pitch(self.pitch - pitch_delta * 2.0 * PI);
        }

        if yaw_delta != 0.0 {
            self.set_yaw(self.yaw - yaw_delta * 2.0 * PI);
        }
    }
}
