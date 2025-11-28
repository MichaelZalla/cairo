use std::{iter::zip, ops};

use crate::vec::vec3::Vec3;

/// Generic state vector for physics simulations.
///
/// This is a newtype wrapper around `Vec<T>` that provides mathematical operations
/// commonly needed for numerical integration (addition, scalar multiplication, etc.).
#[derive(Default, Debug, Clone)]
pub struct StateVector<T = Vec3>(pub Vec<T>);

pub trait ToStateVector {
    fn write_to(&self, state: &mut StateVector<Vec3>, n: usize, i: usize);
}

pub trait FromStateVector {
    fn write_from(&mut self, state: &StateVector<Vec3>, n: usize, i: usize);
}

impl<T: Default + Clone> StateVector<T> {
    pub fn new(size: usize) -> Self {
        Self(vec![Default::default(); size])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of entities represented in this state vector.
    ///
    /// `components_per_entity` specifies how many T elements each entity uses.
    /// For example, a particle system with position and velocity would use 2 Vec3s per particle.
    pub fn num_entities(&self, components_per_entity: usize) -> usize {
        self.len() / components_per_entity
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
