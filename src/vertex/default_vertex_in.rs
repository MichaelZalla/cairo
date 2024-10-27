use std::ops::{Add, Div, Mul, Sub};

use crate::{
    animation::lerp,
    vec::{vec2::Vec2, vec3::Vec3},
};

#[derive(Debug, Default, Copy, Clone)]
pub struct DefaultVertexIn {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: Vec2,
    pub color: Vec3,
}

impl DefaultVertexIn {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn interpolate(start: Self, end: Self, alpha: f32) -> Self {
        lerp(start, end, alpha)
    }
}

impl Add<DefaultVertexIn> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn add(self, rhs: Self) -> DefaultVertexIn {
        DefaultVertexIn {
            position: self.position + rhs.position,
            normal: self.normal + rhs.normal,
            tangent: self.tangent + rhs.tangent,
            bitangent: self.bitangent + rhs.bitangent,
            uv: self.uv + rhs.uv,
            color: self.color + rhs.color,
        }
    }
}

impl Sub<DefaultVertexIn> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn sub(self, rhs: Self) -> DefaultVertexIn {
        DefaultVertexIn {
            position: self.position - rhs.position,
            normal: self.normal - rhs.normal,
            tangent: self.tangent - rhs.tangent,
            bitangent: self.bitangent - rhs.bitangent,
            uv: self.uv - rhs.uv,
            color: self.color - rhs.color,
        }
    }
}

impl Mul<f32> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn mul(self, scalar: f32) -> DefaultVertexIn {
        DefaultVertexIn {
            position: self.position * scalar,
            normal: self.normal * scalar,
            tangent: self.tangent * scalar,
            bitangent: self.bitangent * scalar,
            uv: self.uv * scalar,
            color: self.color * scalar,
        }
    }
}

impl Div<f32> for DefaultVertexIn {
    type Output = DefaultVertexIn;
    fn div(self, scalar: f32) -> DefaultVertexIn {
        DefaultVertexIn {
            position: self.position / scalar,
            normal: self.normal / scalar,
            tangent: self.tangent / scalar,
            bitangent: self.bitangent / scalar,
            uv: self.uv / scalar,
            color: self.color / scalar,
        }
    }
}
