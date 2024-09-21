use std::ops;

use cairo::{matrix::Mat4, vec::vec3::Vec3};

use crate::quaternion::Quaternion;

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
