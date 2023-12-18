use crate::color;
use crate::vec::vec2::Vec2;
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
        uv_coordinates: Vec<Vec2>,
        vertex_normals: Vec<Vec3>,
        face_vertex_indices: Vec<(usize, usize, usize)>,
        face_vertex_uv_coordinate_indices: Vec<(usize, usize, usize)>,
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

            for (face_index, vertex_indices) in face_vertex_indices.iter().enumerate() {
                let normal_indices = face_vertex_normal_indices[face_index];

                let mut v0_in = DefaultVertexIn {
                    p: vertices[vertex_indices.0].clone(),
                    n: vertex_normals[normal_indices.0].clone(),
                    c: white.clone(),
                    uv: Default::default(),
                };
                let mut v1_in = DefaultVertexIn {
                    p: vertices[vertex_indices.1].clone(),
                    n: vertex_normals[normal_indices.1].clone(),
                    c: white.clone(),
                    uv: Default::default(),
                };
                let mut v2_in = DefaultVertexIn {
                    p: vertices[vertex_indices.2].clone(),
                    n: vertex_normals[normal_indices.2].clone(),
                    c: white.clone(),
                    uv: Default::default(),
                };

                if face_vertex_uv_coordinate_indices.len() > 0 {
                    let uv_coordinate_indices: (usize, usize, usize) =
                        face_vertex_uv_coordinate_indices[face_index];
                    v0_in.uv = uv_coordinates[uv_coordinate_indices.0].clone();
                    v1_in.uv = uv_coordinates[uv_coordinate_indices.1].clone();
                    v2_in.uv = uv_coordinates[uv_coordinate_indices.2].clone();
                }

                mesh.vertices.push(v0_in);
                mesh.vertices.push(v1_in);
                mesh.vertices.push(v2_in);

                mesh.face_indices
                    .push((face_index * 3, face_index * 3 + 1, face_index * 3 + 2))
            }
        } else if vertex_normals_len != vertices_len {
            // Case 2. No normal data was provided; we'll generate a normal for each
            // face, creating 3 unique Vertex instances for that face;

            for (face_index, vertex_indices) in face_vertex_indices.iter().enumerate() {
                let uv_coordinate_indices = face_vertex_uv_coordinate_indices[face_index];

                let computed_normal = (vertices[vertex_indices.1] - vertices[vertex_indices.0])
                    .cross(vertices[vertex_indices.2] - vertices[vertex_indices.0])
                    .as_normal();

                let v0_in = DefaultVertexIn {
                    p: vertices[vertex_indices.0].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    uv: uv_coordinates[uv_coordinate_indices.0].clone(),
                };
                let v1_in = DefaultVertexIn {
                    p: vertices[vertex_indices.1].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    uv: uv_coordinates[uv_coordinate_indices.1].clone(),
                };
                let v2_in = DefaultVertexIn {
                    p: vertices[vertex_indices.2].clone(),
                    n: computed_normal.clone(),
                    c: white.clone(),
                    uv: uv_coordinates[uv_coordinate_indices.2].clone(),
                };

                mesh.vertices.push(v0_in);
                mesh.vertices.push(v1_in);
                mesh.vertices.push(v2_in);

                mesh.face_indices
                    .push((face_index * 3, face_index * 3 + 1, face_index * 3 + 2))
            }
        }

        if vertex_normals_len == vertices_len {
            // Case 3. One normal is defined per-vertex; no need for duplicate Vertexs;

            for (vertex_index, vertex) in vertices.iter().enumerate() {
                let v_in = DefaultVertexIn {
                    p: vertex.clone(),
                    n: vertex_normals[vertex_index].clone(),
                    c: white.clone(),
                    uv: uv_coordinates[vertex_index],
                };

                mesh.vertices.push(v_in);
            }

            mesh.face_indices = face_vertex_indices;
        }

        return mesh;
    }
}
