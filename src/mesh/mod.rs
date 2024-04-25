use std::{fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use self::geometry::Geometry;

use crate::{physics::collision::aabb::AABB, serde::PostDeserialize};

pub mod geometry;
pub mod obj;
pub mod primitive;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Face {
    pub vertices: (usize, usize, usize), // Indices into Vec<Vec3>
    pub normals: Option<(usize, usize, usize)>, // Indices into Vec<Vec3>
    pub uvs: Option<(usize, usize, usize)>, // Indices into Vec<Vec2>
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub object_source: Option<String>,
    pub object_name: Option<String>,
    pub group_name: Option<String>,
    pub material_source: Option<String>,
    pub material_name: Option<String>,
    pub geometry: Option<Rc<Geometry>>,
    pub faces: Vec<Face>,
    #[serde(skip)]
    pub aabb: AABB,
    // #[serde(skip)]
    // pub aabb_geometry: Geometry,
}

impl PostDeserialize for Mesh {
    fn post_deserialize(&mut self) {
        match &self.geometry {
            Some(geometry) => {
                self.aabb = geometry.make_object_space_bounding_box();

                // self.aabb_geometry = make_bounding_box_geometry(&self.aabb);
            }
            None => (),
        }
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

impl Mesh {
    pub fn new(
        geometry: Option<Rc<Geometry>>,
        faces: Vec<Face>,
        material_name: Option<String>,
    ) -> Self {
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
