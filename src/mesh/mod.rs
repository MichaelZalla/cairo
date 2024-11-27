use std::{collections::HashMap, fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    geometry::{accelerator::static_triangle_bvh::StaticTriangleBVH, primitives::aabb::AABB},
    resource::handle::Handle,
    serde::PostDeserialize,
    vec::vec3::Vec3,
};

use face::{get_processed_faces, Face, PartialFace};
use mesh_geometry::MeshGeometry;

pub mod face;
pub mod mesh_geometry;
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub object_source: Option<String>,
    pub object_name: Option<String>,
    pub group_name: Option<String>,
    pub material_source: Option<String>,
    pub material: Option<Handle>,
    pub geometry: Rc<MeshGeometry>,
    pub faces: Vec<Face>,
    #[serde(skip)]
    pub aabb: AABB,
    #[serde(skip)]
    pub collider: Option<Rc<StaticTriangleBVH>>,
}

impl PostDeserialize for Mesh {
    fn post_deserialize(&mut self) {
        self.aabb = AABB::from_mesh(self);
    }
}

impl fmt::Display for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mesh")
            .field("object_source", &self.object_source)
            .field("object_name", &self.object_name)
            .field("group_name", &self.group_name)
            .field("material_source", &self.material_source)
            .field("material", &self.material)
            .finish()
    }
}

impl Mesh {
    pub fn new(
        geometry: Rc<MeshGeometry>,
        partial_faces: Vec<PartialFace>,
        material: Option<Handle>,
    ) -> Self {
        let faces = get_processed_faces(&geometry, &partial_faces);

        let mut mesh = Mesh {
            object_source: None,
            object_name: None,
            group_name: None,
            material_source: None,
            material,
            geometry,
            faces,
            aabb: Default::default(),
            collider: None,
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
}
