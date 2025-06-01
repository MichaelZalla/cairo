use std::{collections::HashMap, ops};

use crate::{geometry::primitives::aabb::Bounded, vec::vec3::Vec3};

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct GridSpaceCoordinate {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl ops::Add for GridSpaceCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl From<(Vec3, f32)> for GridSpaceCoordinate {
    fn from(vec_and_scale: (Vec3, f32)) -> Self {
        let (vec, scale) = vec_and_scale;

        Self {
            x: (vec.x / scale).floor() as isize,
            y: (vec.y / scale).floor() as isize,
            z: (vec.z / scale).floor() as isize,
        }
    }
}

impl From<(GridSpaceCoordinate, f32)> for Vec3 {
    fn from(val: (GridSpaceCoordinate, f32)) -> Self {
        let (coord, scale) = &val;

        Vec3 {
            x: coord.x as f32 * scale,
            y: coord.y as f32 * scale,
            z: coord.z as f32 * scale,
        }
    }
}

impl<T: Bounded> From<(&T, HashGridInsertionStrategy, f32)> for GridSpaceCoordinate {
    fn from(data: (&T, HashGridInsertionStrategy, f32)) -> Self {
        let (bounded, strategy, scale) = data;

        let aabb = bounded.aabb();

        let point_of_interest = match strategy {
            HashGridInsertionStrategy::AABBCenter => aabb.center(),
        };

        Self::from((point_of_interest, scale))
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub enum HashGridInsertionStrategy {
    #[default]
    AABBCenter,
}

#[derive(Debug, Clone)]
pub struct HashGrid {
    pub scale: f32,
    pub map: HashMap<GridSpaceCoordinate, Vec<usize>>,
    pub strategy: HashGridInsertionStrategy,
}

impl Default for HashGrid {
    fn default() -> Self {
        Self {
            strategy: Default::default(),
            scale: 1.0,
            map: Default::default(),
        }
    }
}

impl HashGrid {
    pub fn new(strategy: HashGridInsertionStrategy, scale: f32) -> Self {
        Self {
            strategy,
            scale,
            ..Default::default()
        }
    }
}
