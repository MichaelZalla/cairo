use cairo::{
    physics::simulation::{force::PointForce, state_vector::StateVector},
    vec::vec3::Vec3,
};

use crate::{simulation::COMPONENTS_PER_PARTICLE, springy_mesh::SpringyMesh};

pub fn system_dynamics_function(
    current_state: &StateVector,
    forces: &[PointForce],
    meshes: &mut [SpringyMesh],
    current_time: f32,
) -> StateVector {
    let n = current_state.len();

    // Compute new accelerations for derivative.
    let mut derivative = compute_accelerations(current_state, forces, meshes, current_time);

    for i in 0..n {
        // Copy velocities from previous (current?) state.
        derivative.data[i] = current_state.data[i + n];
    }

    derivative
}

fn compute_accelerations(
    current_state: &StateVector,
    forces: &[PointForce],
    meshes: &mut [SpringyMesh],
    current_time: f32,
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

        // Write the final environmental acceleration into derivative.

        derivative.data[i + n] = net_force_acceleration;
    }

    for mesh in meshes {
        // Compute forces acting on the mesh (spring, damper, drag, and lift).

        for strut in &mut mesh.struts {
            strut.compute_accelerations(current_state, &mut derivative, mesh.state_index_offset, n);
        }
    }

    derivative
}

pub fn integrate_euler(
    current_state: &StateVector,
    derivative: &StateVector,
    h: f32,
) -> StateVector {
    // Performs basic Euler integration over position and velocity.

    let n = current_state.len();

    let mut result = current_state.clone() + derivative * h;

    for i in 0..n {
        let midpoint_velocity = (current_state.data[i + n] + result.data[i + n]) / 2.0;

        result.data[i] = current_state.data[i] + midpoint_velocity * h;
    }

    result
}
