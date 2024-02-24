use crate::{matrix::Mat4, vec::vec3::Vec3};

pub mod look_vector;

#[derive(Debug, Copy, Clone)]
pub struct Transform3D {
    translation: Vec3,
    rotation: Vec3,
    scale: Vec3,
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

    pub fn rotation(&self) -> &Vec3 {
        &self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
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

        let rotation_mat = Mat4::rotation_x(self.rotation.x)
            * Mat4::rotation_y(self.rotation.y)
            * Mat4::rotation_z(self.rotation.z);

        let scale_mat = Mat4::scale([self.scale.x, self.scale.y, self.scale.z, 1.0]);

        self.mat = rotation_mat * scale_mat * translation_mat;
    }
}
