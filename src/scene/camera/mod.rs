use crate::{matrix::Mat4, vec::vec4::Vec4};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Vec4,
    pub position_inverse: Vec4,
    pub rotation_inverse_transform: Mat4,
    pub rotation_inverse_transposed: Mat4,
    pub movement_speed: f32,
    pub roll: f32,
    pub roll_speed: f32,
}

impl Camera {
    pub fn new(
        position: Vec4,
        rotation_inverse_transform: Mat4,
        movement_speed: f32,
        roll: f32,
        roll_speed: f32,
    ) -> Self {
        return Camera {
            position,
            position_inverse: position * -1.0,
            rotation_inverse_transform,
            rotation_inverse_transposed: rotation_inverse_transform.transposed(),
            movement_speed,
            roll,
            roll_speed,
        };
    }
}
