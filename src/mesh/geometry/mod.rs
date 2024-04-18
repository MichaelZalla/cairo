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

impl Default for Geometry {
    fn default() -> Self {
        Self {
            object_source: None,
            object_name: None,
            group_name: None,
            material_source: None,
            material_name: None,
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            v,
            "Geometry (\"{}\")",
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
