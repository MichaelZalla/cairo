use cairo::{
    physics::simulation::{
        force::Force,
        state_vector::{FromStateVector, StateVector, ToStateVector},
    },
    vec::vec3::Vec3,
};

use crate::{
    springy_mesh::SpringyMesh,
    static_line_segment_collider::{Collider, StaticLineSegmentCollider},
};

pub type PointForce = Force<StateVector>;

pub struct Simulation<'a> {
    pub forces: Vec<&'a PointForce>,
    pub wind: Vec3,
    pub static_colliders: Vec<StaticLineSegmentCollider>,
    pub meshes: Vec<SpringyMesh>,
}

impl<'a> Simulation<'a> {
    pub fn tick(&mut self, current_time: f32, h: f32) {
        let n = self.meshes.iter().map(|m| m.points.len()).sum();

        let mut state = StateVector::new(2, n);

        let mut point_index = 0;

        for mesh in self.meshes.iter_mut() {
            mesh.state_index_offset = point_index;

            for point in &mesh.points {
                point.write_to(&mut state, n, point_index);

                point_index += 1;
            }
        }

        let derivative = self.system_dynamics_function(&state, current_time, h);

        let mut new_state = self.integrate(&state, &derivative, h);

        // Detect and resolve collisions against all static colliders.

        for i in 0..n {
            let position = state.data[i];

            let mut new_position = new_state.data[i];
            let mut new_velocity = new_state.data[i + n];

            // We'll break early on the first collision (if any).

            for collider in &self.static_colliders {
                // Check if this particle has just crossed over the  plane.

                if let Some((_f, new_distance)) = collider.test(&position, &new_position) {
                    // Perform an approximate collision resolution.

                    collider.resolve_approximate(
                        &mut new_position,
                        &mut new_velocity,
                        new_distance,
                    );

                    new_state.data[i + n] = new_velocity;
                    new_state.data[i] = new_position;

                    break;
                }
            }
        }

        let mut point_index = 0;

        for mesh in self.meshes.iter_mut() {
            mesh.state_index_offset = point_index;

            for point in mesh.points.iter_mut() {
                point.write_from(&new_state, n, point_index);

                point_index += 1;
            }
        }
    }

    fn system_dynamics_function(
        &mut self,
        current_state: &StateVector,
        current_time: f32,
        h: f32,
    ) -> StateVector {
        let n = current_state.len();

        // Compute new accelerations (i.e., derivative).
        let mut derivative = self.compute_accelerations(current_state, current_time, h);

        for i in 0..n {
            // Copy velocities from previous (current?) state.
            derivative.data[i] = current_state.data[i + n];
        }

        derivative
    }

    fn compute_accelerations(
        &mut self,
        current_state: &StateVector,
        current_time: f32,
        _h: f32,
    ) -> StateVector {
        let n = current_state.len();

        let mut derivative = StateVector::new(2, n);

        // For each point, compute net environmental force acting on it.
        for i in 0..n {
            let mut net_force_acceleration: Vec3 = Default::default();

            for force in &self.forces {
                let (newtons, _contact_point) = force(current_state, i, current_time);

                net_force_acceleration += newtons;
            }

            // Write the final net environmental acceleration.
            derivative.data[i + n] = net_force_acceleration;
        }

        for mesh in self.meshes.iter_mut() {
            // Compute forces acting on the mesh (spring, damper, drag, and lift).
            for strut in mesh.struts.iter_mut() {
                strut.compute_accelerations(
                    current_state,
                    &mut derivative,
                    mesh.state_index_offset,
                    n,
                    &self.wind,
                );
            }

            // Compute torque needed to maintain the resting angles for each vertex.
            for face in mesh.faces.iter() {
                mesh.compute_torsional_accelerations(face, &mut derivative, n);
            }
        }

        derivative
    }

    fn integrate(
        &self,
        current_state: &StateVector,
        derivative: &StateVector,
        h: f32,
    ) -> StateVector {
        // Performs basic Euler integration over position and velocity.

        current_state.clone() + derivative * h
    }
}
