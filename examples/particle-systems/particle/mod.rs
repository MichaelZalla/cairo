use cairo::vec::vec3::Vec3;

use crate::{
    simulation::{FromStateVector, ToStateVector},
    state_vector::StateVector,
};

pub mod generator;
pub mod particlelist;

pub static PARTICLE_MASS: f32 = 50_000_000_000_000.0;
pub static PARTICLE_MAX_AGE_SECONDS: f32 = 60.0;

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub alive: bool,
    pub age: f32,
    #[allow(unused)]
    pub mass: f32,
    pub position: Vec3,
    pub prev_position: Vec3,
    pub velocity: Vec3,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            alive: true,
            mass: 1.0,
            age: Default::default(),
            position: Default::default(),
            prev_position: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl ToStateVector for Particle {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize) {
        state.data[i] = self.position;
        state.data[i + n] = self.velocity;
    }
}

impl FromStateVector for Particle {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize) {
        self.velocity = state.data[i + n];
        self.prev_position = self.position;
        self.position = state.data[i];
    }
}

impl Particle {
    pub fn should_be_killed(&mut self, h: f32) -> bool {
        self.age += h;

        self.age > PARTICLE_MAX_AGE_SECONDS
    }
}
