use std::{iter::zip, ops};

use cairo::physics::simulation::rigid_body::{
    rigid_body_simulation_state::RigidBodySimulationState, RigidBody,
};

#[derive(Default, Debug, Clone)]
pub struct StateVector<T = f32>(pub Vec<T>);

impl<T: Default + Clone> StateVector<T> {
    pub fn new(size: usize) -> Self {
        Self(vec![Default::default(); size])
    }
}

impl<T: Copy + ops::AddAssign> ops::AddAssign for StateVector<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in zip(self.0.iter_mut(), rhs.0.iter()) {
            *lhs += *rhs;
        }
    }
}

impl<T: Copy + ops::AddAssign> ops::Add for StateVector<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl<T: Copy + ops::MulAssign<T>> ops::MulAssign for StateVector<T> {
    fn mul_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in zip(self.0.iter_mut(), rhs.0.iter()) {
            *lhs *= *rhs;
        }
    }
}

impl<T: Copy + ops::MulAssign<T>> ops::Mul for StateVector<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl<T: Copy + ops::Mul<f32, Output = T>> ops::Mul<f32> for StateVector<T> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = self;

        for i in 0..result.0.len() {
            result.0[i] = result.0[i] * rhs;
        }

        result
    }
}

impl<T: Copy + ops::Mul<f32, Output = T>> ops::Mul<f32> for &StateVector<T> {
    type Output = StateVector<T>;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = self.clone();

        for i in 0..result.0.len() {
            result.0[i] = result.0[i] * rhs;
        }

        result
    }
}

impl From<&[RigidBody]> for StateVector<RigidBodySimulationState> {
    fn from(rigid_bodies: &[RigidBody]) -> Self {
        let n = rigid_bodies.len();
        let mut state = StateVector::<RigidBodySimulationState>::new(n);

        for (i, body) in rigid_bodies.iter().enumerate() {
            state.0[i] = body.into();
        }

        state
    }
}
