use cairo::vec::vec3::Vec3;

use crate::{
    force::Force,
    rigid_body::{RigidBody, COEFFICIENT_COUNT},
    state_vector::{FromStateVector, StateVector, ToStateVector},
};

pub struct Simulation<'a> {
    pub forces: Vec<&'a Force>,
    pub rigid_bodies: Vec<RigidBody>,
}

impl<'a> Simulation<'a> {
    pub fn tick(&mut self, current_time: f32, h: f32, _cursor_world_space: Vec3) {
        let n = self.rigid_bodies.len();
        let size = n * COEFFICIENT_COUNT;

        let mut state = StateVector::new(size);

        for (i, body) in self.rigid_bodies.iter().enumerate() {
            let start = i * COEFFICIENT_COUNT;
            let end = start + COEFFICIENT_COUNT - 1;

            let slice = &mut state.0[start..=end];

            body.write_to(slice);
        }

        let new_state = {
            let derivative = system_dynamics_function(&state, &self.forces, current_time);

            forward_euler(&state, &derivative, h)
        };

        for (i, body) in self.rigid_bodies.iter_mut().enumerate() {
            let start = i * COEFFICIENT_COUNT;
            let end = start + COEFFICIENT_COUNT - 1;

            let slice = &new_state.0[start..=end];

            body.write_from(slice);
        }
    }
}

fn system_dynamics_function(
    current_state: &StateVector,
    forces: &[&Force],
    current_time: f32,
) -> StateVector {
    let n = current_state.0.len() / COEFFICIENT_COUNT;

    // Compute new accelerations (i.e., derivative) from net forces.
    let mut derivative = compute_accelerations(current_state, forces, current_time);

    // Copy
    for i in 0..n {
        // Copy first derivatives (velocity, etc) from the prior state.

        let start = i * COEFFICIENT_COUNT;

        let one_over_mass = current_state.0[start];

        // Derive a velocity from the linear momentum.

        derivative.0[start + 1] = current_state.0[start + 8] * one_over_mass;
        derivative.0[start + 2] = current_state.0[start + 9] * one_over_mass;
        derivative.0[start + 3] = current_state.0[start + 10] * one_over_mass;
    }

    derivative
}

fn compute_accelerations(
    current_state: &StateVector,
    forces: &[&Force],
    current_time: f32,
) -> StateVector {
    let size = current_state.0.len();
    let n = size / COEFFICIENT_COUNT;

    let mut derivative = StateVector::new(size);

    // Compute net force and torque acting on each rigid body.

    // For each point, compute net environmental force acting on it.
    for i in 0..n {
        let mut net_force_acceleration: Vec3 = Default::default();

        for force in forces {
            net_force_acceleration += force(current_state, i, current_time);
        }

        // Write the final net environmental acceleration.
        let start = i * COEFFICIENT_COUNT;

        // Linear momentum

        derivative.0[start + 8] = net_force_acceleration.x;
        derivative.0[start + 9] = net_force_acceleration.y;
        derivative.0[start + 10] = net_force_acceleration.z;
    }

    derivative
}

fn forward_euler(current_state: &StateVector, derivative: &StateVector, h: f32) -> StateVector {
    // Performs basic Euler integration over position and velocity.

    current_state.clone() + derivative * h
}