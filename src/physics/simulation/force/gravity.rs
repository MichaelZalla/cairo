use crate::{
    physics::simulation::{
        physical_constants::EARTH_GRAVITY,
        rigid_body::rigid_body_simulation_state::RigidBodySimulationState,
        state_vector::StateVector, units::Newtons,
    },
    vec::vec3,
};

use super::{ContactPoint, Force, PointForce};

pub static GRAVITY_POINT_FORCE: PointForce =
    |_state: &StateVector, _i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        let newtons = -vec3::UP * EARTH_GRAVITY;

        (newtons, None)
    };

pub static GRAVITY_RIGID_BODY_FORCE: Force<RigidBodySimulationState> =
    |_state: &RigidBodySimulationState,
     _i: usize,
     _current_time: f32|
     -> (Newtons, Option<ContactPoint>) {
        let newtons = -vec3::UP * EARTH_GRAVITY;

        (newtons, None)
    };
