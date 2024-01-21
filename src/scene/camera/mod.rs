use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    time::TimingInfo,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub movement_speed: f32,
    field_of_view: f32,
    aspect_ratio: f32,
    projection_z_near: f32,
    projection_z_far: f32,
    projection_transform: Mat4,
    projection_inverse_transform: Mat4,
    position: Vec3,
    target: Vec3,
    forward: Vec3,
    up: Vec3,
    right: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32, position: Vec3, target: Vec3) -> Self {
        let field_of_view = 75.0;

        let projection_z_near = 0.3;
        let projection_z_far = 1000.0;

        let projection_transform = Mat4::projection_for_fov(
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
        );

        let projection_inverse_transform = Mat4::projection_inverse_for_fov(
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
        );

        let mut camera = Camera {
            field_of_view,
            aspect_ratio,
            projection_z_near,
            projection_z_far,
            movement_speed: 50.0,
            projection_transform,
            projection_inverse_transform,
            position,
            target: vec3::FORWARD,
            forward: vec3::FORWARD,
            up: vec3::UP,
            right: vec3::LEFT * -1.0,
            pitch: 0.0,
            yaw: PI / 2.0,
            roll: 0.0,
        };

        camera.set_target_position(target);

        return camera;
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn get_projection_z_near(&self) -> f32 {
        self.projection_z_near
    }

    pub fn get_projection_z_far(&self) -> f32 {
        self.projection_z_far
    }

    pub fn set_projection_z_far(&mut self, far: f32) {
        self.projection_z_far = far;

        self.projection_transform = Mat4::projection_for_fov(
            self.field_of_view,
            self.aspect_ratio,
            self.projection_z_near,
            self.projection_z_far,
        );

        self.projection_inverse_transform = Mat4::projection_inverse_for_fov(
            self.field_of_view,
            self.aspect_ratio,
            self.projection_z_near,
            self.projection_z_far,
        );
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

    pub fn get_lookat_matrix(&self) -> Mat4 {
        let (p, f, r, u) = (self.position, self.forward, self.right, self.up);

        let rotation_transposed = Mat4::new_from_elements([
            // Row-major ordering
            [r.x, u.x, f.x, 0.0],
            [r.y, u.y, f.y, 0.0],
            [r.z, u.z, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let translation_negated = Mat4::new_from_elements([
            // Row-major ordering
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-p.x, -p.y, -p.z, 1.0],
        ]);

        translation_negated * rotation_transposed
    }

    pub fn get_view_inverse_transform(&self) -> Mat4 {
        self.get_lookat_matrix()
    }

    pub fn get_view_rotation_transform(&self) -> Mat4 {
        let (f, r, u) = (self.forward, self.right, self.up);

        Mat4::new_from_elements([
            // Row-major ordering
            [r.x, r.y, r.z, 0.0],
            [u.x, u.y, u.z, 0.0],
            [f.x, f.y, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection_transform
    }

    pub fn get_projection_inverse(&self) -> Mat4 {
        self.projection_inverse_transform
    }

    pub fn get_pixel_world_space_position(
        &self,
        screen_x: u32,
        screen_y: u32,
        width: u32,
        height: u32,
    ) -> Vec4 {
        let pixel_coordinate_ndc_space = Vec4 {
            x: screen_x as f32 / width as f32,
            y: screen_y as f32 / height as f32,
            z: -1.0,
            w: 1.0,
        };

        // Transform our screen-space coordinate by the camera's inverse projection.

        let pixel_coordinate_projection_space =
            pixel_coordinate_ndc_space * self.get_projection_inverse();

        // Camera-direction vector in camera-view-space: (0, 0, -1)

        // Compute pixel coordinate in camera-view-space.

        // Near-plane coordinates in camera-view-space:
        //
        //  x: -1 to 1
        //  y: -1 to 1 (y is up)
        //  z: -1 (near) to 1 (far)

        let pixel_coordinate_camera_view_space: Vec4 = Vec4 {
            x: -1.0 + pixel_coordinate_projection_space.x * 2.0,
            y: -1.0 + (1.0 - pixel_coordinate_projection_space.y) * 2.0,
            z: 1.0,
            w: 1.0, // ???????
        };

        // Transform camera-view-space coordinate to world-space coordinate.

        // Note: Treating the camera's position as the world-space origin.

        let pixel_coordinate_world_space =
            pixel_coordinate_camera_view_space * self.get_view_rotation_transform();

        pixel_coordinate_world_space
    }

    pub fn update(
        &mut self,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        // Apply camera movement based on mouse input.

        // Translate relative mouse movements to NDC values (in the range [0, 1]).

        let mouse_x_delta = mouse_state.relative_motion.0 as f32 / 400.0;
        let mouse_y_delta = mouse_state.relative_motion.1 as f32 / 400.0;

        // Update camera pitch and yaw, based on mouse position deltas.

        self.set_pitch(self.pitch - mouse_y_delta * 2.0 * PI);
        self.set_yaw(self.yaw - mouse_x_delta * 2.0 * PI);

        // Apply field-of-view zoom based on mousewheel input.

        if mouse_state.wheel_did_move {
            self.field_of_view -= mouse_state.wheel_y as f32;

            self.field_of_view = self.field_of_view.max(1.0).min(120.0);

            self.projection_transform = Mat4::projection_for_fov(
                self.field_of_view,
                self.aspect_ratio,
                self.projection_z_near,
                self.projection_z_far,
            );

            self.projection_inverse_transform = Mat4::projection_inverse_for_fov(
                self.field_of_view,
                self.aspect_ratio,
                self.projection_z_near,
                self.projection_z_far,
            );
        }

        // Apply camera movement based on keyboard input.

        let camera_movement_step = self.movement_speed * timing_info.seconds_since_last_update;

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
                    self.set_position(self.position - vec3::UP * camera_movement_step);
                }
                Keycode::E { .. } => {
                    self.set_position(self.position + vec3::UP * camera_movement_step);
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

        let yaw_delta = -right_joystick_position_normalized.x * PI / 32.0;
        let pitch_delta = -right_joystick_position_normalized.y * PI / 32.0;
        let _roll_delta = -yaw_delta * 0.5;

        self.set_pitch(self.pitch - pitch_delta * 2.0 * PI);
        self.set_yaw(self.yaw - yaw_delta * 2.0 * PI);
    }
}
