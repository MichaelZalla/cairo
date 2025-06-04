use crate::physics::simulation::{
    physical_constants::EARTH_GRAVITY_ACCELERATION,
    rigid_body::rigid_body_simulation_state::RigidBodySimulationState, state_vector::StateVector,
    units::Newtons,
};

use super::{ContactPoint, Force, PointForce};

pub static GRAVITY_POINT_FORCE: PointForce =
    |_state: &StateVector,
     _i: usize,
     _current_time: f32|
     -> (Newtons, Option<ContactPoint>, bool) { (EARTH_GRAVITY_ACCELERATION, None, true) };

pub static GRAVITY_RIGID_BODY_FORCE: Force<RigidBodySimulationState> =
    |_state: &RigidBodySimulationState,
     _i: usize,
     _current_time: f32|
     -> (Newtons, Option<ContactPoint>, bool) { (EARTH_GRAVITY_ACCELERATION, None, true) };
