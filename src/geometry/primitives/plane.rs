use serde::{Deserialize, Serialize};

use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Plane {
    // Constant-normal form.
    pub normal: Vec3, // Plane normal.
    pub d: f32,       // d = dot(n, P) for any point P on the plane.
}

impl Plane {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let normal = direction.as_normal();

        let d = normal.dot(point);

        Self { normal, d }
    }

    pub fn is_on_or_in_front_of(&self, position: &Vec3, radius: f32) -> bool {
        self.get_signed_distance(position) > -radius
    }

    pub fn get_signed_distance(&self, position: &Vec3) -> f32 {
        self.normal.dot(*position) - self.d
    }
}
