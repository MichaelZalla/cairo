use std::{cell::RefCell, rc::Rc};

use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use cairo::{random::sampler::RandomSampler, vec::vec3::Vec3};

use crate::{
    force::{Acceleration, Force},
    operator::{AdditiveAccelerationOperator, FunctionalAccelerationOperator, VelocityOperator},
    particle::{
        generator::{ParticleGenerator, ParticleGeneratorKind},
        particlelist::ParticleList,
        PARTICLE_MASS,
    },
    state_vector::StateVector,
};

static COMPONENTS_PER_PARTICLE: usize = 2;

pub(crate) trait ToStateVector {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize);
}

pub(crate) trait FromStateVector {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize);
}

fn system_dynamics_function(
    current_state: &StateVector,
    forces: &[&Force],
    operators: &mut Operators,
    current_time: f32,
    h: f32,
) -> StateVector {
    let n = current_state.len();

    // Compute new accelerations for derivative.
    let mut derivative = compute_accelerations(current_state, forces, operators, current_time, h);

    for i in 0..n {
        // Copy velocities from previous (current?) state.
        derivative.data[i] = current_state.data[i + n];
    }

    derivative
}

fn compute_accelerations(
    current_state: &StateVector,
    forces: &[&Force],
    operators: &mut Operators,
    current_time: f32,
    h: f32,
) -> StateVector {
    let n = current_state.len();

    let mut derivative = StateVector::new(COMPONENTS_PER_PARTICLE, n);

    // Compute environmental accelerations acting on each particle.
    for i in 0..n {
        let mut net_force_acceleration: Vec3 = Default::default();

        for force in forces {
            net_force_acceleration += force(current_state, i, current_time);
        }

        let mut net_force_acceleration_with_operators = net_force_acceleration;

        // Contribute any additive acceleration operators, in order.
        for operator in operators.additive_acceleration.iter_mut() {
            net_force_acceleration_with_operators +=
                operator(current_state, i, &net_force_acceleration_with_operators, h);
        }

        // Write the final environmental acceleration into derivative.
        derivative.data[i + n] = net_force_acceleration_with_operators;
    }

    // Compute interparticle acceleration acting on each particle.

    if n < 2 {
        return derivative;
    }

    for i in 0..n - 1 {
        for j in i + 1..n {
            let force = compute_interparticle_force(current_state, i, j, current_time);

            derivative.data[i + n] += force / PARTICLE_MASS;
            derivative.data[j + n] -= force / PARTICLE_MASS;
        }
    }

    derivative
}

fn compute_interparticle_force(
    current_state: &StateVector,
    i: usize,
    j: usize,
    _current_time: f32,
) -> Acceleration {
    // Basic interparticle gravitational force.
    let i_to_j = current_state.data[j] - current_state.data[i];

    let distance = i_to_j.mag();

    if distance < 1.0 {
        return Default::default();
    }

    let g_j = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32 * PARTICLE_MASS;

    let i_to_j_direction = i_to_j.as_normal();

    let particle_i_mass_over_distance_squared = PARTICLE_MASS / (distance * distance);

    let gravitational_force = i_to_j_direction * particle_i_mass_over_distance_squared * g_j;

    gravitational_force
}

fn integrate(
    current_state: &StateVector,
    derivative: &StateVector,
    operators: &mut Operators,
    h: f32,
) -> StateVector {
    let n = current_state.len();

    // Performs basic Euler integration over position and velocity.
    let mut result = current_state.clone() + derivative * h;

    for i in 0..n {
        // Applies any functional acceleration operators, in order.
        for operator in operators.functional_acceleration.iter_mut() {
            result.data[i + n] = operator(current_state, i, &result.data[i + n], h);
        }

        // Applies any velocity operators, in order.
        let mut midpoint_velocity = (current_state.data[i + n] + result.data[i + n]) / 2.0;

        for operator in operators.velocity.iter_mut() {
            midpoint_velocity = operator(current_state, i, &midpoint_velocity, h);
        }

        result.data[i] = current_state.data[i] + midpoint_velocity * h;
    }

    result
}

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

        let num_active_particles = pool.active();

        // @TODO Allocate vector memory using a longer-lived stack arena.
        let mut state = StateVector::new(COMPONENTS_PER_PARTICLE, num_active_particles);

        let n = state.len();

        let alive_indices: Vec<usize> = pool
            .iter()
            .enumerate()
            .filter(|(_i, p)| p.alive)
            .map(|(i, _p)| i)
            .collect();

        // Copy current positions and velocities into the current state.
        for (i, index) in alive_indices.iter().enumerate() {
            match pool.at(*index) {
                Some(particle) => {
                    particle.write_to(&mut state, n, i);
                }
                None => panic!(),
            }
        }

        let mut operators = self.operators.borrow_mut();

        let derivative =
            system_dynamics_function(&state, &self.forces, &mut operators, uptime_seconds, h);

        let new_state = integrate(&state, &derivative, &mut operators, h);

        // Copy new positions and velocities back into each particle.
        for (i, index) in alive_indices.iter().enumerate() {
            match pool.at_mut(*index) {
                Some(particle) => {
                    particle.write_from(&new_state, n, i);
                }
                None => panic!(),
            }
        }

        Ok(())
    }
}
