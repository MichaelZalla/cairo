use std::collections::HashMap;

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

impl From<Vec3> for GridSpaceCoordinate {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x.floor() as isize,
            y: value.y.floor() as isize,
            z: value.z.floor() as isize,
        }
    }
}

impl From<&RigidBodySimulationState> for GridSpaceCoordinate {
    fn from(state: &RigidBodySimulationState) -> Self {
        Self::from(state.position)
    }
}

pub type HashGrid = HashMap<GridSpaceCoordinate, Vec<usize>>;
