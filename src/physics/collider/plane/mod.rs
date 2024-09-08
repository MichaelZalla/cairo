use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn is_sphere_on_or_in_front_of(&self, sphere_position: &Vec3, sphere_radius: f32) -> bool {
        self.get_signed_distance_to_plane(sphere_position) > -sphere_radius
    }

    pub fn get_signed_distance_to_plane(&self, sphere_position: &Vec3) -> f32 {
        self.normal.dot(*sphere_position - self.point)
    }
}
