use std::{collections::HashMap, fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use self::geometry::Geometry;

use crate::{physics::collision::aabb::AABB, serde::PostDeserialize, vec::vec3::Vec3};

pub mod geometry;
pub mod obj;
pub mod primitive;

static TANGENT_BITANGENT_SMOOTHING_LIKENESS_THRESHOLD: f32 = 4.0;

macro_rules! smooth_tangents_or_bitangents {
    ($self:ident, $field:ident, &mut $frontier:ident) => {
        // Process local tangents/bitangents in batches, based on
        // their level of similarity (using a threshold value).

        while !$frontier.is_empty() {
            let mut smoothing_group = vec![$frontier.pop().unwrap()];

            if !$frontier.is_empty() {
                let to_visit = $frontier.len();

                for i in 0..to_visit {
                    let (_face_index, _vertex_index_position, face_vertex_tangent_or_bitangent) =
                        &$frontier[i - (smoothing_group.len() - 1)];

                    if smoothing_group[0].2.dot(*face_vertex_tangent_or_bitangent)
                        >= TANGENT_BITANGENT_SMOOTHING_LIKENESS_THRESHOLD
                    {
                        smoothing_group.push($frontier.pop().unwrap());
                    }
                }
            }

            let smooth_tangent = {
                let mut st: Vec3 = Default::default();

                for (_face_index, _vertex_index_position, face_vertex_tangent_or_bitangent) in
                    &smoothing_group
                {
                    st += *face_vertex_tangent_or_bitangent;
                }

                st = st.as_normal();

                st
            };

            for (face_index, vertex_index_position, _face_vertex_tangent_or_bitangent) in
                &smoothing_group
            {
                $self.faces[*face_index].$field[*vertex_index_position] = smooth_tangent;
            }
        }
    };
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PartialFace {
    pub vertices: [usize; 3],
    pub normals: Option<[usize; 3]>,
    pub uvs: Option<[usize; 3]>,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Face {
    pub vertices: [usize; 3],
    pub normals: [usize; 3],
    pub uvs: [usize; 3],
    pub tangents: [Vec3; 3],
    pub bitangents: [Vec3; 3],
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub object_source: Option<String>,
    pub object_name: Option<String>,
    pub group_name: Option<String>,
    pub material_source: Option<String>,
    pub material_name: Option<String>,
    pub geometry: Rc<Geometry>,
    pub faces: Vec<Face>,
    #[serde(skip)]
    pub aabb: AABB,
}

impl PostDeserialize for Mesh {
    fn post_deserialize(&mut self) {
        self.aabb = self.make_object_space_bounding_box();
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
        )
    }
}

fn get_processed_faces(geometry: &Rc<Geometry>, partial_faces: &Vec<PartialFace>) -> Vec<Face> {
    let mut faces: Vec<Face> = Vec::<Face>::with_capacity(partial_faces.len());

    for partial_face in partial_faces {
        let normals = match partial_face.normals {
            Some(face_normal_indices) => face_normal_indices,
            None => {
                todo!("Compute flat normals for this face, insert into Geometry.normals, and return indices.");
            }
        };

        let (uvs, tangent, bitangent) = match partial_face.uvs {
            Some(face_uv_indices) => {
                let (v0, v1, v2) = (
                    geometry.vertices[partial_face.vertices[0]],
                    geometry.vertices[partial_face.vertices[1]],
                    geometry.vertices[partial_face.vertices[2]],
                );

                let (uv0, uv1, uv2) = (
                    geometry.uvs[face_uv_indices[0]],
                    geometry.uvs[face_uv_indices[1]],
                    geometry.uvs[face_uv_indices[2]],
                );

                // Compute and store tangent and bitangent vectors, on a
                // per-vertex-per-face basis; tangent and bitangent are computed based
                // on a (computed) hard normal, ignoring the imported normals as they
                // may have smoothing applied.

                let (tangent, bitangent) = {
                    let edge0 = v1 - v0;
                    let edge1 = v2 - v0;

                    let delta_uv0 = uv1 - uv0;
                    let delta_uv1 = uv2 - uv0;

                    let f = 1.0 / (delta_uv0.x * delta_uv1.y - delta_uv1.x * delta_uv0.y);

                    (
                        Vec3 {
                            x: f * (delta_uv1.y * edge0.x - delta_uv0.y * edge1.x),
                            y: f * (delta_uv1.y * edge0.y - delta_uv0.y * edge1.y),
                            z: f * (delta_uv1.y * edge0.z - delta_uv0.y * edge1.z),
                        },
                        Vec3 {
                            x: f * (-delta_uv1.x * edge0.x + delta_uv0.x * edge1.x),
                            y: f * (-delta_uv1.x * edge0.y + delta_uv0.x * edge1.y),
                            z: f * (-delta_uv1.x * edge0.z + delta_uv0.x * edge1.z),
                        },
                    )
                };

                (face_uv_indices, tangent, bitangent)
            }
            None => {
                // Can't derive UVs, and can't compute tangent/bitangent; we leave them "blank".
                Default::default()
            }
        };

        faces.push(Face {
            vertices: partial_face.vertices.to_owned(),
            normals,
            uvs,
            tangents: [tangent, tangent, tangent],
            bitangents: [bitangent, bitangent, bitangent],
        });
    }

    faces
}

impl Mesh {
    pub fn new(
        geometry: Rc<Geometry>,
        partial_faces: Vec<PartialFace>,
        material_name: Option<String>,
    ) -> Self {
        let faces = get_processed_faces(&geometry, &partial_faces);

        let mut mesh = Mesh {
            object_source: None,
            object_name: None,
            group_name: None,
            material_source: None,
            material_name,
            geometry,
            faces,
            aabb: Default::default(),
        };

        mesh.post_deserialize();

        mesh.post_process().unwrap();

        mesh
    }

    fn post_process(&mut self) -> Result<(), String> {
        // Tangent and bitangent smoothing.

        let mut face_indices_per_vertex = HashMap::<usize, Vec<usize>>::new();

        for (face_index, face) in self.faces.iter().enumerate() {
            for vertex_index in &face.vertices {
                match face_indices_per_vertex.get_mut(vertex_index) {
                    Some(entry) => {
                        entry.push(face_index);
                    }
                    None => {
                        face_indices_per_vertex.insert(*vertex_index, vec![face_index]);
                    }
                }
            }
        }

        let vertex_indices: Vec<usize> = face_indices_per_vertex.keys().copied().collect();

        static VERTEX_CONNECTIVITY_THRESHOLD: usize = 32;

        for vertex_index in &vertex_indices {
            match face_indices_per_vertex.get(vertex_index) {
                Some(face_indices) => {
                    if face_indices.len() >= VERTEX_CONNECTIVITY_THRESHOLD {
                        continue;
                    }

                    let mut tangents: Vec<(usize, usize, Vec3)> = vec![];
                    let mut bitangents: Vec<(usize, usize, Vec3)> = vec![];

                    for face_index in face_indices {
                        let face = &self.faces[*face_index];

                        let vertex_index_position = face
                            .vertices
                            .iter()
                            .position(|i| *i == *vertex_index)
                            .unwrap();

                        let face_vertex_tangent = face.tangents[vertex_index_position];

                        tangents.push((*face_index, vertex_index_position, face_vertex_tangent));

                        let face_vertex_bitangent = face.bitangents[vertex_index_position];

                        bitangents.push((
                            *face_index,
                            vertex_index_position,
                            face_vertex_bitangent,
                        ));
                    }

                    // Tangent smoothing pass
                    smooth_tangents_or_bitangents!(self, tangents, &mut tangents);

                    // Bitangent smoothing pass
                    smooth_tangents_or_bitangents!(self, bitangents, &mut bitangents);
                }
                None => panic!(),
            }
        }

        Ok(())
    }

    fn make_object_space_bounding_box(&self) -> AABB {
        AABB::from_mesh(self)
    }
}
