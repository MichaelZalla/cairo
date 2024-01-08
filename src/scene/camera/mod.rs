use std::f32::consts::PI;

use crate::{
    matrix::Mat4,
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    position: Vec3,
    forward: Vec3,
    up: Vec3,
    right: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        let mut camera = Camera {
            position,
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

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_target_position(&mut self, target: Vec3) {
        let world_up = vec3::UP;

        self.forward = (target - self.position).as_normal();

        self.right = world_up.cross(self.forward).as_normal();

        self.up = self.forward.cross(self.right).as_normal();
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
        self.pitch = pitch.max(PI / 2.0).min(3.0 * PI / 2.0);
    }

    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn get_roll(&self) -> f32 {
        self.roll
    }

    pub fn set_roll(&mut self, roll: f32) {
        self.roll = roll;
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
}
