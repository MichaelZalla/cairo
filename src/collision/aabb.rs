use std::fmt::{self};

use crate::{vec::vec3::Vec3, vertex::default_vertex_in::DefaultVertexIn};

#[derive(Debug, Default, Copy, Clone)]
pub struct AABB {
    pub center: Vec3,
    pub half_dimension: f32,

    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
}

impl AABB {
    pub fn new(center: Vec3, half_dimension: f32) -> Self {
        return AABB {
            center,
            half_dimension,
            left: center.x - half_dimension,
            right: center.x + half_dimension,
            top: center.y + half_dimension,
            bottom: center.y - half_dimension,
            near: center.z + half_dimension,
            far: center.z - half_dimension,
        };
    }

    pub fn new_from_triangle(
        v0: &DefaultVertexIn,
        v1: &DefaultVertexIn,
        v2: &DefaultVertexIn,
    ) -> Self {
        let min_x = v0.p.x.min(v1.p.x).min(v2.p.x);

        let max_x = v0.p.x.max(v1.p.x).max(v2.p.x);

        let min_y = v0.p.y.min(v1.p.y).min(v2.p.y);

        let max_y = v0.p.y.max(v1.p.y).max(v2.p.y);

        let min_z = v0.p.z.min(v1.p.z).min(v2.p.z);

        let max_z = v0.p.z.max(v1.p.z).max(v2.p.z);

        let center = Vec3 {
            x: min_x + (max_x - min_x) / 2.0,
            y: min_y + (max_y - min_y) / 2.0,
            z: min_z + (max_z - min_z) / 2.0,
        };

        let largest_dimension = (max_x - min_x).max(max_y - min_y).max(max_z - min_z);

        let half_dimension = largest_dimension / 2.0;

        return AABB::new(center, half_dimension);
    }

    pub fn contains_point(&self, p: Vec3) -> bool {
        if p.x < self.left
            || p.x > self.right
            || p.y < self.bottom
            || p.y > self.top
            || p.z < self.near
            || p.z > self.far
        {
            return false;
        }

        return true;
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

        return true;
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
