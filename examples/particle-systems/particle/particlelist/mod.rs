use std::slice::{Iter, IterMut};

use super::Particle;

static MAX_PARTICLES: usize = 8192;

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
        let mut pool = Vec::<Particle>::with_capacity(MAX_PARTICLES);

        let mut inactive_stack = Vec::<usize>::with_capacity(MAX_PARTICLES);

        for i in 0..MAX_PARTICLES {
            pool.push(Particle {
                alive: false,
                ..Default::default()
            });

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

    pub fn at(&self, index: usize) -> Option<&Particle> {
        if index < self.pool.len() {
            Some(&self.pool[index])
        } else {
            None
        }
    }

    pub fn at_mut(&mut self, index: usize) -> Option<&mut Particle> {
        if index < self.pool.len() {
            Some(&mut self.pool[index])
        } else {
            None
        }
    }

    pub fn reset_inactive_stack(&mut self) {
        self.inactive_stack = (0..MAX_PARTICLES).collect();

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

    #[allow(unused)]
    /// Deactivates all particles.
    pub fn clear(&mut self) {
        for particle in &mut self.pool {
            particle.alive = false;
        }

        self.reset_inactive_stack();
    }
}
