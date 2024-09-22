use std::ops;

use cairo::{
    matrix::Mat4,
    transform::quaternion::Quaternion,
    vec::{vec3::Vec3, vec4::Vec4},
};

use crate::force::Force;

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBodySimulationState {
    pub inverse_mass: f32,
    pub inverse_moment_of_interia: Mat4,
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

    pub fn angular_velocity(&self) -> Quaternion {
        let orientation = self.orientation;

        let angular_momentum = Vec4::new(self.angular_momentum, 0.0);

        let inverse_moment_of_intertia_world_space = {
            let r = *orientation.mat();

            r * self.inverse_moment_of_interia * r.transposed()
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
    }

    pub fn accumulate_accelerations(
        &self,
        forces: &[Force],
        current_time: f32,
        derivative: &mut Self,
    ) {
        let position = self.position;

        for force in forces {
            let (f, point) = force(self, current_time);

            // Accumulate linear momentum.

            derivative.linear_momentum += f * self.inverse_mass;

            // Accumulate angular momentum.

            if let Some(point) = point {
                let r = point - position;
                let torque = -r.cross(f);

                derivative.angular_momentum += torque;
            }
        }
    }
}
