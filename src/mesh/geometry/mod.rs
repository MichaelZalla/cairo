use std::fmt;

use serde::{Deserialize, Serialize};

use crate::vec::{vec2::Vec2, vec3::Vec3};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    pub vertices: Box<[Vec3]>,
    pub normals: Box<[Vec3]>,
    pub uvs: Box<[Vec2]>,
}

impl fmt::Display for Geometry {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Geometry",)?;
        writeln!(v, "  > Vertices: {}", self.vertices.len())?;
        writeln!(v, "  > UVs: {}", self.uvs.len())?;
        writeln!(v, "  > Normals: {}", self.normals.len())
    }
}
