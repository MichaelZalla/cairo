use std::cell::RefCell;

use cairo::{random::sampler::RandomSampler, vec::vec3::Vec3};

use crate::{
    force::Force,
    particle::{
        generator::{ParticleGenerator, ParticleGeneratorKind},
        particlelist::ParticleList,
    },
};

pub(crate) struct Simulation<'a> {
    pub sampler: RefCell<RandomSampler<1024>>,
    pub pool: RefCell<ParticleList>,
    pub forces: Vec<&'a Force>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
}

impl<'a> Simulation<'a> {
    pub fn tick(
        &self,
        h: f32,
        uptime_seconds: f32,
        cursor_world_space: &Vec3,
    ) -> Result<(), String> {
        let mut sampler = self.sampler.borrow_mut();
        let mut pool = self.pool.borrow_mut();
        let mut generators = self.generators.borrow_mut();

        for generator in generators.iter_mut() {
            match generator.kind {
                ParticleGeneratorKind::Omnidirectional(ref mut origin) => {
                    *origin = Vec3 {
                        y: 30.0 + 20.0 * (uptime_seconds * 3.0).sin(),
                        x: origin.x,
                        z: origin.z,
                    }
                }
                ParticleGeneratorKind::Directed(origin, ref mut direction) => {
                    *direction = (*cursor_world_space - origin).as_normal();
                }
            }

            generator.generate(&mut pool, &mut sampler, h)?;
        }

        pool.test_and_deactivate(h);

        pool.compute_accelerations(&self.forces);

        pool.integrate(h);

        Ok(())
    }
}
