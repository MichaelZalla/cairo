use cairo::vec::vec3::Vec3;

use crate::particle::Particle;

type Acceleration = Vec3;

pub(crate) trait AdditiveAccelerationOperator:
    FnMut(&Particle, &Vec3, f32) -> Acceleration
{
}

impl<T: FnMut(&Particle, &Vec3, f32) -> Acceleration> AdditiveAccelerationOperator for T {}

pub(crate) trait FunctionalAccelerationOperator:
    FnMut(&Particle, &Vec3, f32) -> Vec3
{
}

impl<T: FnMut(&Particle, &Vec3, f32) -> Vec3> FunctionalAccelerationOperator for T {}

pub(crate) trait VelocityOperator: FnMut(&Particle, &Vec3, f32) -> Vec3 {}

impl<T: FnMut(&Particle, &Vec3, f32) -> Vec3> VelocityOperator for T {}
