pub mod accelerator;
pub mod intersect;
pub mod primitives;

/// Classification of where a primitive intersects a triangle.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CollisionKind {
    Face,
    Edge,
    Vertex,
}
