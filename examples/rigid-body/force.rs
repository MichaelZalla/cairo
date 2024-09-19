use cairo::vec::vec3::Vec3;

use crate::state_vector::StateVector;

pub type Newtons = Vec3;

pub type Force = fn(&StateVector, i: usize, current_time: f32) -> Newtons;
