use crate::{
    physics::simulation::{
        physical_constants::EARTH_GRAVITY, state_vector::StateVector, units::Newtons,
    },
    vec::vec3,
};

use super::{ContactPoint, PointForce};

pub static GRAVITY_POINT_FORCE: PointForce =
    |_state: &StateVector, _i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        let newtons = -vec3::UP * EARTH_GRAVITY;

        (newtons, None)
    };
