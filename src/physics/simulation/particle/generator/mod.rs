use crate::{
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler, VectorDisplaceSampler},
    vec::vec3::Vec3,
};

use super::{Particle, particlelist::ParticleList};

#[derive(Debug, Copy, Clone)]
pub enum ParticleGeneratorKind {
    // Omnidirectional(origin)
    // Emits in all directions, with equal likelihood, from an origin.
    Omnidirectional(Vec3),

    // Directed(origin, direction) Emits from an original
    // towards a preferred direction, with directional variance controlled by a
    // maximum angular offset.
    Directed(Vec3, Vec3),
}

impl Default for ParticleGeneratorKind {
    fn default() -> Self {
        Self::Omnidirectional(Vec3::default())
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ParticleGenerator {
    pub kind: ParticleGeneratorKind,
    prototype: Particle,
    particles_per_second: f32,
    fractional_particles_accumulator: f32,
    max_deflection_angle_radians: Option<f32>,
    mass: f32,
    speed_mean: f32,
    speed_range: f32,
}

impl ParticleGenerator {
    pub fn new(
        kind: ParticleGeneratorKind,
        prototype: Particle,
        particles_per_second: f32,
        max_deflection_angle_radians: Option<f32>,
        mass: f32,
        speed_mean: f32,
        speed_range: f32,
    ) -> Self {
        Self {
            kind,
            prototype,
            particles_per_second,
            fractional_particles_accumulator: 0.0,
            max_deflection_angle_radians,
            mass,
            speed_mean,
            speed_range,
        }
    }

    pub fn generate<const N: usize>(
        &mut self,
        list: &mut ParticleList<N>,
        sampler: &mut RandomSampler<N>,
        h: f32,
    ) -> Result<(), String> {
        // Compute the number of (fractional) particles to generate, based on `h`.
        let mut num_particles_to_generate_fractional = h * self.particles_per_second;

        // Cash out any particles we've accumulated from past calls to `generate()`.
        if self.fractional_particles_accumulator > 1.0 {
            let whole_particles_accumulated = self.fractional_particles_accumulator.floor();

            num_particles_to_generate_fractional += whole_particles_accumulated;

            self.fractional_particles_accumulator -= whole_particles_accumulated;
        }

        // Save any remaining fractional particle for future calls.

        self.fractional_particles_accumulator += num_particles_to_generate_fractional.fract();

        // Drop the fractional bit for this generation.

        let num_particles_to_generate = num_particles_to_generate_fractional.floor() as usize;

        for _ in 0..num_particles_to_generate {
            // 1. Determine an initial position for our particle.

            let mut position = match self.kind {
                ParticleGeneratorKind::Omnidirectional(origin) => origin,
                ParticleGeneratorKind::Directed(origin, _) => origin,
            };

            let direction = match self.kind {
                ParticleGeneratorKind::Omnidirectional(_) => sampler.sample_direction_uniform(),
                ParticleGeneratorKind::Directed(_, direction) => {
                    let angle = self.max_deflection_angle_radians.unwrap();

                    match sampler.sample_displacement_normal(&direction, angle) {
                        Ok(displaced) => displaced,
                        Err(err) => panic!("{}", err),
                    }
                }
            };

            let speed = sampler.sample_range_normal(self.speed_mean, self.speed_range / 3.0);

            let velocity = direction * speed;

            // Apply a random position offset (along direction).

            position += velocity * h * sampler.sample_range_uniform(0.0, 1.0);

            list.activate(Particle {
                position,
                velocity,
                ..self.prototype
            })?;
        }

        Ok(())
    }
}
