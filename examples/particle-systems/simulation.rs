use std::{cell::RefCell, rc::Rc};

use cairo::{
    geometry::primitives::line_segment::LineSegment,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::resolve_plane_collision_approximate,
            force::PointForce,
            particle::{
                generator::{ParticleGenerator, ParticleGeneratorKind},
                particlelist::ParticleList,
            },
            state_vector::{FromStateVector, StateVector, ToStateVector},
        },
    },
    random::sampler::RandomSampler,
    resource::handle::Handle,
    scene::resources::SceneResources,
    vec::vec3::Vec3,
};

use crate::{
    integrate::{integrate_euler, system_dynamics_function},
    intersect::intersect_line_segment_bvh,
};

pub const COMPONENTS_PER_PARTICLE: usize = 2;

pub struct Simulation<const N: usize> {
    pub sampler: RefCell<RandomSampler<N>>,
    pub resources: Rc<SceneResources>,
    pub static_mesh_handle: Handle,
    pub pool: RefCell<ParticleList<N>>,
    pub generators: RefCell<Vec<ParticleGenerator>>,
    pub forces: Vec<PointForce>,
}

impl<const N: usize> Simulation<N> {
    pub fn tick(&self, h: f32, uptime_seconds: f32) -> Result<(), String> {
        let mut pool = self.pool.borrow_mut();

        let mut generators = self.generators.borrow_mut();

        {
            let mut sampler = self.sampler.borrow_mut();

            for generator in generators.iter_mut() {
                if let ParticleGeneratorKind::Omnidirectional(ref mut origin) = generator.kind {
                    let uptime_scaled = uptime_seconds / 2.0;

                    *origin = Vec3 {
                        x: 12.0 * uptime_scaled.sin(),
                        z: 12.0 * uptime_scaled.cos(),
                        y: 6.0,
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

        let physics_material = PhysicsMaterial {
            dynamic_friction: 0.15,
            restitution: 0.25,
        };

        let mesh_arena = self.resources.mesh.borrow();

        if let Ok(entry) = mesh_arena.get(&self.static_mesh_handle) {
            let mesh = &entry.item;

            if let Some(collider) = mesh.collider.as_ref() {
                for i in 0..num_active_particles {
                    let start_position = state.data[i];

                    let mut end_position = new_state.data[i];

                    let mut segment = LineSegment::new(start_position, end_position);

                    intersect_line_segment_bvh(&mut segment, collider);

                    if let Some(tri_index) = &segment.colliding_primitive {
                        let mut end_velocity = new_state.data[num_active_particles + i];

                        let triangle = &collider.tris[*tri_index];

                        let mag = segment.mag();

                        if mag < 0.02 {
                            new_state.data[i] = state.data[i];
                            new_state.data[i + n] = state.data[i + n];
                        } else {
                            let plane_normal = triangle.plane.normal;
                            let penetration_depth = mag * (1.0 - segment.t);

                            resolve_plane_collision_approximate(
                                plane_normal,
                                &physics_material,
                                &mut end_position,
                                &mut end_velocity,
                                penetration_depth,
                            );

                            new_state.data[i] = end_position;
                            new_state.data[i + n] = end_velocity;
                        }
                    }
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
