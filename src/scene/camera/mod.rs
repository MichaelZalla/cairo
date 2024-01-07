use crate::{matrix::Mat4, vec::vec3::Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation_inverse_transform: Mat4,
    pub rotation_inverse_transposed: Mat4,
}

impl Camera {
    pub fn new(position: Vec3, rotation_inverse_transform: Mat4) -> Self {
        return Camera {
            position,
            rotation_inverse_transform,
            rotation_inverse_transposed: rotation_inverse_transform.transposed(),
        };
    }

    pub fn get_view_inverse_transform(&self) -> Mat4 {
        let position_inverse = self.position * -1.0;
        let translation_inverse_transform = Mat4::translation(Vec3 {
            x: position_inverse.x,
            y: position_inverse.y,
            z: position_inverse.z,
        });

        translation_inverse_transform * self.rotation_inverse_transform
    }
}
