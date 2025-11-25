use cairo::{
    physics::simulation::rigid_body::{
        rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
        RigidBody,
    },
    transform::quaternion::Quaternion,
    vec::{vec3::Vec3, vec4::Vec4},
};

use crate::state_vector::StateVector;

pub struct Simulation {
    pub forces: Vec<Box<DynRigidBodyForce>>,
    pub rigid_bodies: Vec<RigidBody>,
}

impl Simulation {
    pub fn tick(&mut self, current_time: f32, h: f32, _cursor_world_space: Vec3) {
        let n = self.rigid_bodies.len();

        let mut state = StateVector::<RigidBodySimulationState>::new(n);

        for (i, circle) in self.rigid_bodies.iter().enumerate() {
            state.0[i] = circle.into();
        }

        let derivative = system_dynamics_function(&state, &self.forces, h, current_time);

        // Semi-implicit Euler integration: update momenta from forces,
        // then update position and orientation from the new momenta.

        let mut new_state = state.clone();

        for (body, derivative) in new_state.0.iter_mut().zip(derivative.0.iter()) {
            // Update linear momentum from forces.
            body.linear_momentum += derivative.linear_momentum * h;

            // Update angular momentum from torques.
            body.angular_momentum += derivative.angular_momentum * h;

            // Update position from new linear momentum (semi-implicit step).
            body.position += body.linear_momentum * body.inverse_mass * h;

            // Compute angular velocity from new angular momentum.
            let new_angular_spin = {
                let angular_momentum = Vec4::vector(body.angular_momentum);

                let inverse_moment_of_inertia_world_space =
                    body.inverse_moment_of_inertia_world_space();

                let angular_velocity =
                    (angular_momentum * inverse_moment_of_inertia_world_space).to_vec3();

                Quaternion::from_raw(0.0, angular_velocity)
            };

            // Compute orientation derivative from angular velocity.
            let orientation_derivative = body.orientation * 0.5 * new_angular_spin;

            // Update orientation.
            body.orientation += orientation_derivative * h;
        }

        // @NOTE `RigidBody` is responsible for re-normalizing its quaternion
        // components as part of its call to `Transform::set_translation_and_orientation(…)`.

        for (i, circle) in self.rigid_bodies.iter_mut().enumerate() {
            circle.apply_simulation_state(&new_state.0[i]);
        }
    }
}

fn system_dynamics_function(
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

        // 1. Rate-of-change of position (velocity).

        // Derive from the body's current linear momentum.

        body_derivative.position = body_state.velocity();

        // 2. Rate-of-change of orientation (angular velocity).

        body_derivative.orientation = body_state.angular_velocity_quaternion();

        // 3. Rate-of-change of linear and angular momenta.

        body_state.accumulate_accelerations(forces, body_derivative, h, current_time);
    }

    derivative
}
