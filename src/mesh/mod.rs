use crate::color;
use crate::vertex::default_vertex_in::DefaultVertexIn;

use super::vec::vec3::Vec3;

pub mod obj;
pub mod primitive;

pub type Face = (usize, usize, usize);

#[derive(Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<DefaultVertexIn>,
    pub face_indices: Vec<(usize, usize, usize)>,
}

impl<'a> Default for &'a Mesh {
    fn default() -> &'a Mesh {
        static DEFAULT: Mesh = Mesh {
            vertices: vec![],
            face_indices: vec![],
        };
        &DEFAULT
    }
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vec3>,
        vertex_normals: Vec<Vec3>,
        face_vertex_indices: Vec<(usize, usize, usize)>,
        face_vertex_normal_indices: Vec<(usize, usize, usize)>,
    ) -> Self {
        let vertices_len = vertices.len();
        let vertex_normals_len = vertex_normals.len();
        let face_vertex_normal_indices_len = face_vertex_normal_indices.len();

        let mut mesh: Mesh = Mesh {
            vertices: vec![],
            face_indices: vec![],
        };

        let white = color::WHITE.to_vec3() / 255.0;

        if face_vertex_normal_indices_len == face_vertex_indices.len() {
            // Case 1. 3 vertex normals are defined per face;

            for (face_index, face) in face_vertex_indices.iter().enumerate() {
                let normal_indices = face_vertex_normal_indices[face_index];

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.0].clone(),
                    n: vertex_normals[normal_indices.0].clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.1].clone(),
                    n: vertex_normals[normal_indices.1].clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.2].clone(),
                    n: vertex_normals[normal_indices.2].clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.face_indices
                    .push((face_index * 3, face_index * 3 + 1, face_index * 3 + 2))
            }
        } else if vertex_normals_len != vertices_len {
            // Case 2. No normal data was provided; we'll generate a normal for each
            // face, creating 3 unique Vertex instances for that face;

            for (face_index, face) in face_vertex_indices.iter().enumerate() {
                let computed_normal = (vertices[face.1] - vertices[face.0])
                    .cross(vertices[face.2] - vertices[face.0])
                    .as_normal();

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.0].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.1].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.vertices.push(DefaultVertexIn {
                    p: vertices[face.2].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                });

                mesh.face_indices
                    .push((face_index * 3, face_index * 3 + 1, face_index * 3 + 2))
            }
        }

        if vertex_normals_len == vertices_len {
            // Case 3. One normal is defined per-vertex; no need for duplicate Vertexs;

            for (vertex_index, vertex) in vertices.iter().enumerate() {
                mesh.vertices.push(DefaultVertexIn {
                    p: vertex.clone(),
                    n: vertex_normals[vertex_index].clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                })
            }

            mesh.face_indices = face_vertex_indices;
        }

        return mesh;
    }
}
