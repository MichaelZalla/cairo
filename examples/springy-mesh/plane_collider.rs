use cairo::{geometry::primitives::plane::Plane, vec::vec3::Vec3};

#[derive(Default, Debug, Clone)]
pub struct PlaneCollider {
    pub point: Vec3,
    pub plane: Plane,
}

impl PlaneCollider {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let plane = Plane::new(point, direction);

        Self { point, plane }
    }
}
