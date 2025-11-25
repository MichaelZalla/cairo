use cairo::physics::simulation::rigid_body::rigid_body_simulation_state::{
    DynRigidBodyForce, RigidBodySimulationState,
};

use crate::state_vector::StateVector;

pub fn system_dynamics_function(
    state: &StateVector<RigidBodySimulationState>,
    forces: &[Box<DynRigidBodyForce>],
    h: f32,
    current_time: f32,
) -> StateVector<RigidBodySimulationState> {
    let n = state.0.len();

    let mut derivative = StateVector::<RigidBodySimulationState>::new(n);

    for i in 0..n {
        let body_state = &state.0[i];
        let body_derivative = &mut derivative.0[i];

        // Compute rate-of-change of linear and angular momenta.

        body_state.accumulate_accelerations(forces, body_derivative, h, current_time);
    }

    derivative
}
