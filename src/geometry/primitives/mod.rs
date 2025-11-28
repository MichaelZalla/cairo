pub mod aabb;
pub mod capsule;
pub mod line_segment;
pub mod plane;
pub mod ray;
pub mod sphere;
pub mod triangle;

pub use triangle::TriangleIntersection;

use line_segment::LineSegment;
use triangle::Triangle;

/// Trait for primitives that have a medial axis (line segment) and can be intersected with triangles.
/// This allows generic BVH traversal code to work with both line segments and swept volumes (capsules, etc.).
pub trait MedialPrimitive: aabb::Bounded + Copy {
    /// Returns a reference to the medial line segment.
    fn medial_segment(&self) -> &LineSegment;

    /// Returns a mutable reference to the medial line segment.
    fn medial_segment_mut(&mut self) -> &mut LineSegment;

    /// Intersects this primitive with a triangle.
    fn intersect_triangle(&mut self, triangle: &Triangle) -> Option<TriangleIntersection>;
}
