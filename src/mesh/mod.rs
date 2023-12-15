use std::fs::File;
use std::io::{self, BufRead};

use std::path::Path;

use crate::vertex::default_vertex_in::DefaultVertexIn;

use super::vec::vec3::Vec3;

pub mod obj;
pub mod primitive;

pub type Face = (usize, usize, usize);

#[derive(Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<DefaultVertexIn>,
    pub face_indices: Vec<Face>,
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
        faces: Vec<Face>,
        vertex_normals: Vec<Vec3>,
        face_normals: Vec<(usize, usize, usize)>,
    ) -> Self {
        let mesh_v_len = vertices.len();
        let mesh_vn_len = vertex_normals.len();
        let mesh_tn_len = face_normals.len();

        let mut mesh: Mesh = Mesh {
            vertices: vec![],
            face_indices: vec![],
        };

        let white = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };

        if mesh_tn_len == faces.len() {
            // Case 1. 3 vertex normals are defined per face;

            for (face_index, face) in faces.iter().enumerate() {
                let normal_indices = face_normals[face_index];

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
        } else if mesh_vn_len != mesh_v_len {
            // Case 2. No normal data was provided; we'll generate a normal for each
            // face, creating 3 unique Vertex instances for that face;

            for (face_index, face) in faces.iter().enumerate() {
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

        if mesh_vn_len == mesh_v_len {
            // Case 3. One normal is defined per-vertex; no need for duplicate Vertexs;

            for (vertex_index, vertex) in vertices.iter().enumerate() {
                mesh.vertices.push(DefaultVertexIn {
                    p: vertex.clone(),
                    n: vertex_normals[vertex_index].clone(),
                    c: white.clone(),
                    world_pos: Vec3::new(),
                })
            }

            mesh.face_indices = faces;
        }

        return mesh;
    }
}

fn read_lines<P>(filepath: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;

    Ok(io::BufReader::new(file).lines())
}
