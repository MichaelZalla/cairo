use std::fmt;

use serde::{Deserialize, Serialize};

use crate::vec::{vec2::Vec2, vec3::Vec3};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Geometry",)?;
        writeln!(v, "  > Vertices: {}", self.vertices.len())?;
        writeln!(v, "  > UVs: {}", self.uvs.len())?;
        writeln!(v, "  > Normals: {}", self.normals.len())
    }
}

impl Geometry {
    pub fn new(vertices: Vec<Vec3>, uvs: Vec<Vec2>, normals: Vec<Vec3>) -> Self {
        let mut geo = Geometry::default();

        geo.vertices = vertices;
        geo.normals = normals;
        geo.uvs = uvs;

        geo
    }
}
