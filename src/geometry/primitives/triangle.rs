use crate::vec::vec3::{Vec3, Vec3A};

use super::plane::Plane;

#[derive(Default, Debug, Copy, Clone)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub centroid: Vec3A,
    pub plane: Plane,
    pub edge_plane_bc: Plane,
    pub edge_plane_ca: Plane,
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

        self.centroid = Vec3A::from_vec3(centroid);

        self.plane = plane;

        self.edge_plane_bc = edge_plane_bc;
        self.edge_plane_ca = edge_plane_ca;
    }
}
