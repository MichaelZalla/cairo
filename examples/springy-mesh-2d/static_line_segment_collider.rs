use cairo::{
    geometry::{
        intersect::intersect_line_segment_plane,
        primitives::{line_segment::LineSegment, plane::Plane},
    },
    vec::vec3::{self, Vec3},
};

pub(crate) struct StaticLineSegmentCollider {
    pub segment: LineSegment,
    pub plane: Plane,
    tangent: Vec3,
    length: f32,
}

impl StaticLineSegmentCollider {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        let segment = LineSegment::new(start, end);

        let midpoint = start + (end - start) / 2.0;

        let delta = end - start;
        let length = delta.mag();
        let tangent = delta.as_normal();
        let normal = vec3::FORWARD.cross(tangent).as_normal();

        let plane = Plane::new(midpoint, normal);

        Self {
            segment,
            plane,
            tangent,
            length,
        }
    }
}

impl StaticLineSegmentCollider {
    pub fn test(&self, start: &Vec3, end: &Vec3) -> Option<(f32, Vec3)> {
        let projection = (end - self.segment.start).dot(self.tangent);

        if projection < 0.0 || projection > self.length {
            return None;
        }

        // @TODO Check whether the position at time `t + f * h` still projects
        // onto the segment.

        intersect_line_segment_plane(&self.plane, *start, *end)
    }
}
