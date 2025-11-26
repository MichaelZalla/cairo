use std::cell::RefCell;

use cairo::{
    geometry::{
        accelerator::static_triangle_tlas::StaticTriangleTLAS, primitives::capsule::Capsule,
    },
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::resolve_point_plane_collision_approximate,
            force::PointForce,
            particle::{
                generator::{ParticleGenerator, ParticleGeneratorKind},
                particlelist::ParticleList,
            },
            state_vector::{FromStateVector, StateVector, ToStateVector},
        },
    },
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

use crate::{
    PARTICLE_MESH_RADIUS,
    integrate::{integrate_euler, system_dynamics_function},
    intersect::intersect_medial_primitive_tlas,
};

pub const COMPONENTS_PER_PARTICLE: usize = 2;

pub struct Simulation<const N: usize> {
    pub sampler: RefCell<RandomSampler<N>>,
    pub pool: RefCell<ParticleList<N>>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
    pub forces: Vec<PointForce>,
}

impl<const N: usize> Simulation<N> {
    pub fn tick(
        &self,
        h: f32,
        uptime_seconds: f32,
        tlas: &StaticTriangleTLAS,
    ) -> Result<(), String> {
        let mut pool = self.pool.borrow_mut();

        let mut generators = self.generators.borrow_mut();

        {
            let mut sampler = self.sampler.borrow_mut();

            for generator in generators.iter_mut() {
                if let ParticleGeneratorKind::Omnidirectional(ref mut origin) = generator.kind {
                    let uptime_scaled = uptime_seconds / 2.0;

                    *origin = Vec3 {
                        x: 10.0 * uptime_scaled.sin(),
                        z: 10.0 * uptime_scaled.cos(),
                        y: 15.0,
                    }
                }

                generator.generate(&mut pool, &mut sampler, h)?;
            }
        }

        pool.test_and_deactivate(h);

        let num_active_particles = pool.active();

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

        // Compute the derivative and integrate over the time delta.

        let derivative = system_dynamics_function(&state, &self.forces, uptime_seconds);

        let mut new_state = integrate_euler(&state, &derivative, h);

        // Resolve collisions.

        static PHYSICS_MATERIAL: PhysicsMaterial = PhysicsMaterial {
            static_friction: 0.0,
            dynamic_friction: 0.15,
            restitution: 0.25,
        };

        for i in 0..num_active_particles {
            let acceleration = derivative.data[i + n];

            let start_position = state.data[i];

            let mut end_position = new_state.data[i];

            let mut capsule = Capsule::new(start_position, end_position, PARTICLE_MESH_RADIUS);

            intersect_medial_primitive_tlas(&mut capsule, tlas);

            if let (Some(bvh_instance_index), Some(tri_index)) = (
                capsule.segment.colliding_bvh_index,
                capsule.segment.colliding_primitive,
            ) {
                let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

                let triangle = &bvh_instance.bvh.tris[tri_index];

                let triangle_normal = triangle.plane.normal;

                let instance_triangle_normal =
                    (triangle_normal * bvh_instance.transform).as_normal();

                let mut end_velocity = new_state.data[num_active_particles + i];

                // Freeze particles with very low velocity to prevent instability
                if capsule.segment.transformed_length < 0.02 {
                    new_state.data[i] = state.data[i];
                    new_state.data[i + n] = state.data[i + n];
                } else {
                    let time_before_collision = h * capsule.segment.t;
                    let time_after_collision = h - time_before_collision;

                    let accumulated_velocity = acceleration * time_after_collision;

                    end_velocity -= accumulated_velocity;

                    // Only process valid collisions (t must be in [0, 1] range)
                    if capsule.segment.t >= 0.0 && capsule.segment.t <= 1.0 {
                        // Compute collision point and penetration in world space
                        let collision_point = capsule.segment.start
                            + (capsule.segment.end - capsule.segment.start) * capsule.segment.t;

                        let penetration_vector = end_position - collision_point;

                        let penetration_depth =
                            penetration_vector.dot(instance_triangle_normal).abs();

                        // Skip negligible penetrations to prevent bias-induced energy gain
                        if penetration_depth > 0.01 {
                            resolve_point_plane_collision_approximate(
                                instance_triangle_normal,
                                &PHYSICS_MATERIAL,
                                &mut end_position,
                                &mut end_velocity,
                                penetration_depth,
                            );
                        }
                    }

                    new_state.data[i] = end_position;
                    new_state.data[i + n] = end_velocity;
                }
            }
        }

        // Copy new positions and velocities back into each particle.

        for (i, index) in alive_indices.iter().enumerate() {
            match pool.at_mut(*index) {
                Some(particle) => {
                    particle.write_from(&new_state, n, i);

                    if particle.position.y < -5.0 {
                        particle.age = f32::MAX;
                    }
                }
                None => panic!(),
            }
        }

        Ok(())
    }
}
