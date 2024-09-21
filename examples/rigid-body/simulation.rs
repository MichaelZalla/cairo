use cairo::vec::{vec3::Vec3, vec4::Vec4};

use crate::{
    force::Force, quaternion::Quaternion, rigid_body::RigidBody,
    rigid_body_simulation_state::RigidBodySimulationState, state_vector::StateVector,
};

pub struct Simulation {
    pub forces: Vec<Force>,
    pub rigid_bodies: Vec<RigidBody>,
}

impl Simulation {
    pub fn tick(&mut self, current_time: f32, h: f32, _cursor_world_space: Vec3) {
        let n = self.rigid_bodies.len();

        let mut state = StateVector::<RigidBodySimulationState>::new(n);

        for (i, body) in self.rigid_bodies.iter().enumerate() {
            state.0[i] = body.state();
        }

        let new_state = {
            let derivative = system_dynamics_function(&state, &self.forces, current_time);

            // Performs basic Euler integration over position and velocity.

            state + derivative * h
        };

        // @NOTE `RigidBody` is responsible for re-normalizing its quaternion
        // components as part of its call to `Transform::set_translation_and_orientation(…)`.

        for (i, body) in self.rigid_bodies.iter_mut().enumerate() {
            body.apply_simulation_state(&new_state.0[i]);
        }
    }
}

fn system_dynamics_function(
    state: &StateVector<RigidBodySimulationState>,
    forces: &[Force],
    current_time: f32,
) -> StateVector<RigidBodySimulationState> {
    let n = state.0.len();

    let mut derivative = StateVector::<RigidBodySimulationState>::new(n);

    for i in 0..n {
        let body_state = &state.0[i];
        let body_derivative = &mut derivative.0[i];

        // 1. Rate-of-change of position (velocity).

        // Derive from the body's current linear momentum.

        body_derivative.position = body_state.linear_momentum * body_state.inverse_mass;

        // 2. Rate-of-change of orientation (angular velocity).

        body_derivative.orientation = {
            let orientation = body_state.orientation;

            let angular_momentum = Vec4::new(body_state.angular_momentum, 0.0);

            let inverse_moment_of_intertia_world_space = {
                let r = *orientation.mat();

                r * body_state.inverse_moment_of_interia * r.transposed()
            };

            let angular_velocity =
                (angular_momentum * inverse_moment_of_intertia_world_space).to_vec3();

            let spin = Quaternion::from_raw(0.0, angular_velocity);

            // First-order integration (assumes that velocity is constant over the timestep).
            //
            // See: https://stackoverflow.com/a/46924782/1623811
            // See: https://www.ashwinnarayan.com/post/how-to-integrate-quaternions/

            let roc = (orientation * 0.5) * spin;

            roc
        };

        // 3. Rate-of-change of linear and angular momenta.

        let position = body_state.position;

        for force in forces {
            let (f, point) = force(body_state, current_time);

            // Accumulate linear momentum.

            body_derivative.linear_momentum += f * body_state.inverse_mass;

            // Accumulate angular momentum.

            if let Some(point) = point {
                let r = point - position;
                let torque = -r.cross(f);

                body_derivative.angular_momentum += torque;
            }
        }
    }

    derivative
}
