use crate::{
    geometry::CollisionKind,
    vec::vec3::{Vec3, Vec3A},
};

use super::plane::Plane;

/// Result of intersecting a primitive with a triangle.
#[derive(Debug, Copy, Clone)]
pub struct TriangleIntersection {
    /// Parametric distance along the medial segment [0, 1].
    pub t: f32,
    /// Barycentric coordinates of the intersection point on the triangle.
    pub barycentric: Vec3,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub centroid: Vec3A,
    pub plane: Plane,
    pub edge_plane_bc: Plane,
    pub edge_plane_ca: Plane,
    pub collision_point: Option<Vec3>,
}

/// Tests if a point projects onto a triangle using barycentric coordinates.
///
/// Returns the barycentric coordinates (u, v, w) if the point projects within the
/// triangle's boundaries, or None if it projects outside.
pub fn point_projects_onto_triangle(point: Vec3, triangle: &Triangle) -> Option<Vec3> {
    // Generous epsilon tolerance for floating point errors near triangle boundaries
    const EPSILON: f32 = 0.1;

    // Compute the barycentric coordinate U; exit if outside the [0..1] range.
    let u = point.dot(triangle.edge_plane_bc.normal) - triangle.edge_plane_bc.d;

    if !(-EPSILON..=1.0 + EPSILON).contains(&u) {
        return None;
    }

    // Compute the barycentric coordinate V; exit if negative.
    let v = point.dot(triangle.edge_plane_ca.normal) - triangle.edge_plane_ca.d;

    if v < -EPSILON {
        return None;
    }

    // Compute the barycentric coordinate W; exit if negative.
    let w = 1.0 - u - v;

    if w < -EPSILON {
        return None;
    }

    Some(Vec3 { x: u, y: v, z: w })
}

/// Classifies an intersection point on a triangle based on barycentric coordinates.
pub fn classify_intersection(barycentric: Vec3) -> CollisionKind {
    const EPSILON: f32 = 0.001;

    let u = barycentric.x;
    let v = barycentric.y;
    let w = barycentric.z;

    // Check if we're at a vertex (one coordinate is ~1.0)
    if u >= 1.0 - EPSILON {
        return CollisionKind::Vertex; // Vertex A
    }

    if v >= 1.0 - EPSILON {
        return CollisionKind::Vertex; // Vertex B
    }

    if w >= 1.0 - EPSILON {
        return CollisionKind::Vertex; // Vertex C
    }

    // Check if we're on an edge (one coordinate is ~0.0)
    if u <= EPSILON {
        return CollisionKind::Edge; // Edge BC (opposite vertex A)
    }

    if v <= EPSILON {
        return CollisionKind::Edge; // Edge CA (opposite vertex B)
    }

    if w <= EPSILON {
        return CollisionKind::Edge; // Edge AB (opposite vertex C)
    }

    // Otherwise, we're on the face interior
    CollisionKind::Face
}

impl Triangle {
    pub fn new(vertices: [usize; 3], a: &Vec3, b: &Vec3, c: &Vec3) -> Self {
        let mut result = Self {
            vertices,
            ..Default::default()
        };

        result.update_vertex_positions(a, b, c);

        result
    }

    pub fn update_vertex_positions(&mut self, a: &Vec3, b: &Vec3, c: &Vec3) {
        let centroid = (a + b + c) * 0.33333;

        let normal = (b - a).cross(c - a).as_normal();

        let plane = Plane::new(*a, normal);

        let mut edge_plane_bc = Plane::new(*b, normal.cross(c - b));

        edge_plane_bc *= 1.0 / (a.dot(edge_plane_bc.normal) - edge_plane_bc.d);

        let mut edge_plane_ca = Plane::new(*c, normal.cross(a - c));

        edge_plane_ca *= 1.0 / (b.dot(edge_plane_ca.normal) - edge_plane_ca.d);

        self.centroid = Vec3A::from(centroid);

        self.plane = plane;

        self.edge_plane_bc = edge_plane_bc;
        self.edge_plane_ca = edge_plane_ca;
    }
}
