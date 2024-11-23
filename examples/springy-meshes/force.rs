use cairo::physics::simulation::units::Newtons;

use crate::state_vector::StateVector;

pub type Force = fn(&StateVector, i: usize, current_time: f32) -> Newtons;
