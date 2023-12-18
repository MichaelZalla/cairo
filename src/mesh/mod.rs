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
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub faces: Vec<Face>,
}

impl<'a> Default for &'a Mesh {
    fn default() -> &'a Mesh {
        static DEFAULT: Mesh = Mesh {
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
            faces: vec![],
        };
        &DEFAULT
    }
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, uvs: Vec<Vec2>, normals: Vec<Vec3>, faces: Vec<Face>) -> Self {
        let mesh = Mesh {
            vertices,
            normals,
            uvs,
            faces,
        };

        return mesh;
    }
}
