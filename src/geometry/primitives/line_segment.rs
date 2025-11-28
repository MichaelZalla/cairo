use crate::{
    geometry::{
        accelerator::static_triangle_bvh::StaticBVHCollision,
        intersect::intersect_line_segment_triangle,
        primitives::aabb::{AABB, Bounded},
    },
    vec::vec3::Vec3,
};

use super::{MedialPrimitive, triangle::Triangle};

#[derive(Debug, Copy, Clone)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub transformed_length: f32,
    pub collision: Option<StaticBVHCollision>,
}

impl LineSegment {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            start,
            end,
            transformed_length: (end - start).mag(),
            collision: None,
        }
    }

    pub fn lerped(&self) -> Vec3 {
        let t = self.collision.map(|c| c.t).unwrap_or(f32::MAX);

        self.start + (self.end - self.start) * t
    }

    /// Updates the collision t-value if the new value is closer.
    /// Returns true if the collision was updated (i.e., t is closer than current).
    pub fn update_collision_t(&mut self, t: f32) -> bool {
        let current_t = self.collision.map(|c| c.t).unwrap_or(f32::MAX);

        if t < current_t {
            // Update or create collision with new t value
            if let Some(ref mut collision) = self.collision {
                collision.t = t;
            } else {
                self.collision = Some(StaticBVHCollision {
                    t,
                    ..Default::default()
                });
            }
            true
        } else {
            false
        }
    }
}

impl Bounded for LineSegment {
    fn aabb(&self) -> AABB {
        let mut aabb = AABB::default();

        // Grow by segment endpoints
        aabb.grow(&self.start);
        aabb.grow(&self.end);

        aabb.recompute_derived_state();

        aabb
    }
}

impl MedialPrimitive for LineSegment {
    fn medial_segment(&self) -> &LineSegment {
        self
    }

    fn medial_segment_mut(&mut self) -> &mut LineSegment {
        self
    }

    fn intersect_triangle(&mut self, triangle: &Triangle) -> Option<Vec3> {
        intersect_line_segment_triangle(self, triangle)
    }
}

pub fn get_closest_points_between_segments(
    p1: Vec3,
    p2: Vec3,
    q1: Vec3,
    q2: Vec3,
) -> Option<((Vec3, f32), (Vec3, f32))> {
    // A vector running the length of edge A.
    let a = p2 - p1;

    // A vector running the length of edge B.
    let b = q2 - q1;

    // The direction of the vector between closest points.
    let normal = a.cross(b).as_normal();

    // Edges A and B can be expressed parametrically, with parameters `s` and `t`:
    //
    //   A(s) = p_1 + (p_2 - p_1) * s
    //   B(t) = q_1 + (q_2 - q_1) * t
    //
    // We want to compute an `s` and a `t` such that, when plugged in to the
    // parametric expressions above, A(s) and B(t) are the closest points
    // between the edges.

    let r = q1 - p1;

    let a_norm_cross_n = a.as_normal().cross(normal);
    let b_norm_cross_n = b.as_normal().cross(normal);

    let s = r.dot(b_norm_cross_n) / a.dot(b_norm_cross_n);
    let t = -r.dot(a_norm_cross_n) / b.dot(a_norm_cross_n);

    // If s < 0 or s > 1, then the closest point on the line forming A falls
    // outside of A.

    if !(0.0..=1.0).contains(&s) {
        return None;
    }

    // If t < 0 or s > 1, then the closest point on the line forming B falls
    // outside of B.

    if !(0.0..=1.0).contains(&t) {
        return None;
    }

    let p_a = p1 + (p2 - p1) * s;
    let q_a = q1 + (q2 - q1) * t;

    Some(((p_a, s), (q_a, t)))
}
