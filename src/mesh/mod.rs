use serde::{Deserialize, Serialize};

use self::geometry::Geometry;

use super::vec::vec3::Vec3;
use crate::{physics::collision::aabb::AABB, serde::PostDeserialize};

pub mod geometry;
pub mod obj;
pub mod primitive;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub geometry: Geometry,
    #[serde(skip)]
    pub aabb: AABB,
    // #[serde(skip)]
    // pub aabb_geometry: Geometry,
}

impl PostDeserialize for Mesh {
    fn post_deserialize(&mut self) {
        self.aabb = make_object_space_bounding_box(&self.geometry);
        // self.aabb_geometry = make_bounding_box_geometry(&self.aabb);
    }
}

impl Mesh {
    pub fn new(geometry: Geometry) -> Self {
        let mut mesh = Mesh {
            geometry,
            aabb: Default::default(),
            // aabb_geometry: Default::default(),
        };

        mesh.post_deserialize();

        mesh
    }
}

fn make_object_space_bounding_box(geometry: &Geometry) -> AABB {
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;

    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;

    let mut z_min = f32::MAX;
    let mut z_max = f32::MIN;

    for v in geometry.vertices.as_slice() {
        if v.x < x_min {
            x_min = v.x;
        } else if v.x > x_max {
            x_max = v.x;
        }

        if v.y < y_min {
            y_min = v.y;
        } else if v.y > y_max {
            y_max = v.y;
        }

        if v.z < z_min {
            z_min = v.z;
        } else if v.z > z_max {
            z_max = v.z;
        }
    }

    let width = x_max - x_min;
    let height = y_max - y_min;
    let depth = z_max - z_min;

    AABB {
        center: Vec3 {
            x: x_min + width / 2.0,
            y: y_min + height / 2.0,
            z: z_min + depth / 2.0,
        },
        half_dimension: (x_max - x_min) / 2.0,
        left: x_min,
        right: x_max,
        top: y_max,
        bottom: y_min,
        near: z_max,
        far: z_min,
    }
}

fn make_bounding_box_geometry(aabb: &AABB) -> Geometry {
    let width = aabb.right - aabb.left;
    let height = aabb.top - aabb.bottom;
    let depth = aabb.near - aabb.far;

    let mut bounding_box_mesh = primitive::cube::generate(width, height, depth);

    for v in bounding_box_mesh.vertices.as_mut_slice() {
        *v += aabb.center;
    }

    bounding_box_mesh
}
