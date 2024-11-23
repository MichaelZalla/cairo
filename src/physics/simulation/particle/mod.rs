use core::f32;

use crate::vec::vec3::Vec3;

use super::units::Velocity;

pub mod generator;
pub mod particlelist;

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub alive: bool,
    pub age: f32,
    pub max_age: f32,
    pub mass: f32,
    pub position: Vec3,
    pub velocity: Velocity,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            alive: true,
            mass: 1.0,
            age: 0.0,
            max_age: f32::MAX,
            position: Vec3::default(),
            velocity: Velocity::default(),
        }
    }
}

impl Particle {
    pub fn should_be_killed(&mut self, h: f32) -> bool {
        self.age += h;
        self.age > self.max_age
    }
}
