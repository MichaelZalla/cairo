use super::{
    state_vector::StateVector,
    units::{Acceleration, Velocity},
};

pub trait AdditiveAccelerationOperator:
    FnMut(&StateVector, usize, &Acceleration, f32) -> Acceleration
{
}

impl<T: FnMut(&StateVector, usize, &Acceleration, f32) -> Acceleration> AdditiveAccelerationOperator
    for T
{
}

pub trait FunctionalAccelerationOperator:
    FnMut(&StateVector, usize, &Velocity, f32) -> Velocity
{
}

impl<T: FnMut(&StateVector, usize, &Velocity, f32) -> Velocity> FunctionalAccelerationOperator
    for T
{
}

pub trait VelocityOperator: FnMut(&StateVector, usize, &Velocity, f32) -> Velocity {}

impl<T: FnMut(&StateVector, usize, &Velocity, f32) -> Velocity> VelocityOperator for T {}

#[derive(Default)]
pub struct Operators {
    pub additive_acceleration: Vec<Box<dyn AdditiveAccelerationOperator>>,
    pub functional_acceleration: Vec<Box<dyn FunctionalAccelerationOperator>>,
    pub velocity: Vec<Box<dyn VelocityOperator>>,
}
