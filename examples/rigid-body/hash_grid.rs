use std::{collections::HashMap, ops};

use cairo::{
    physics::simulation::rigid_body::rigid_body_simulation_state::RigidBodySimulationState,
    vec::vec3::Vec3,
};

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

impl From<(&RigidBodySimulationState, f32)> for GridSpaceCoordinate {
    fn from(state_and_scale: (&RigidBodySimulationState, f32)) -> Self {
        let (state, scale) = state_and_scale;

        Self::from((state.position, scale))
    }
}

#[derive(Debug, Clone)]
pub struct HashGrid {
    pub scale: f32,
    pub map: HashMap<GridSpaceCoordinate, Vec<usize>>,
}

impl Default for HashGrid {
    fn default() -> Self {
        Self {
            scale: 1.0,
            map: Default::default(),
        }
    }
}

impl HashGrid {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }
}
