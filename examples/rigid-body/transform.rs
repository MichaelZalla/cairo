use cairo::{matrix::Mat4, transform::quaternion::Quaternion, vec::vec3::Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    translation: Vec3,
    orientation: Quaternion,
    scale: Vec3,
    mat: Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            orientation: Default::default(),
            scale: Vec3::ones(),
            mat: Mat4::identity(),
        }
    }
}

impl Transform {
    pub fn new(position: Vec3) -> Self {
        Self {
            translation: position,
            ..Default::default()
        }
    }

    pub fn translation(&self) -> &Vec3 {
        &self.translation
    }

    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;

        self.recompute_derived_state();
    }

    pub fn orientation(&self) -> &Quaternion {
        &self.orientation
    }

    pub fn set_orientation(&mut self, orientation: Quaternion) {
        self.orientation = orientation;

        self.recompute_derived_state();
    }

    pub fn set_translation_and_orientation(&mut self, translation: Vec3, orientation: Quaternion) {
        self.translation = translation;
        self.orientation = orientation;

        self.recompute_derived_state();
    }

    pub fn scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;

        self.recompute_derived_state();
    }

    pub fn mat(&self) -> &Mat4 {
        &self.mat
    }

    fn recompute_derived_state(&mut self) {
        self.orientation.renormalize();

        let rotation = *self.orientation.mat();
        let translation = Mat4::translation(self.translation);
        let scale = Mat4::scale([self.scale.x, self.scale.y, self.scale.z, 1.0]);

        self.mat = rotation * scale * translation;
    }
}
