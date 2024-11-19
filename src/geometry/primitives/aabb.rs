use std::fmt::{self};

use serde::{Deserialize, Serialize};

use crate::{
    mesh::{mesh_geometry::MeshGeometry, Mesh},
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct AABB {
    pub center: Vec3,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
    pub bounding_sphere_radius: f32,
}

impl AABB {
    pub fn cube(center: Vec3, half_extent: f32) -> Self {
        let mut aabb = AABB {
            center,
            left: center.x - half_extent,
            right: center.x + half_extent,
            top: center.y + half_extent,
            bottom: center.y - half_extent,
            near: center.z + half_extent,
            far: center.z - half_extent,
            bounding_sphere_radius: 0.0,
        };

        aabb.bounding_sphere_radius = aabb.get_bounding_sphere_radius();

        aabb
    }

    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        let half_extents = Vec3 {
            x: (max.x - min.x),
            y: (max.y - min.y),
            z: (max.z - min.z),
        } / 2.0;

        let center = Vec3 {
            x: min.x,
            y: min.y,
            z: min.z,
        } + half_extents;

        let mut aabb = AABB {
            center,
            left: min.x,
            right: max.x,
            top: max.y,
            bottom: min.y,
            near: max.z,
            far: min.z,
            bounding_sphere_radius: 0.0,
        };

        aabb.bounding_sphere_radius = aabb.get_bounding_sphere_radius();

        aabb
    }

    pub fn from_geometry(geometry: &MeshGeometry) -> Self {
        let (min, max) = get_min_max_for_geometry(geometry);

        AABB::from_min_max(min, max)
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (min, max) = get_min_max_for_mesh(mesh);

        AABB::from_min_max(min, max)
    }

    pub fn new_from_triangle(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Self {
        let (min, max) = Vec3::extent(&[*v0, *v1, *v2]);

        AABB::from_min_max(min, max)
    }

    pub fn intersects(&self, rhs: &Self) -> bool {
        if self.right < rhs.left
            || self.left > rhs.right
            || self.top < rhs.bottom
            || self.bottom > rhs.top
            || self.far > rhs.near
            || self.near < rhs.far
        {
            return false;
        }

        true
    }

    pub fn subdivide_2d(&self) -> [Self; 4] {
        let top_left_subdivision = Self::from_min_max(
            Vec3 {
                x: self.left,
                y: self.center.y,
                z: 0.0,
            },
            Vec3 {
                x: self.center.x,
                y: self.top,
                z: 0.0,
            },
        );

        let top_right_subdivision = Self::from_min_max(
            Vec3 {
                x: self.center.x,
                y: self.center.y,
                z: 0.0,
            },
            Vec3 {
                x: self.right,
                y: self.top,
                z: 0.0,
            },
        );

        let bottom_left_subdivision = Self::from_min_max(
            Vec3 {
                x: self.left,
                y: self.bottom,
                z: 0.0,
            },
            Vec3 {
                x: self.center.x,
                y: self.center.y,
                z: 0.0,
            },
        );

        let bottom_right_subdivision = Self::from_min_max(
            Vec3 {
                x: self.center.x,
                y: self.bottom,
                z: 0.0,
            },
            Vec3 {
                x: self.right,
                y: self.center.y,
                z: 0.0,
            },
        );

        [
            top_left_subdivision,
            top_right_subdivision,
            bottom_left_subdivision,
            bottom_right_subdivision,
        ]
    }

    fn get_bounding_sphere_radius(&self) -> f32 {
        // Center-to-corner is equidistant for all corners on the AABB.

        let top_left_near = Vec3 {
            x: self.left,
            y: self.top,
            z: self.near,
        };

        (top_left_near - self.center).mag()
    }
}

impl fmt::Display for AABB {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            v,
            "AABB (center={}) (l={}, r={}, b={}, t={}, n={}, f={})",
            self.center, self.left, self.right, self.bottom, self.top, self.near, self.far
        )
    }
}

fn get_min_max_for_geometry(geometry: &MeshGeometry) -> (Vec3, Vec3) {
    let mut min = vec3::MAX;
    let mut max = vec3::MIN;

    for v in geometry.vertices.iter() {
        min = min.min(v);
        max = max.max(v);
    }

    (min, max)
}

fn get_min_max_for_mesh(mesh: &Mesh) -> (Vec3, Vec3) {
    let mut min = vec3::MAX;
    let mut max = vec3::MIN;

    for face in &mesh.faces {
        for vertex_index in &face.vertices {
            let v = &mesh.geometry.vertices[*vertex_index];

            min = min.min(v);
            max = max.max(v);
        }
    }

    (min, max)
}
