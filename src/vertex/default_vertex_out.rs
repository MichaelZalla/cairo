use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Mul, MulAssign, Sub},
};

use crate::vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4};

#[derive(Copy, Clone, Default)]
pub struct DefaultVertexOut {
    pub position: Vec4,
    pub normal: Vec4,
    pub color: Vec3,
    pub uv: Vec2,
    pub world_pos: Vec3,
    pub depth: f32,
}

impl DefaultVertexOut {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn interpolate(start: Self, end: Self, alpha: f32) -> Self {
        return start + (end - start) * alpha;
    }
}

impl Add<DefaultVertexOut> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn add(self, rhs: Self) -> DefaultVertexOut {
        DefaultVertexOut {
            position: self.position + rhs.position,
            normal: self.normal + rhs.normal,
            color: self.color + rhs.color,
            uv: self.uv + rhs.uv,
            world_pos: self.world_pos + rhs.world_pos,
            depth: 1.0,
        }
    }
}

impl AddAssign<DefaultVertexOut> for DefaultVertexOut {
    fn add_assign(&mut self, rhs: DefaultVertexOut) {
        self.position += rhs.position;
        self.normal += rhs.normal;
        self.color += rhs.color;
        self.uv += rhs.uv;
        self.world_pos += rhs.world_pos;
        self.depth += rhs.depth;
    }
}

impl Sub<DefaultVertexOut> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn sub(self, rhs: Self) -> DefaultVertexOut {
        DefaultVertexOut {
            position: self.position - rhs.position,
            normal: self.normal - rhs.normal,
            color: self.color - rhs.color,
            uv: self.uv - rhs.uv,
            world_pos: self.world_pos - rhs.world_pos,
            depth: self.depth - rhs.depth,
        }
    }
}

impl Mul<f32> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn mul(self, scalar: f32) -> DefaultVertexOut {
        DefaultVertexOut {
            position: self.position * scalar,
            normal: self.normal * scalar,
            color: self.color * scalar,
            uv: self.uv * scalar,
            world_pos: self.world_pos * scalar,
            depth: self.depth * scalar,
        }
    }
}

impl MulAssign<f32> for DefaultVertexOut {
    fn mul_assign(&mut self, scalar: f32) {
        self.position *= scalar;
        self.normal *= scalar;
        self.color *= scalar;
        self.uv *= scalar;
        self.world_pos *= scalar;
        self.depth *= scalar;
    }
}

impl Div<f32> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn div(self, scalar: f32) -> DefaultVertexOut {
        DefaultVertexOut {
            position: self.position / scalar,
            normal: self.normal / scalar,
            color: self.color / scalar,
            uv: self.uv / scalar,
            world_pos: self.world_pos / scalar,
            depth: self.depth / scalar,
        }
    }
}

impl Display for DefaultVertexOut {
    fn fmt(&self, v: &mut Formatter<'_>) -> Result {
        write!(v, "{}", self.position)
    }
}
