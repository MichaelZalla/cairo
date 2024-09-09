use std::ops;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct State {
    pub f0: f32,
    pub f1: f32,
}

pub(crate) type StateDerivative = State;

impl ops::AddAssign for State {
    fn add_assign(&mut self, rhs: Self) {
        self.f0 += rhs.f0;
        self.f1 += rhs.f1;
    }
}

impl ops::Add for State {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut cloned = self;
        cloned += rhs;
        cloned
    }
}

impl ops::MulAssign for State {
    fn mul_assign(&mut self, rhs: Self) {
        self.f0 += rhs.f0;
        self.f1 += rhs.f1;
    }
}

impl ops::Mul for State {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut cloned = self;
        cloned *= rhs;
        cloned
    }
}

impl ops::Mul<f32> for State {
    type Output = Self;
    fn mul(self, scale: f32) -> Self::Output {
        let mut cloned = self;
        cloned.f0 *= scale;
        cloned.f1 *= scale;
        cloned
    }
}
