use crate::{
    geometry::{intersect::intersect_capsule_triangle, primitives::aabb::Bounded},
    vec::vec3::{self, Vec3},
};

use super::{MedialPrimitive, TriangleIntersection, aabb::AABB, line_segment::LineSegment, triangle::Triangle};

#[derive(Debug, Copy, Clone)]
pub struct Capsule {
    pub segment: LineSegment,
    pub radius: f32,
}

impl Capsule {
    pub fn new(start: Vec3, end: Vec3, radius: f32) -> Self {
        Self {
            segment: LineSegment::new(start, end),
            radius,
        }
    }

    pub fn lerped(&self) -> Vec3 {
        self.segment.lerped()
    }
}

impl From<LineSegment> for Capsule {
    fn from(segment: LineSegment) -> Self {
        Self {
            segment,
            radius: 0.0,
        }
    }
}

impl Bounded for Capsule {
    fn aabb(&self) -> AABB {
        let mut aabb = self.segment.aabb();

        // Expand by radius in all directions
        aabb.min -= vec3::ONES * self.radius;
        aabb.max += vec3::ONES * self.radius;

        aabb.recompute_derived_state();

        aabb
    }
}

impl MedialPrimitive for Capsule {
    fn medial_segment(&self) -> &LineSegment {
        &self.segment
    }

    fn medial_segment_mut(&mut self) -> &mut LineSegment {
        &mut self.segment
    }

    fn intersect_triangle(&mut self, triangle: &Triangle) -> Option<TriangleIntersection> {
        intersect_capsule_triangle(self, triangle)
    }
}
