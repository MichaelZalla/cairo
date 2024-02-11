use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Mul, MulAssign, Sub},
};

use crate::{
    matrix::Mat4,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct TangentSpaceInfo {
    pub tbn: Mat4,
    pub tbn_inverse: Mat4,
    pub normal: Vec3,
    pub point_light_position: Vec3,
    pub view_position: Vec3,
    pub fragment_position: Vec3,
}

impl Add<TangentSpaceInfo> for TangentSpaceInfo {
    type Output = TangentSpaceInfo;
    fn add(self, rhs: Self) -> TangentSpaceInfo {
        TangentSpaceInfo {
            tbn: self.tbn + rhs.tbn,
            tbn_inverse: self.tbn_inverse + rhs.tbn_inverse,
            normal: self.normal + rhs.normal,
            point_light_position: self.point_light_position + rhs.point_light_position,
            view_position: self.view_position + rhs.view_position,
            fragment_position: self.fragment_position + rhs.fragment_position,
            ..self
        }
    }
}

impl AddAssign<TangentSpaceInfo> for TangentSpaceInfo {
    fn add_assign(&mut self, rhs: TangentSpaceInfo) {
        self.tbn += rhs.tbn;
        self.tbn_inverse += rhs.tbn_inverse;
        self.normal += rhs.normal;
        self.point_light_position += rhs.point_light_position;
        self.view_position += rhs.view_position;
        self.fragment_position += rhs.fragment_position;
    }
}

impl Sub<TangentSpaceInfo> for TangentSpaceInfo {
    type Output = TangentSpaceInfo;
    fn sub(self, rhs: Self) -> TangentSpaceInfo {
        TangentSpaceInfo {
            tbn: self.tbn - rhs.tbn,
            tbn_inverse: self.tbn_inverse - rhs.tbn_inverse,
            normal: self.normal - rhs.normal,
            point_light_position: self.point_light_position - rhs.point_light_position,
            view_position: self.view_position - rhs.view_position,
            fragment_position: self.fragment_position - rhs.fragment_position,
            ..self
        }
    }
}

impl Mul<f32> for TangentSpaceInfo {
    type Output = TangentSpaceInfo;
    fn mul(self, scalar: f32) -> TangentSpaceInfo {
        TangentSpaceInfo {
            tbn: self.tbn * scalar,
            tbn_inverse: self.tbn_inverse * scalar,
            normal: self.normal * scalar,
            point_light_position: self.point_light_position * scalar,
            view_position: self.view_position * scalar,
            fragment_position: self.fragment_position * scalar,
            ..self
        }
    }
}

impl MulAssign<f32> for TangentSpaceInfo {
    fn mul_assign(&mut self, scalar: f32) {
        self.tbn *= scalar;
        self.tbn_inverse *= scalar;
        self.normal *= scalar;
        self.point_light_position *= scalar;
        self.view_position *= scalar;
        self.fragment_position *= scalar;
    }
}

impl Div<f32> for TangentSpaceInfo {
    type Output = TangentSpaceInfo;
    fn div(self, scalar: f32) -> TangentSpaceInfo {
        TangentSpaceInfo {
            tbn: self.tbn / scalar,
            tbn_inverse: self.tbn_inverse / scalar,
            normal: self.normal / scalar,
            point_light_position: self.point_light_position / scalar,
            view_position: self.view_position / scalar,
            fragment_position: self.fragment_position / scalar,
            ..self
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct DefaultVertexOut {
    pub position: Vec4,
    pub normal: Vec4,
    pub tangent_space_info: TangentSpaceInfo,
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
            tangent_space_info: self.tangent_space_info + rhs.tangent_space_info,
            color: self.color + rhs.color,
            uv: self.uv + rhs.uv,
            world_pos: self.world_pos + rhs.world_pos,
            depth: self.depth + rhs.depth,
        }
    }
}

impl AddAssign<DefaultVertexOut> for DefaultVertexOut {
    fn add_assign(&mut self, rhs: DefaultVertexOut) {
        self.position += rhs.position;
        self.normal += rhs.normal;
        self.tangent_space_info += rhs.tangent_space_info;
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
            tangent_space_info: self.tangent_space_info - rhs.tangent_space_info,
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
            tangent_space_info: self.tangent_space_info * scalar,
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
        self.tangent_space_info *= scalar;
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
            tangent_space_info: self.tangent_space_info / scalar,
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
