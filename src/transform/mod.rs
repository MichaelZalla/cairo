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
}

impl Default for Transform3D {
    fn default() -> Self {
        let mut t = Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ones(),
            mat: Default::default(),
        };

        t.recompute_transform();

        t
    }
}

impl Transform3D {
    pub fn translation(&self) -> &Vec3 {
        &self.translation
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;

        self.recompute_transform();
    }

    pub fn rotation(&self) -> &Quaternion {
        &self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.rotation = rotation;

        self.recompute_transform();
    }

    pub fn scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;

        self.recompute_transform();
    }

    pub fn mat(&self) -> &Mat4 {
        &self.mat
    }

    fn recompute_transform(&mut self) {
        let translation_mat = Mat4::translation(self.translation);

        let scale_mat = Mat4::scale([self.scale.x, self.scale.y, self.scale.z, 1.0]);

        self.mat = *self.rotation.mat() * scale_mat * translation_mat;
    }
}
