use std::slice::{Iter, IterMut};

use crate::force::Force;

use super::Particle;

static MAX_PARTICLES: usize = 4096;

#[derive(Debug, Clone)]
pub struct ParticleList {
    /// Stores all particles, including their current alive-or-dead state.
    pool: Vec<Particle>,
    /// An array of indices into `pool`; a stack of all inactive cells.
    inactive_stack: Vec<usize>,
    // Current number of unused cells in the pool.
    inactive_count: usize,
}

impl Default for ParticleList {
    fn default() -> Self {
        let mut pool: Vec<Particle> = Default::default();

        pool.reserve(MAX_PARTICLES);

        let mut inactive_stack: Vec<usize> = Default::default();

        inactive_stack.reserve(MAX_PARTICLES);

        for i in 0..MAX_PARTICLES {
            let mut particle = Particle::default();

            particle.alive = false;

            pool.push(particle);

            inactive_stack.push(i);
        }

        Self {
            pool,
            inactive_stack,
            inactive_count: MAX_PARTICLES,
        }
    }
}

impl ParticleList {
    #[allow(unused)]
    pub fn active(&self) -> usize {
        self.pool.capacity() - self.inactive_count
    }

    #[allow(unused)]
    pub fn inactive(&self) -> usize {
        self.inactive_count
    }

    pub fn iter(&self) -> Iter<'_, Particle> {
        self.pool.iter()
    }

    #[allow(unused)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Particle> {
        self.pool.iter_mut()
    }

    #[allow(unused)]
    pub fn reset_inactive_stack(&mut self) {
        self.inactive_stack.clear();

        for i in 0..MAX_PARTICLES {
            self.inactive_stack.push(i);
        }

        self.inactive_count = MAX_PARTICLES;
    }

    /// Claims and activate a new particle in the pool, based on a description.
    pub fn activate(&mut self, particle: Particle) -> Result<(), String> {
        match self.inactive_stack.pop() {
            Some(index) => {
                self.pool[index] = Particle {
                    alive: true,
                    ..Default::default()
                };

                self.inactive_count -= 1;

                self.pool[index] = particle;

                Ok(())
            }
            None => Err("No pool memory.".to_string()),
        }
    }

    /// Asks each particle if it should still be alive; if not, deactivates it.
    pub fn test_and_deactivate(&mut self, h: f32) {
        for (index, particle) in self.pool.iter_mut().enumerate() {
            if particle.alive && particle.should_be_killed(h) {
                particle.alive = false;

                self.inactive_stack.push(index);
                self.inactive_count += 1;
            }
        }
    }

    /// Computes and stores a new acceleration for each particle.
    pub fn compute_accelerations(&mut self, forces: &[&Force]) {
        for particle in self.pool.iter_mut() {
            particle.compute_acceleration(forces);
        }
    }

    /// Performs numerical integration to update each active (alive) particle.
    pub fn integrate(&mut self, h: f32) {
        for particle in self.pool.iter_mut() {
            particle.integrate(h);
        }
    }

    #[allow(unused)]
    /// Deactivates all particles.
    pub fn clear(&mut self) {
        for particle in &mut self.pool {
            particle.alive = false;
        }

        self.reset_inactive_stack();
    }
}
