use std::cell::RefCell;

use cairo::{
    physics::simulation::{
        force::{ContactPoint, PointForce},
        particle::{
            generator::{ParticleGenerator, ParticleGeneratorKind},
            Particle,
        },
        physical_constants::EARTH_GRAVITY,
        state_vector::StateVector,
        units::Newtons,
    },
    random::sampler::RandomSampler,
    vec::vec3,
};

use crate::simulation::Simulation;

pub(crate) const SEED_SIZE: usize = 2048;

pub static PARTICLE_MAX_AGE_SECONDS: f32 = 10.0;

static GRAVITY_POINT_FORCE: PointForce =
    |_state: &StateVector, _i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        let newtons = -vec3::UP * EARTH_GRAVITY;

        (newtons, None)
    };

pub fn make_simulation(sampler: RefCell<RandomSampler<SEED_SIZE>>) -> Simulation<SEED_SIZE> {
    let mass = 50.0;

    // Define some particle generators.

    let prototype = Particle {
        mass,
        max_age: PARTICLE_MAX_AGE_SECONDS,
        ..Default::default()
    };

    let omnidirectional = ParticleGenerator::new(
        ParticleGeneratorKind::Omnidirectional(vec3::UP),
        prototype,
        100.0,
        None,
        mass,
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
