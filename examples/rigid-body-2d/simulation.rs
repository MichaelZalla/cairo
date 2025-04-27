use cairo::{
    physics::simulation::rigid_body::rigid_body_simulation_state::{
        RigidBodyForce, RigidBodySimulationState,
    },
    vec::vec3::Vec3,
};

use crate::{rigid_body::CircleRigidBody, state_vector::StateVector};

pub struct Simulation {
    pub forces: Vec<RigidBodyForce>,
    pub rigid_bodies: Vec<CircleRigidBody>,
}

impl Simulation {
    pub fn tick(&mut self, current_time: f32, h: f32, _cursor_world_space: Vec3) {
        let n = self.rigid_bodies.len();

        let mut state = StateVector::<RigidBodySimulationState>::new(n);

        for (i, circle) in self.rigid_bodies.iter().enumerate() {
            state.0[i] = circle.rigid_body.state();
        }

        let new_state = {
            let derivative = system_dynamics_function(&state, &self.forces, current_time);

            // Performs basic Euler integration over position and velocity.

            state + derivative * h
        };

        // @NOTE `RigidBody` is responsible for re-normalizing its quaternion
        // components as part of its call to `Transform::set_translation_and_orientation(…)`.

        for (i, circle) in self.rigid_bodies.iter_mut().enumerate() {
            circle.rigid_body.apply_simulation_state(&new_state.0[i]);
        }
    }
}

fn system_dynamics_function(
    state: &StateVector<RigidBodySimulationState>,
    forces: &[RigidBodyForce],
    current_time: f32,
) -> StateVector<RigidBodySimulationState> {
    let n = state.0.len();

    let mut derivative = StateVector::<RigidBodySimulationState>::new(n);

    for i in 0..n {
        let body_state = &state.0[i];
        let body_derivative = &mut derivative.0[i];

        // 1. Rate-of-change of position (velocity).

        // Derive from the body's current linear momentum.

        body_derivative.position = body_state.velocity();

        // 2. Rate-of-change of orientation (angular velocity).

        body_derivative.orientation = body_state.angular_velocity();

        // 3. Rate-of-change of linear and angular momenta.

        body_state.accumulate_accelerations(forces, current_time, body_derivative);
    }

    derivative
}
