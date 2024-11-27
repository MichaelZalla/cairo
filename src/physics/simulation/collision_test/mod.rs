use crate::geometry::primitives::{line_segment::LineSegment, plane::Plane};

pub fn test_line_segment_plane(segment: &LineSegment, plane: &Plane) -> Option<(f32, f32)> {
    let approach_distance = plane.get_signed_distance(&segment.start);
    let penetration_distance = plane.get_signed_distance(&segment.end);

    if (approach_distance * penetration_distance) < 0.0 && penetration_distance < 0.0 {
        // Calculates the fraction of the timestep at which the collision
        // occurred; note that this calculation assumes constant acceleration of
        // the plane and the segment being tested.

        let penetration_depth = -penetration_distance;

        let total_distance = approach_distance + penetration_depth;

        let f = approach_distance / total_distance;

        Some((f, penetration_depth))
    } else {
        None
    }
}
