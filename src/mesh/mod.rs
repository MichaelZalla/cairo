use std::{fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use self::geometry::Geometry;

use crate::{physics::collision::aabb::AABB, serde::PostDeserialize};

pub mod geometry;
pub mod obj;
pub mod primitive;

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
    // #[serde(skip)]
    // pub aabb_geometry: Geometry,
}

impl PostDeserialize for Mesh {
    fn post_deserialize(&mut self) {
        self.aabb = self.geometry.make_object_space_bounding_box();
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

fn get_processed_faces(_geometry: &Rc<Geometry>, partial_faces: &Vec<PartialFace>) -> Vec<Face> {
    let mut faces: Vec<Face> = Vec::<Face>::with_capacity(partial_faces.len());

    for partial_face in partial_faces {
        let normals = match partial_face.normals {
            Some(face_normal_indices) => face_normal_indices,
            None => {
                todo!("Compute flat normals for this face, insert into Geometry.normals, and return indices.");
            }
        };

        let uvs = match partial_face.uvs {
            Some(face_uv_indices) => face_uv_indices,
            None => {
                // Can't derive UVs; we leave them "blank".
                Default::default()
            }
        };

        faces.push(Face {
            vertices: partial_face.vertices.to_owned(),
            normals,
            uvs,
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
            // aabb_geometry: Default::default(),
        };

        mesh.post_deserialize();

        mesh
    }
}
