use cairo::physics::simulation::{
    contact::StaticContactList,
    rigid_body::rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
};

use crate::state_vector::StateVector;

pub fn system_dynamics_function(
    state: &StateVector<RigidBodySimulationState>,
    forces: &[Box<DynRigidBodyForce>],
    predicted_contacts: &[StaticContactList<6>],
    h: f32,
    current_time: f32,
) -> StateVector<RigidBodySimulationState> {
    let n = state.0.len();

    let mut derivative = StateVector::<RigidBodySimulationState>::new(n);

    for i in 0..n {
        let body_state = &state.0[i];
        let body_derivative = &mut derivative.0[i];
        let body_predicted_contacts = &predicted_contacts[i];

        // Compute rate-of-change of linear and angular momenta.

        body_state.accumulate_accelerations(
            forces,
            body_predicted_contacts,
            body_derivative,
            h,
            current_time,
        );
    }

    derivative
}
