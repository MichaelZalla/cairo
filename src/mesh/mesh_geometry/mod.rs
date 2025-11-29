use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    geometry::primitives::{aabb::AABB, triangle::Triangle},
    vec::{vec2::Vec2, vec3::Vec3},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MeshGeometry {
    pub vertices: Box<[Vec3]>,
    pub normals: Box<[Vec3]>,
    pub uvs: Box<[Vec2]>,
}

impl fmt::Display for MeshGeometry {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "MeshGeometry",)?;
        writeln!(v, "  > Vertices: {}", self.vertices.len())?;
        writeln!(v, "  > UVs: {}", self.uvs.len())?;
        writeln!(v, "  > Normals: {}", self.normals.len())
    }
}

impl MeshGeometry {
    pub fn get_vertices(&self, v0: usize, v1: usize, v2: usize) -> (&Vec3, &Vec3, &Vec3) {
        (&self.vertices[v0], &self.vertices[v1], &self.vertices[v2])
    }

    /// Gets the three vertex positions for a triangle.
    pub fn get_triangle_vertices(&self, triangle: &Triangle) -> (Vec3, Vec3, Vec3) {
        let [idx_a, idx_b, idx_c] = triangle.vertices;

        (
            self.vertices[idx_a],
            self.vertices[idx_b],
            self.vertices[idx_c],
        )
    }

    pub fn center(&mut self) {
        let aabb = AABB::from(&*self);

        let center = aabb.center();

        for vertex in self.vertices.iter_mut() {
            vertex.x -= center.x;
            vertex.y -= center.y;
            vertex.z -= center.z;
        }
    }
}
