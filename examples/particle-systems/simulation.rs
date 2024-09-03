use std::{cell::RefCell, rc::Rc};

use cairo::{random::sampler::RandomSampler, vec::vec3::Vec3};

use crate::{
    force::Force,
    operator::{AdditiveAccelerationOperator, FunctionalAccelerationOperator, VelocityOperator},
    particle::{
        generator::{ParticleGenerator, ParticleGeneratorKind},
        particlelist::ParticleList,
    },
};

#[derive(Default)]
pub(crate) struct Operators {
    // pub initialization: Vec<fn(&mut Particle)>,
    pub additive_acceleration: Vec<Box<dyn AdditiveAccelerationOperator>>,
    pub functional_acceleration: Vec<Box<dyn FunctionalAccelerationOperator>>,
    pub velocity: Vec<Box<dyn VelocityOperator>>,
}

pub(crate) struct Simulation<'a, const N: usize> {
    pub sampler: Rc<RefCell<RandomSampler<N>>>,
    pub pool: RefCell<ParticleList>,
    pub forces: Vec<&'a Force>,
    pub operators: RefCell<Operators>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
}

impl<'a, const N: usize> Simulation<'a, N> {
    pub fn tick(
        &self,
        h: f32,
        uptime_seconds: f32,
        cursor_world_space: &Vec3,
    ) -> Result<(), String> {
        let mut pool = self.pool.borrow_mut();
        let mut generators = self.generators.borrow_mut();

        {
            let mut sampler = self.sampler.borrow_mut();

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
        }

        pool.test_and_deactivate(h);

        let mut operators = self.operators.borrow_mut();

        pool.compute_accelerations(&self.forces, &mut operators, h);

        pool.integrate(&mut operators, h);

        Ok(())
    }
}
