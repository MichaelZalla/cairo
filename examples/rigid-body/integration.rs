use cairo::physics::simulation::rigid_body::rigid_body_simulation_state::{
    DynRigidBodyForce, RigidBodySimulationState,
};

use crate::state_vector::StateVector;

pub fn system_dynamics_function(
    state: &StateVector<RigidBodySimulationState>,
    forces: &[Box<DynRigidBodyForce>],
    current_time: f32,
) -> StateVector<RigidBodySimulationState> {
    let n = state.0.len();

    let mut derivative = StateVector::<RigidBodySimulationState>::new(n);

    for i in 0..n {
        let body_state = &state.0[i];
        let body_derivative = &mut derivative.0[i];

        // 1. Rate-of-change of position (velocity).

        // Derive velocity from the body's current linear momentum.

        body_derivative.position = body_state.velocity();

        // 2. Derive angular velocity from current orientation and angular momentum.

        body_derivative.orientation = body_state.angular_velocity_quaternion();

        // 3. Rate-of-change of linear and angular momenta.

        body_state.accumulate_accelerations(forces, current_time, body_derivative);
    }

    derivative
}
