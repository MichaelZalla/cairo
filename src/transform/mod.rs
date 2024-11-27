use serde::{Deserialize, Serialize};

use crate::{matrix::Mat4, vec::vec3::Vec3};

use quaternion::Quaternion;

pub mod look_vector;
pub mod quaternion;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Transform3D {
    translation: Vec3,
    rotation: Quaternion,
    scale: Vec3,
    #[serde(skip)]
    mat: Mat4,
    #[serde(skip)]
    inverse_mat: Mat4,
}

impl Default for Transform3D {
    fn default() -> Self {
        let mut t = Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ones(),
            mat: Default::default(),
            inverse_mat: Default::default(),
        };

        t.recompute_transforms();

        t
    }
}

impl Transform3D {
    pub fn translation(&self) -> &Vec3 {
        &self.translation
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;

        self.recompute_transforms();
    }

    pub fn rotation(&self) -> &Quaternion {
        &self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.rotation = rotation;

        self.recompute_transforms();
    }

    pub fn scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;

        self.recompute_transforms();
    }

    pub fn mat(&self) -> &Mat4 {
        &self.mat
    }

    pub fn inverse_mat(&self) -> &Mat4 {
        &self.inverse_mat
    }

    fn recompute_transforms(&mut self) {
        let (translation, scale, rotation) = (self.translation, self.scale, self.rotation);

        self.mat = {
            let translation_mat = Mat4::translation(translation);

            let scale_mat = Mat4::scale(scale);

            let rotation_mat = *rotation.mat();

            scale_mat * rotation_mat * translation_mat
        };

        self.inverse_mat = {
            let inverse_translation_mat = Mat4::translation(-translation);

            let inverse_scale_mat = Mat4::scale(scale.reciprocal());

            let inverse_rotation = rotation.inverse();

            let inverse_rotation_mat = *inverse_rotation.mat();

            inverse_translation_mat * inverse_rotation_mat * inverse_scale_mat
        };
    }
}
