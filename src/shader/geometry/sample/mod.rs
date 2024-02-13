use std::ops::{Add, Div, Mul, Sub};

use crate::{
    vec::{vec2::Vec2, vec3::Vec3},
    vertex::default_vertex_out::TangentSpaceInfo,
};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct GeometrySample {
    pub stencil: bool, //  1 byte (aligned to 4)
    pub uv: Vec2,
    // @TODO ambient_factor becomes light accumulation texture
    pub ambient_factor: f32, //  4 bytes
    // @TODO diffuse is albedo color?
    pub diffuse: Vec3, // 12 bytes
    // @TODO world normal?
    // @TODO reconstruct z component:
    //          normal.z = (1.0 - normal.x^2 - normal.y^2).sqrt()
    pub normal: Vec3, // 12 bytes
    pub tangent_space_info: TangentSpaceInfo,
    // @TODO reconstruct from depth sample + pixel coordinate (index)
    pub world_pos: Vec3, // 12 bytes
    // Non-linear Z in world-view-projection space
    pub depth: f32,
    pub displacement: f32,
    // @TODO could be an i8
    pub specular_exponent: i32, //  4 bytes
    // @TODO could be an i8 (0 -> 255, 0.0 -> 1.0)
    pub specular_intensity: f32, //  4 bytes
    pub emissive: Vec3,          // 12 bytes
    pub alpha: f32,              // 4 bytes
}

impl Add<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn add(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv + rhs.uv,
            ambient_factor: self.ambient_factor + rhs.ambient_factor,
            diffuse: self.diffuse + rhs.diffuse,
            normal: self.normal + rhs.normal,
            tangent_space_info: self.tangent_space_info,
            world_pos: self.world_pos + rhs.world_pos,
            depth: self.depth + rhs.depth,
            displacement: self.displacement + rhs.displacement,
            specular_exponent: self.specular_exponent + rhs.specular_exponent,
            specular_intensity: self.specular_intensity + rhs.specular_intensity,
            emissive: self.emissive + rhs.emissive,
            alpha: self.alpha + rhs.alpha,
        }
    }
}

impl Sub<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn sub(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv - rhs.uv,
            ambient_factor: self.ambient_factor - rhs.ambient_factor,
            diffuse: self.diffuse - rhs.diffuse,
            normal: self.normal - rhs.normal,
            tangent_space_info: self.tangent_space_info,
            world_pos: self.world_pos - rhs.world_pos,
            depth: self.depth - rhs.depth,
            displacement: self.displacement - rhs.displacement,
            specular_exponent: self.specular_exponent - rhs.specular_exponent,
            specular_intensity: self.specular_intensity - rhs.specular_intensity,
            emissive: self.emissive - rhs.emissive,
            alpha: self.alpha - rhs.alpha,
        }
    }
}

impl Mul<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn mul(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv * rhs.uv,
            ambient_factor: self.ambient_factor * rhs.ambient_factor,
            diffuse: self.diffuse * rhs.diffuse,
            normal: self.normal * rhs.normal,
            tangent_space_info: self.tangent_space_info,
            world_pos: self.world_pos * rhs.world_pos,
            depth: self.depth * rhs.depth,
            displacement: self.displacement * rhs.displacement,
            specular_exponent: self.specular_exponent * rhs.specular_exponent,
            specular_intensity: self.specular_intensity * rhs.specular_intensity,
            emissive: self.emissive * rhs.emissive,
            alpha: self.alpha * rhs.alpha,
        }
    }
}

impl Div<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn div(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv / rhs.uv,
            ambient_factor: self.ambient_factor / rhs.ambient_factor,
            diffuse: self.diffuse / rhs.diffuse,
            normal: self.normal / rhs.normal,
            tangent_space_info: self.tangent_space_info,
            world_pos: self.world_pos / rhs.world_pos,
            depth: self.depth / rhs.depth,
            displacement: self.displacement / rhs.displacement,
            specular_exponent: self.specular_exponent / rhs.specular_exponent,
            specular_intensity: self.specular_intensity / rhs.specular_intensity,
            emissive: self.emissive / rhs.emissive,
            alpha: self.alpha / rhs.alpha,
        }
    }
}
