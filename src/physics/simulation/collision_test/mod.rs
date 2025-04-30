use crate::geometry::primitives::{line_segment::LineSegment, plane::Plane};

pub fn test_line_segment_plane(segment: &LineSegment, plane: &Plane) -> Option<(f32, f32)> {
    let start_distance = plane.get_signed_distance(&segment.start);
    let end_distance = plane.get_signed_distance(&segment.end);

    if (start_distance * end_distance) < 0.0 && end_distance < 0.0 {
        // Sign change indicates an intersection.

        // Computes a t-value using start_distance and end_distance.

        let f = start_distance / (start_distance - end_distance);

        // Negative `end_distance` indicates the penetration depth.

        Some((f, -end_distance))
    } else {
        None
    }
}
