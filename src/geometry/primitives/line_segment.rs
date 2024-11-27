use crate::vec::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub t: f32,
    pub colliding_primitive: Option<usize>,
}

impl LineSegment {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            start,
            end,
            t: f32::MAX,
            colliding_primitive: None,
        }
    }

    pub fn mag(&self) -> f32 {
        (self.end - self.start).mag()
    }
}
