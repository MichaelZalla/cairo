use crate::vec::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub transformed_length: f32,
    pub t: f32,
    pub colliding_bvh_index: Option<usize>,
    pub colliding_primitive: Option<usize>,
}

impl LineSegment {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            start,
            end,
            transformed_length: (end - start).mag(),
            t: f32::MAX,
            colliding_bvh_index: None,
            colliding_primitive: None,
        }
    }
}
