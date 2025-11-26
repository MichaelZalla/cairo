use std::cell::RefCell;

use cairo::{
    physics::simulation::{
        force::gravity::GRAVITY_POINT_FORCE,
        particle::{
            Particle,
            generator::{ParticleGenerator, ParticleGeneratorKind},
        },
    },
    random::sampler::RandomSampler,
    vec::vec3,
};

use crate::simulation::Simulation;

pub static PARTICLE_MASS: f32 = 50.0;

pub(crate) const SEED_SIZE: usize = 2048;

pub static PARTICLE_MAX_AGE_SECONDS: f32 = 10.0;

pub fn make_simulation(sampler: RefCell<RandomSampler<SEED_SIZE>>) -> Simulation<SEED_SIZE> {
    // Define some particle generators.

    let prototype: Particle = Particle {
        mass: PARTICLE_MASS,
        max_age: PARTICLE_MAX_AGE_SECONDS,
        ..Default::default()
    };

    let omnidirectional = ParticleGenerator::new(
        ParticleGeneratorKind::Omnidirectional(vec3::UP),
        prototype,
        100.0,
        None,
        PARTICLE_MASS,
        5.0,
        2.0,
    );

    Simulation {
        sampler,
        pool: Default::default(),
        forces: vec![GRAVITY_POINT_FORCE],
        generators: RefCell::new(vec![omnidirectional]),
    }
}
