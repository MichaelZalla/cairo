use std::fmt::{self};

use crate::vec::vec3::Vec3;

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

    pub fn new_from_triangle(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Self {
        let min_x = v0.x.min(v1.x).min(v2.x);

        let max_x = v0.x.max(v1.x).max(v2.x);

        let min_y = v0.y.min(v1.y).min(v2.y);

        let max_y = v0.y.max(v1.y).max(v2.y);

        let min_z = v0.z.min(v1.z).min(v2.z);

        let max_z = v0.z.max(v1.z).max(v2.z);

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
