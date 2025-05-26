use std::ops;

use crate::{
    matrix::Mat4,
    physics::simulation::{
        force::{DynForce, Force},
        physical_constants::EARTH_GRAVITY_ACCELERATION,
    },
    transform::quaternion::Quaternion,
    vec::{vec3::Vec3, vec4::Vec4},
};

use super::{RigidBodyKind};

pub type RigidBodyForce = Force<RigidBodySimulationState>;
pub type DynRigidBodyForce = DynForce<RigidBodySimulationState>;

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBodySimulationState {
    pub kind: RigidBodyKind,
    pub inverse_mass: f32,
    pub inverse_moment_of_inertia: Mat4,
    pub position: Vec3,
    pub orientation: Quaternion,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
}

impl ops::AddAssign for RigidBodySimulationState {
    fn add_assign(&mut self, rhs: Self) {
        self.position += rhs.position;
        self.orientation += rhs.orientation;
        self.linear_momentum += rhs.linear_momentum;
        self.angular_momentum += rhs.angular_momentum;
    }
}

impl ops::Add for RigidBodySimulationState {
    type Output = RigidBodySimulationState;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl ops::MulAssign for RigidBodySimulationState {
    fn mul_assign(&mut self, rhs: Self) {
        self.position *= rhs.position;
        self.orientation *= rhs.orientation;
        self.linear_momentum *= rhs.linear_momentum;
        self.angular_momentum *= rhs.angular_momentum;
    }
}

impl ops::Mul for RigidBodySimulationState {
    type Output = RigidBodySimulationState;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl ops::Mul<f32> for RigidBodySimulationState {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut result = self;

        result.position *= scalar;
        result.orientation *= scalar;
        result.linear_momentum *= scalar;
        result.angular_momentum *= scalar;

        result
    }
}

impl RigidBodySimulationState {
    pub fn velocity(&self) -> Vec3 {
        self.linear_momentum * self.inverse_mass
    }

    pub fn inverse_moment_of_inertia_world_space(&self) -> Mat4 {
        let r = *self.orientation.mat();

        r * self.inverse_moment_of_inertia * r.transposed()
    }

    pub fn angular_velocity(&self) -> Vec3 {
        let angular_momentum = Vec4::vector(self.angular_momentum);

        let inverse_moment_of_inertia_world_space = self.inverse_moment_of_inertia_world_space();

        (angular_momentum * inverse_moment_of_inertia_world_space).to_vec3()
    }

    pub fn angular_velocity_quaternion(&self) -> Quaternion {
        let angular_velocity = self.angular_velocity();

        let spin = Quaternion::from_raw(0.0, angular_velocity);

        // First-order integration (assumes that velocity is constant over the timestep).
        //
        // See: https://stackoverflow.com/a/46924782/1623811
        // See: https://www.ashwinnarayan.com/post/how-to-integrate-quaternions/

        self.orientation * 0.5 * spin
    }

    pub fn accumulate_accelerations(
        &self,
        forces: &[Box<DynRigidBodyForce>],
        current_time: f32,
        derivative: &mut Self,
    ) {
        let position = self.position;

        for force in forces {
            let (newtons, contact_point) = force(self, 0, current_time);

            // Accumulate linear momentum.

            let f = if newtons == EARTH_GRAVITY_ACCELERATION {
                newtons
            } else {
                newtons * self.inverse_mass
            };

            derivative.linear_momentum += f;

            // Accumulate angular momentum.

            if let Some(point) = contact_point {
                let r = point - position;
                let torque = -r.cross(f);

                derivative.angular_momentum += torque;
            }
        }
    }
}
