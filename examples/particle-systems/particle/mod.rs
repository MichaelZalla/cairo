use cairo::vec::vec3::Vec3;

use crate::{
    force::{Force, Newtons},
    simulation::Operators,
};

pub mod generator;
pub mod particlelist;

pub static PARTICLE_MAX_AGE_SECONDS: f32 = 2.0;

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub alive: bool,
    pub age: f32,
    pub mass: f32,
    pub position: Vec3,
    pub prev_position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
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
            acceleration: Default::default(),
        }
    }
}

impl Particle {
    pub fn should_be_killed(&mut self, h: f32) -> bool {
        self.age += h;

        self.age > PARTICLE_MAX_AGE_SECONDS
    }

    pub fn compute_acceleration(&mut self, forces: &[&Force], operators: &mut Operators, h: f32) {
        let mut total_force_newtons: Newtons = Default::default();

        for force in forces {
            total_force_newtons += force(&self);
        }

        self.acceleration = total_force_newtons / self.mass;

        for operator in operators.additive_acceleration.iter_mut() {
            self.acceleration += operator(&self, h);
        }
    }

    pub fn integrate(&mut self, operators: &mut Operators, h: f32) {
        let mut new_velocity = self.velocity + self.acceleration * h;

        for operator in operators.functional_acceleration.iter_mut() {
            new_velocity = operator(&self, &new_velocity, h);
        }

        self.prev_position = self.position;

        self.position = self.position + (self.velocity + new_velocity) / 2.0 * h;

        self.velocity = new_velocity;
    }
}
