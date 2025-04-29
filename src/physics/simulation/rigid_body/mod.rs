use std::f32::consts::PI;

use rigid_body_simulation_state::RigidBodySimulationState;

use crate::{matrix::Mat4, transform::Transform3D, vec::vec3::Vec3};

pub mod rigid_body_simulation_state;

#[derive(Debug, Copy, Clone)]
pub enum RigidBodyKind {
    Circle(f32),
    Sphere(f32),
}

impl Default for RigidBodyKind {
    fn default() -> Self {
        Self::Sphere(0.5)
    }
}

impl RigidBodyKind {
    pub fn get_moment_of_intertia(&self, mass: f32) -> (Mat4, Mat4) {
        match self {
            RigidBodyKind::Circle(radius) => {
                let scale = (PI * radius.powi(4)) / 2.0;

                let moment_of_inertia = Mat4::scale_uniform(scale);

                let inverse_moment_of_inertia = {
                    let inverse_scale = 1.0 / scale;

                    Mat4::scale_uniform(inverse_scale)
                };

                (moment_of_inertia, inverse_moment_of_inertia)
            }
            RigidBodyKind::Sphere(radius) => {
                let scale = (2.0 / 5.0) * mass * radius * radius;

                let moment_of_inertia = Mat4::scale_uniform(scale);

                let inverse_moment_of_inertia = {
                    let inverse_scale = 1.0 / scale;

                    Mat4::scale_uniform(inverse_scale)
                };

                (moment_of_inertia, inverse_moment_of_inertia)
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBody {
    pub kind: RigidBodyKind,
    pub transform: Transform3D,
    pub mass: f32,
    pub moment_of_inertia: Mat4,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    pub did_collide: bool,
    // Derived state
    inverse_mass: f32,
    inverse_moment_of_inertia: Mat4,
}

impl RigidBody {
    pub fn new(kind: RigidBodyKind, mass: f32, position: Vec3) -> Self {
        let inverse_mass = 1.0 / mass;

        let (moment_of_inertia, inverse_moment_of_inertia) = kind.get_moment_of_intertia(mass);

        let transform = {
            let mut transform = Transform3D::default();

            transform.set_translation(position);

            transform
        };

        Self {
            kind,
            mass,
            inverse_mass,
            transform,
            moment_of_inertia,
            inverse_moment_of_inertia,
            ..Default::default()
        }
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
}
