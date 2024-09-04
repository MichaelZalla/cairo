use cairo::vec::vec3::Vec3;

use crate::{force::Acceleration, state_vector::StateVector};

pub(crate) trait AdditiveAccelerationOperator:
    FnMut(&StateVector, usize, &Vec3, f32) -> Acceleration
{
}

impl<T: FnMut(&StateVector, usize, &Vec3, f32) -> Acceleration> AdditiveAccelerationOperator for T {}

pub(crate) trait FunctionalAccelerationOperator:
    FnMut(&StateVector, usize, &Vec3, f32) -> Vec3
{
}

impl<T: FnMut(&StateVector, usize, &Vec3, f32) -> Vec3> FunctionalAccelerationOperator for T {}

pub(crate) trait VelocityOperator: FnMut(&StateVector, usize, &Vec3, f32) -> Vec3 {}

impl<T: FnMut(&StateVector, usize, &Vec3, f32) -> Vec3> VelocityOperator for T {}
