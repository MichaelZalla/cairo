use crate::vec::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
}

impl LineSegment {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    pub fn mag(&self) -> f32 {
        (self.end - self.start).mag()
    }
}
