use std::fmt;

use crate::vec::vec2::Vec2;

use super::vec::vec3::Vec3;

pub mod obj;
pub mod primitive;

#[derive(Default, Copy, Clone)]
pub struct Face {
    pub vertices: (usize, usize, usize), // Indices into Vec<Vec3>
    pub normals: Option<(usize, usize, usize)>, // Indices into Vec<Vec3>
    pub uvs: Option<(usize, usize, usize)>, // Indices into Vec<Vec2>
}

#[derive(Default, Clone)]
pub struct MaterialSource {
    pub filepath: String,
}

#[derive(Clone)]
pub struct Mesh {
    pub object_name: String,
    pub group_name: String,
    pub material_source: MaterialSource,
    pub material_name: String,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub faces: Vec<Face>,
}

impl Default for Mesh {
    fn default() -> Mesh {
        Mesh {
            object_name: "__undefined__".to_string(),
            group_name: "__undefined__".to_string(),
            material_source: Default::default(),
            material_name: "__undefined__".to_string(),
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
        }
    }
}

impl fmt::Display for Mesh {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Mesh (\"{}\")", self.object_name)?;
        writeln!(v, "  > Object name: {}", self.object_name)?;
        writeln!(v, "  > Group name: {}", self.group_name)?;
        writeln!(v, "  > Material source: {}", self.material_source.filepath)?;
        writeln!(v, "  > Material name: {}", self.material_name)?;
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

        return mesh;
    }
}
