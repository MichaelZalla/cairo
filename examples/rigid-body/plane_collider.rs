use cairo::{
    geometry::primitives::plane::Plane,
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Clone)]
pub struct PlaneCollider {
    pub point: Vec3,
    pub plane: Plane,
}

impl Default for PlaneCollider {
    fn default() -> Self {
        let point = Vec3::default();

        Self {
            point,
            plane: Plane::new(point, vec3::UP),
        }
    }
}

impl PlaneCollider {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let plane = Plane::new(point, direction);

        Self { point, plane }
    }
}
