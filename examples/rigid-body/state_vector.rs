use std::{iter::zip, ops};

#[derive(Default, Debug, Clone)]
pub struct StateVector(pub Vec<f32>);

impl StateVector {
    pub fn new(size: usize) -> Self {
        Self(vec![0.0; size])
    }
}

impl ops::AddAssign for StateVector {
    fn add_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in zip(self.0.iter_mut(), rhs.0.iter()) {
            *lhs += *rhs;
        }
    }
}

impl ops::Add for StateVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut cloned = self.clone();
        cloned += rhs;
        cloned
    }
}

impl ops::MulAssign<f32> for StateVector {
    fn mul_assign(&mut self, scalar: f32) {
        for value in self.0.iter_mut() {
            *value *= scalar;
        }
    }
}

impl ops::Mul<f32> for StateVector {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut cloned = self.clone();
        cloned *= scalar;
        cloned
    }
}

impl ops::Mul<f32> for &StateVector {
    type Output = StateVector;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut cloned = self.clone();
        cloned *= scalar;
        cloned
    }
}

pub trait ToStateVector {
    fn write_to(&self, state: &mut [f32]);
}

pub trait FromStateVector {
    fn write_from(&mut self, state: &[f32]);
}
