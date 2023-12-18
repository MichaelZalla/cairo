use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, Div, Mul, Sub},
};

use crate::vec::{vec2::Vec2, vec3::Vec3};

#[derive(Debug, Default, Copy, Clone)]
pub struct DefaultVertexIn {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
    pub c: Vec3,
}

impl DefaultVertexIn {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn interpolate(start: Self, end: Self, alpha: f32) -> Self {
        return start + (end - start) * alpha;
    }
}

impl Add<DefaultVertexIn> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn add(self, rhs: Self) -> DefaultVertexIn {
        DefaultVertexIn {
            p: self.p + rhs.p,
            n: self.n + rhs.n,
            uv: self.uv + rhs.uv,
            c: self.c + rhs.c,
        }
    }
}

impl Sub<DefaultVertexIn> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn sub(self, rhs: Self) -> DefaultVertexIn {
        DefaultVertexIn {
            p: self.p - rhs.p,
            n: self.n - rhs.n,
            uv: self.uv - rhs.uv,
            c: self.c - rhs.c,
        }
    }
}

impl Mul<f32> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn mul(self, scalar: f32) -> DefaultVertexIn {
        DefaultVertexIn {
            p: self.p * scalar,
            n: self.n * scalar,
            uv: self.uv * scalar,
            c: self.c * scalar,
        }
    }
}

impl Div<f32> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn div(self, scalar: f32) -> DefaultVertexIn {
        DefaultVertexIn {
            p: self.p / scalar,
            n: self.n / scalar,
            uv: self.uv / scalar,
            c: self.c / scalar,
        }
    }
}

impl Display for DefaultVertexIn {
    fn fmt(&self, v: &mut Formatter<'_>) -> Result {
        write!(v, "{}", self.p)
    }
}
