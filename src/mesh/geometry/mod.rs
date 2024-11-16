use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    physics::collision::aabb::AABB,
    vec::{vec2::Vec2, vec3::Vec3},
};

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

impl Geometry {
    pub fn center(&mut self) {
        let aabb = AABB::from_geometry(self);

        let center = aabb.center;

        for vertex in self.vertices.iter_mut() {
            vertex.x -= center.x;
            vertex.y -= center.y;
            vertex.z -= center.z;
        }
    }
}
