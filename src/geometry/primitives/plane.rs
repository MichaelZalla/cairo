use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn is_on_or_in_front_of(&self, position: &Vec3, radius: f32) -> bool {
        self.get_signed_distance(position) > -radius
    }

    pub fn get_signed_distance(&self, position: &Vec3) -> f32 {
        self.normal.dot(*position - self.point)
    }
}
