use std::{
    iter::zip,
    ops::{Add, AddAssign, Mul, MulAssign},
};

use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Clone)]
pub struct StateVector {
    size: usize,
    pub data: Vec<Vec3>,
}

pub trait ToStateVector {
    fn write_to(&self, state: &mut StateVector, n: usize, i: usize);
}

pub trait FromStateVector {
    fn write_from(&mut self, state: &StateVector, n: usize, i: usize);
}

impl AddAssign for StateVector {
    fn add_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in zip(self.data.iter_mut(), rhs.data.iter()) {
            *lhs += *rhs;
        }
    }
}

impl Add for StateVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut cloned = self.clone();
        cloned += rhs;
        cloned
    }
}

impl MulAssign<f32> for StateVector {
    fn mul_assign(&mut self, scalar: f32) {
        for value in self.data.iter_mut() {
            *value *= scalar;
        }
    }
}

impl Mul<f32> for StateVector {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut cloned = self.clone();
        cloned *= scalar;
        cloned
    }
}

impl Mul<f32> for &StateVector {
    type Output = StateVector;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut cloned = self.clone();
        cloned *= scalar;
        cloned
    }
}

impl StateVector {
    pub fn new(components: usize, size: usize) -> Self {
        Self {
            size,
            data: vec![Default::default(); components * size],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
