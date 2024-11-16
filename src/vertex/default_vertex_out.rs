use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use crate::{
    matrix::Mat4,
    render::viewport::RenderViewport,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct TangentSpaceInfo {
    pub tbn: Mat4,
    pub tbn_inverse: Mat4,
    pub normal: Vec3,
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
            view_position: self.view_position + rhs.view_position,
            fragment_position: self.fragment_position + rhs.fragment_position,
        }
    }
}

impl AddAssign<TangentSpaceInfo> for TangentSpaceInfo {
    fn add_assign(&mut self, rhs: TangentSpaceInfo) {
        self.tbn += rhs.tbn;
        self.tbn_inverse += rhs.tbn_inverse;
        self.normal += rhs.normal;
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
            view_position: self.view_position - rhs.view_position,
            fragment_position: self.fragment_position - rhs.fragment_position,
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
            view_position: self.view_position * scalar,
            fragment_position: self.fragment_position * scalar,
        }
    }
}

impl MulAssign<f32> for TangentSpaceInfo {
    fn mul_assign(&mut self, scalar: f32) {
        self.tbn *= scalar;
        self.tbn_inverse *= scalar;
        self.normal *= scalar;
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
            view_position: self.view_position / scalar,
            fragment_position: self.fragment_position / scalar,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct DefaultVertexOut {
    pub position_world_space: Vec3,
    pub position_view_space: Vec3,
    pub position_projection_space: Vec4,
    pub normal_world_space: Vec4,
    pub tangent_world_space: Vec4,
    pub bitangent_world_space: Vec4,
    pub tangent_space_info: TangentSpaceInfo,
    pub color: Vec3,
    pub uv: Vec2,
    pub depth: f32,
}

impl DefaultVertexOut {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn projection_space_to_viewport_space(&mut self, viewport: &RenderViewport) {
        let w_inverse = 1.0 / self.position_projection_space.w;

        *self *= w_inverse;

        self.position_projection_space.x =
            (self.position_projection_space.x + 1.0) * viewport.width_over_2;

        self.position_projection_space.y =
            (-self.position_projection_space.y + 1.0) * viewport.height_over_2;

        self.position_projection_space.w = w_inverse;
    }
}

impl Add<DefaultVertexOut> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn add(self, rhs: Self) -> DefaultVertexOut {
        DefaultVertexOut {
            position_world_space: self.position_world_space + rhs.position_world_space,
            position_view_space: self.position_view_space + rhs.position_view_space,
            position_projection_space: self.position_projection_space
                + rhs.position_projection_space,
            normal_world_space: self.normal_world_space + rhs.normal_world_space,
            tangent_world_space: self.tangent_world_space + rhs.tangent_world_space,
            bitangent_world_space: self.bitangent_world_space + rhs.bitangent_world_space,
            tangent_space_info: self.tangent_space_info + rhs.tangent_space_info,
            color: self.color + rhs.color,
            uv: self.uv + rhs.uv,
            depth: self.depth + rhs.depth,
        }
    }
}

impl AddAssign<DefaultVertexOut> for DefaultVertexOut {
    fn add_assign(&mut self, rhs: DefaultVertexOut) {
        self.position_world_space += rhs.position_world_space;
        self.position_view_space += rhs.position_view_space;
        self.position_projection_space += rhs.position_projection_space;
        self.normal_world_space += rhs.normal_world_space;
        self.tangent_world_space += rhs.tangent_world_space;
        self.bitangent_world_space += rhs.bitangent_world_space;
        self.tangent_space_info += rhs.tangent_space_info;
        self.color += rhs.color;
        self.uv += rhs.uv;
        self.depth += rhs.depth;
    }
}

impl Sub<DefaultVertexOut> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn sub(self, rhs: Self) -> DefaultVertexOut {
        DefaultVertexOut {
            position_world_space: self.position_world_space - rhs.position_world_space,
            position_view_space: self.position_view_space - rhs.position_view_space,
            position_projection_space: self.position_projection_space
                - rhs.position_projection_space,
            normal_world_space: self.normal_world_space - rhs.normal_world_space,
            tangent_world_space: self.tangent_world_space - rhs.tangent_world_space,
            bitangent_world_space: self.bitangent_world_space - rhs.bitangent_world_space,
            tangent_space_info: self.tangent_space_info - rhs.tangent_space_info,
            color: self.color - rhs.color,
            uv: self.uv - rhs.uv,
            depth: self.depth - rhs.depth,
        }
    }
}

impl Mul<f32> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn mul(self, scalar: f32) -> DefaultVertexOut {
        DefaultVertexOut {
            position_world_space: self.position_world_space * scalar,
            position_view_space: self.position_view_space * scalar,
            position_projection_space: self.position_projection_space * scalar,
            normal_world_space: self.normal_world_space * scalar,
            tangent_world_space: self.tangent_world_space * scalar,
            bitangent_world_space: self.bitangent_world_space * scalar,
            tangent_space_info: self.tangent_space_info * scalar,
            color: self.color * scalar,
            uv: self.uv * scalar,
            depth: self.depth * scalar,
        }
    }
}

impl MulAssign<f32> for DefaultVertexOut {
    fn mul_assign(&mut self, scalar: f32) {
        self.position_world_space *= scalar;
        self.position_view_space *= scalar;
        self.position_projection_space *= scalar;
        self.normal_world_space *= scalar;
        self.tangent_world_space *= scalar;
        self.bitangent_world_space *= scalar;
        self.tangent_space_info *= scalar;
        self.color *= scalar;
        self.uv *= scalar;
        self.depth *= scalar;
    }
}

impl Div<f32> for DefaultVertexOut {
    type Output = DefaultVertexOut;
    fn div(self, scalar: f32) -> DefaultVertexOut {
        DefaultVertexOut {
            position_world_space: self.position_world_space / scalar,
            position_view_space: self.position_view_space / scalar,
            position_projection_space: self.position_projection_space / scalar,
            normal_world_space: self.normal_world_space / scalar,
            tangent_world_space: self.tangent_world_space / scalar,
            bitangent_world_space: self.bitangent_world_space / scalar,
            tangent_space_info: self.tangent_space_info / scalar,
            color: self.color / scalar,
            uv: self.uv / scalar,
            depth: self.depth / scalar,
        }
    }
}
