use std::fmt::{self};

use serde::{Deserialize, Serialize};

use crate::{
    mesh::{mesh_geometry::MeshGeometry, Mesh},
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
    pub bounding_sphere_radius: f32,
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: vec3::MAX,
            max: vec3::MIN,
            bounding_sphere_radius: 0.0,
        }
    }
}

impl fmt::Display for AABB {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(v, "AABB (min={}, max={})", self.min, self.max)
    }
}

impl AABB {
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        let center = min + (max - min) / 2.0;

        let bounding_sphere_radius = (max - center).mag();

        Self {
            min,
            max,
            bounding_sphere_radius,
        }
    }

    pub fn from_geometry(geometry: &MeshGeometry) -> Self {
        let (min, max) = get_min_max_for_geometry(geometry);

        AABB::from_min_max(min, max)
    }

    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (min, max) = get_min_max_for_mesh(mesh);

        AABB::from_min_max(min, max)
    }

    pub fn center(&self) -> Vec3 {
        self.min + (self.max + self.min) / 2.0
    }

    pub fn extent(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn get_vertices(&self) -> [Vec3; 8] {
        let left = Vec3 {
            x: self.min.x,
            ..Default::default()
        };

        let right = Vec3 {
            x: self.max.x,
            ..Default::default()
        };

        let top = Vec3 {
            y: self.max.y,
            ..Default::default()
        };

        let bottom = Vec3 {
            y: self.min.y,
            ..Default::default()
        };

        let near = Vec3 {
            z: self.min.z,
            ..Default::default()
        };

        let far = Vec3 {
            z: self.max.z,
            ..Default::default()
        };

        [
            // 0. Near top left
            near + top + left,
            // 1. Near top right
            near + top + right,
            // 2. Near bottom right
            near + bottom + right,
            // 3. Near bottom left
            self.min,
            // 4. Far top left
            far + top + left,
            // 5. Far top right
            self.max,
            // 6. Far bottom right
            far + bottom + right,
            // 7. Far bottom left
            far + bottom + left,
        ]
    }

    pub fn intersects(&self, rhs: &Self) -> bool {
        if self.max.x < rhs.min.x
            || self.min.x > rhs.max.x
            || self.max.y < rhs.min.y
            || self.min.y > rhs.max.y
            || self.max.z > rhs.min.z
            || self.min.z < rhs.max.z
        {
            return false;
        }

        true
    }

    pub fn grow(&mut self, point: &Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn subdivide_2d(&self) -> [Self; 4] {
        let center = self.center();

        let top_left_subdivision = Self::from_min_max(
            Vec3 {
                x: self.min.x,
                y: center.y,
                z: 0.0,
            },
            Vec3 {
                x: center.x,
                y: self.max.y,
                z: 0.0,
            },
        );

        let top_right_subdivision = Self::from_min_max(
            Vec3 {
                x: center.x,
                y: center.y,
                z: 0.0,
            },
            Vec3 {
                x: self.max.x,
                y: self.max.y,
                z: 0.0,
            },
        );

        let bottom_left_subdivision = Self::from_min_max(
            Vec3 {
                x: self.min.x,
                y: self.min.y,
                z: 0.0,
            },
            Vec3 {
                x: center.x,
                y: center.y,
                z: 0.0,
            },
        );

        let bottom_right_subdivision = Self::from_min_max(
            Vec3 {
                x: center.x,
                y: self.min.y,
                z: 0.0,
            },
            Vec3 {
                x: self.max.x,
                y: center.y,
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
