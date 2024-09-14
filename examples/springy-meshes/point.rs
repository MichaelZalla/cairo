use cairo::vec::vec3::Vec3;

use crate::state_vector::{FromStateVector, StateVector, ToStateVector};

#[derive(Default, Debug, Copy, Clone)]
pub struct Point {
    pub position: Vec3,
    pub velocity: Vec3,
}

pub static POINT_MASS: f32 = 2.5;

impl ToStateVector for Point {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize) {
        state.data[i] = self.position;
        state.data[i + n] = self.velocity;
    }
}

impl FromStateVector for Point {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize) {
        self.velocity = state.data[i + n];
        self.position = state.data[i];
    }
}
