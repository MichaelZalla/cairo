use std::cell::RefCell;

use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use cairo::{
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::resolve_plane_collision_approximate,
            force::PointForce,
            operator::Operators,
            particle::{
                generator::{ParticleGenerator, ParticleGeneratorKind},
                particlelist::ParticleList,
            },
            state_vector::{FromStateVector, StateVector, ToStateVector},
            units::Acceleration,
        },
    },
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

use crate::{quadtree::Quadtree, static_line_segment_collider::StaticLineSegmentCollider};

static COMPONENTS_PER_PARTICLE: usize = 2;

fn system_dynamics_function(
    current_state: &StateVector,
    quadtree: &Quadtree,
    forces: &[&PointForce],
    operators: &mut Operators,
    current_time: f32,
    h: f32,
) -> StateVector {
    let n = current_state.len();

    // Compute new accelerations for derivative.
    let mut derivative =
        compute_accelerations(current_state, quadtree, forces, operators, current_time, h);

    for i in 0..n {
        // Copy velocities from previous (current?) state.
        derivative.data[i] = current_state.data[i + n];
    }

    derivative
}

fn compute_accelerations(
    current_state: &StateVector,
    quadtree: &Quadtree,
    forces: &[&PointForce],
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
            let (newtons, _contact_point) = force(current_state, i, current_time);

            net_force_acceleration += newtons;
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

    if let Some(ptr) = quadtree.root {
        let root = unsafe { ptr.as_ref() };

        // Compute interparticle acceleration acting on each particle.

        for i in 0..n {
            let position = &current_state.data[i];

            derivative.data[i + n] += root.acceleration(position);
        }
    }

    derivative
}

pub fn universal_gravity_acceleration(
    attractor_position: &Vec3,
    attractor_mass: f32,
    attractee_position: &Vec3,
) -> Acceleration {
    let attractee_to_attractor = *attractor_position - *attractee_position;

    let distance = attractee_to_attractor.mag();

    if distance < 1.0 {
        return Default::default();
    }

    let g_a = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32 * attractor_mass;

    (attractee_to_attractor / distance.powi(3)) * g_a
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

pub(crate) struct Simulation<'a, const N: usize> {
    pub sampler: RefCell<RandomSampler<N>>,
    pub pool: RefCell<ParticleList<N>>,
    pub forces: Vec<&'a PointForce>,
    pub static_colliders: RefCell<Vec<StaticLineSegmentCollider>>,
    pub operators: RefCell<Operators>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
    pub quadtree: RefCell<Quadtree>,
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
        let mut quadtree = self.quadtree.borrow_mut();

        {
            let mut sampler = self.sampler.borrow_mut();

            for generator in generators.iter_mut() {
                match generator.kind {
                    ParticleGeneratorKind::Omnidirectional(ref mut origin) => {
                        *origin = Vec3 {
                            y: 40.0 + 20.0 * (uptime_seconds * 3.0).sin(),
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

        let positions = &state.data[0..n];

        let mut new_quadtree = Quadtree::new(Vec3::extent(positions));

        // Build out the tree!

        match new_quadtree.root.as_mut() {
            Some(link) => unsafe {
                let root = link.as_mut();

                for (i, index) in alive_indices.iter().enumerate() {
                    match pool.at(*index) {
                        Some(particle) => {
                            let mass = particle.mass;
                            let position = state.data[i];

                            root.insert(position, mass);
                        }
                        None => panic!(),
                    }
                }
            },
            None => panic!("Something is very wrong"),
        }

        *quadtree = new_quadtree;

        let mut operators = self.operators.borrow_mut();

        let derivative = system_dynamics_function(
            &state,
            &quadtree,
            &self.forces,
            &mut operators,
            uptime_seconds,
            h,
        );

        let mut new_state = integrate(&state, &derivative, &mut operators, h);

        // Detect and resolve particle collisions for all colliders.

        static PHYSICS_MATERIAL: PhysicsMaterial = PhysicsMaterial {
            dynamic_friction: 0.15,
            restitution: 0.9,
        };

        for i in 0..n {
            let position = state.data[i];

            let mut end_position = new_state.data[i];
            let mut end_velocity = new_state.data[i + n];

            // We'll break early on the first collision (if any).

            let colliders = self.static_colliders.borrow();

            for collider in colliders.iter() {
                // Check if this particle has just crossed over the  plane.

                if let Some((_f, new_distance)) = collider.test(&position, &end_position) {
                    // Perform an approximate collision resolution.

                    resolve_plane_collision_approximate(
                        &collider.plane,
                        &PHYSICS_MATERIAL,
                        &mut end_position,
                        &mut end_velocity,
                        new_distance,
                    );

                    new_state.data[i + n] = end_velocity;
                    new_state.data[i] = end_position;

                    break;
                }
            }
        }

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
