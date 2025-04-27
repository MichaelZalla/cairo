use rigid_body_simulation_state::RigidBodySimulationState;

use crate::{
    matrix::Mat4,
    transform::Transform3D,
    vec::{vec3::Vec3, vec4::Vec4},
};

pub mod rigid_body_simulation_state;

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBody {
    pub transform: Transform3D,
    pub mass: f32,
    pub moment_of_inertia: Mat4,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    // Derived state
    inverse_mass: f32,
    inverse_moment_of_inertia: Mat4,
    velocity: Vec3,
    angular_velocity: Vec3,
}

impl RigidBody {
    pub fn new(
        mass: f32,
        transform: Transform3D,
        moment_of_inertia: Mat4,
        inverse_moment_of_inertia: Mat4,
    ) -> Self {
        let mut result = Self {
            mass,
            inverse_mass: 1.0 / mass,
            transform,
            moment_of_inertia,
            inverse_moment_of_inertia,
            ..Default::default()
        };

        result.recompute_derived_state();

        result
    }

    pub fn state(&self) -> RigidBodySimulationState {
        RigidBodySimulationState {
            inverse_mass: self.inverse_mass,
            inverse_moment_of_interia: self.inverse_moment_of_inertia,
            position: *self.transform.translation(),
            orientation: *self.transform.rotation(),
            linear_momentum: self.linear_momentum,
            angular_momentum: self.angular_momentum,
        }
    }

    pub fn apply_simulation_state(&mut self, state: &RigidBodySimulationState) {
        let (translation, mut orientation) = (state.position, state.orientation);

        self.transform.set_translation(translation);

        orientation.renormalize();

        self.transform.set_rotation(orientation);

        self.linear_momentum = state.linear_momentum;

        self.angular_momentum = state.angular_momentum;
    }

    fn recompute_derived_state(&mut self) {
        self.velocity = self.linear_momentum * self.inverse_mass;

        self.angular_velocity =
            (Vec4::vector(self.angular_momentum) * self.inverse_moment_of_inertia).to_vec3();
    }
}
