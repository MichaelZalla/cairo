use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::vec::vec3::Vec3;

use super::mesh_geometry::MeshGeometry;

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

pub(in crate::mesh) fn get_processed_faces(
    geometry: &Rc<MeshGeometry>,
    partial_faces: &Vec<PartialFace>,
) -> Vec<Face> {
    let mut faces: Vec<Face> = Vec::<Face>::with_capacity(partial_faces.len());

    for partial_face in partial_faces {
        let normals = match partial_face.normals {
            Some(face_normal_indices) => face_normal_indices,
            None => {
                todo!(
                    "Compute flat normals for this face, insert into Geometry.normals, and return indices."
                );
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
