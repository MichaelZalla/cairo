use cairo::{
    physics::simulation::units::{Acceleration, Velocity},
    vec::vec3::Vec3,
};

use crate::state_vector::StateVector;

pub(crate) trait AdditiveAccelerationOperator:
    FnMut(&StateVector, usize, &Acceleration, f32) -> Acceleration
{
}

impl<T: FnMut(&StateVector, usize, &Vec3, f32) -> Acceleration> AdditiveAccelerationOperator for T {}

pub(crate) trait FunctionalAccelerationOperator:
    FnMut(&StateVector, usize, &Velocity, f32) -> Velocity
{
}

impl<T: FnMut(&StateVector, usize, &Velocity, f32) -> Velocity> FunctionalAccelerationOperator
    for T
{
}

pub(crate) trait VelocityOperator:
    FnMut(&StateVector, usize, &Velocity, f32) -> Velocity
{
}

impl<T: FnMut(&StateVector, usize, &Velocity, f32) -> Velocity> VelocityOperator for T {}
