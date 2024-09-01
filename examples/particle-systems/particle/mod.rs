use cairo::vec::vec3::Vec3;

use crate::force::Force;

pub mod generator;
pub mod particlelist;

pub static PARTICLE_MAX_AGE_SECONDS: f32 = 5.0;

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

    pub fn compute_acceleration(&mut self, forces: &[&Force]) {
        let mut total_acceleration_from_forces: Vec3 = Default::default();

        for force in forces {
            total_acceleration_from_forces += force(&self);
        }

        self.acceleration = total_acceleration_from_forces;
    }

    pub fn integrate(&mut self, h: f32) {
        let new_velocity = self.velocity + self.acceleration * h;

        self.prev_position = self.position;

        self.position = self.position + (self.velocity + new_velocity) / 2.0 * h;

        self.velocity = new_velocity;
    }
}
