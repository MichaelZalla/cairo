use cairo::physics::simulation::particle::Particle;

use crate::state_vector::{FromStateVector, StateVector, ToStateVector};

pub static PARTICLE_MASS: f32 = 10_000_000_000_000.0;

pub static PARTICLE_MAX_AGE_SECONDS: f32 = 8.0;

pub static MAX_PARTICLE_SIZE_PIXELS: u32 = 8;

impl ToStateVector for Particle {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize) {
        state.data[i] = self.position;
        state.data[i + n] = self.velocity;
    }
}

impl FromStateVector for Particle {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize) {
        self.position = state.data[i];
        self.velocity = state.data[i + n];
    }
}
