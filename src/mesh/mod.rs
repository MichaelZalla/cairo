use std::fmt;

use super::vec::vec3::Vec3;
use crate::vec::vec2::Vec2;

pub mod obj;
pub mod primitive;

#[derive(Default, Debug, Copy, Clone)]
pub struct Face {
    pub vertices: (usize, usize, usize), // Indices into Vec<Vec3>
    pub normals: Option<(usize, usize, usize)>, // Indices into Vec<Vec3>
    pub uvs: Option<(usize, usize, usize)>, // Indices into Vec<Vec2>
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub object_source: Option<String>,
    pub object_name: Option<String>,
    pub group_name: Option<String>,
    pub material_source: Option<String>,
    pub material_name: Option<String>,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub faces: Vec<Face>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            object_source: None,
            object_name: None,
            group_name: None,
            material_source: Default::default(),
            material_name: None,
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
        }
    }
}

impl fmt::Display for Mesh {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            v,
            "Mesh (\"{}\")",
            self.object_name.as_ref().unwrap_or(&"Unnamed".to_string())
        )?;

        writeln!(
            v,
            "  > Source: {}",
            self.object_source
                .as_ref()
                .unwrap_or(&"No source".to_string())
        )?;

        writeln!(
            v,
            "  > Group name: {}",
            self.group_name.as_ref().unwrap_or(&"No group".to_string())
        )?;

        writeln!(
            v,
            "  > Material source: {}",
            self.material_source
                .as_ref()
                .unwrap_or(&"No material".to_string())
        )?;

        writeln!(
            v,
            "  > Material name: {}",
            self.material_name
                .as_ref()
                .unwrap_or(&"Unnamed".to_string())
        )?;

        writeln!(v, "  > Vertices: {}", self.vertices.len())?;
        writeln!(v, "  > UVs: {}", self.uvs.len())?;
        writeln!(v, "  > Normals: {}", self.normals.len())?;
        writeln!(v, "  > Faces: {}", self.faces.len())
    }
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, uvs: Vec<Vec2>, normals: Vec<Vec3>, faces: Vec<Face>) -> Self {
        let mut mesh = Mesh::default();

        mesh.vertices = vertices;
        mesh.normals = normals;
        mesh.uvs = uvs;
        mesh.faces = faces;

        mesh
    }
}
