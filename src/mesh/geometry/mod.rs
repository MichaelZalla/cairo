use std::fmt;

use serde::{Deserialize, Serialize};

use crate::vec::{vec2::Vec2, vec3::Vec3};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Face {
    pub vertices: (usize, usize, usize), // Indices into Vec<Vec3>
    pub normals: Option<(usize, usize, usize)>, // Indices into Vec<Vec3>
    pub uvs: Option<(usize, usize, usize)>, // Indices into Vec<Vec2>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub faces: Vec<Face>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Geometry",)?;
        writeln!(v, "  > Vertices: {}", self.vertices.len())?;
        writeln!(v, "  > UVs: {}", self.uvs.len())?;
        writeln!(v, "  > Normals: {}", self.normals.len())?;
        writeln!(v, "  > Faces: {}", self.faces.len())
    }
}

impl Geometry {
    pub fn new(vertices: Vec<Vec3>, uvs: Vec<Vec2>, normals: Vec<Vec3>, faces: Vec<Face>) -> Self {
        let mut geo = Geometry::default();

        geo.vertices = vertices;
        geo.normals = normals;
        geo.uvs = uvs;
        geo.faces = faces;

        geo
    }
}
