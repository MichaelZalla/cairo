use std::ops::{Add, Div, Mul, Sub};

use crate::{
    vec::{vec2::Vec2, vec3::Vec3},
    vertex::default_vertex_out::TangentSpaceInfo,
};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct GeometrySample {
    pub stencil: bool,
    // @TODO reconstruct from depth sample + pixel coordinate (index)
    pub world_pos: Vec3,
    // Non-linear Z in world-view-projection space
    // @TODO reconstruct z component:
    //   normal.z = (1.0 - normal.x^2 - normal.y^2).sqrt()
    pub world_normal: Vec3,
    pub uv: Vec2,
    pub depth: f32,
    pub tangent_space_info: TangentSpaceInfo,
    // Common
    pub specular_color: Vec3,
    pub specular_exponent: u8,
    pub emissive_color: Vec3,
    pub alpha: f32,
    // Blinn-Phong
    pub ambient_factor: f32,
    pub diffuse_color: Vec3,
    // PBR
    pub albedo: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    // pub sheen: f32,
    // pub clearcoat_thickness: f32,
    // pub clearcoat_roughness: f32,
    // pub anisotropy: f32,
    // pub anisotropy_rotation: f32,
}

impl Add<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn add(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv + rhs.uv,
            albedo: self.albedo + rhs.albedo,
            roughness: self.roughness + rhs.roughness,
            metallic: self.metallic + rhs.metallic,
            ambient_factor: self.ambient_factor + rhs.ambient_factor,
            diffuse_color: self.diffuse_color + rhs.diffuse_color,
            world_normal: self.world_normal + rhs.world_normal,
            tangent_space_info: self.tangent_space_info, // + rhs.tangent_space_info,
            world_pos: self.world_pos + rhs.world_pos,
            depth: self.depth + rhs.depth,
            specular_exponent: self.specular_exponent + rhs.specular_exponent,
            specular_color: self.specular_color + rhs.specular_color,
            emissive_color: self.emissive_color + rhs.emissive_color,
            alpha: self.alpha + rhs.alpha,
            // sheen: self.sheen + rhs.sheen,
            // clearcoat_thickness: self.clearcoat_thickness + rhs.clearcoat_thickness,
            // clearcoat_roughness: self.clearcoat_roughness + rhs.clearcoat_roughness,
            // anisotropy: self.anisotropy + rhs.anisotropy,
            // anisotropy_rotation: self.anisotropy_rotation + rhs.anisotropy_rotation,
        }
    }
}

impl Sub<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn sub(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv - rhs.uv,
            albedo: self.albedo - rhs.albedo,
            roughness: self.roughness - rhs.roughness,
            metallic: self.metallic - rhs.metallic,
            ambient_factor: self.ambient_factor - rhs.ambient_factor,
            diffuse_color: self.diffuse_color - rhs.diffuse_color,
            world_normal: self.world_normal - rhs.world_normal,
            tangent_space_info: self.tangent_space_info, // - rhs.tangent_space_info,
            world_pos: self.world_pos - rhs.world_pos,
            depth: self.depth - rhs.depth,
            specular_exponent: self.specular_exponent - rhs.specular_exponent,
            specular_color: self.specular_color - rhs.specular_color,
            emissive_color: self.emissive_color - rhs.emissive_color,
            alpha: self.alpha - rhs.alpha,
            // sheen: self.sheen - rhs.sheen,
            // clearcoat_thickness: self.clearcoat_thickness - rhs.clearcoat_thickness,
            // clearcoat_roughness: self.clearcoat_roughness - rhs.clearcoat_roughness,
            // anisotropy: self.anisotropy - rhs.anisotropy,
            // anisotropy_rotation: self.anisotropy_rotation - rhs.anisotropy_rotation,
        }
    }
}

impl Mul<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn mul(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv * rhs.uv,
            albedo: self.albedo * rhs.albedo,
            roughness: self.roughness * rhs.roughness,
            metallic: self.metallic * rhs.metallic,
            ambient_factor: self.ambient_factor * rhs.ambient_factor,
            diffuse_color: self.diffuse_color * rhs.diffuse_color,
            world_normal: self.world_normal * rhs.world_normal,
            tangent_space_info: self.tangent_space_info, // * rhs.tangent_space_info,
            world_pos: self.world_pos * rhs.world_pos,
            depth: self.depth * rhs.depth,
            specular_exponent: self.specular_exponent * rhs.specular_exponent,
            specular_color: self.specular_color * rhs.specular_color,
            emissive_color: self.emissive_color * rhs.emissive_color,
            alpha: self.alpha * rhs.alpha,
            // sheen: self.sheen * rhs.sheen,
            // clearcoat_thickness: self.clearcoat_thickness * rhs.clearcoat_thickness,
            // clearcoat_roughness: self.clearcoat_roughness * rhs.clearcoat_roughness,
            // anisotropy: self.anisotropy * rhs.anisotropy,
            // anisotropy_rotation: self.anisotropy_rotation * rhs.anisotropy_rotation,
        }
    }
}

impl Div<GeometrySample> for GeometrySample {
    type Output = GeometrySample;

    fn div(self, rhs: Self) -> Self::Output {
        GeometrySample {
            stencil: self.stencil,
            uv: self.uv / rhs.uv,
            albedo: self.albedo / rhs.albedo,
            roughness: self.roughness / rhs.roughness,
            metallic: self.metallic / rhs.metallic,
            ambient_factor: self.ambient_factor / rhs.ambient_factor,
            diffuse_color: self.diffuse_color / rhs.diffuse_color,
            world_normal: self.world_normal / rhs.world_normal,
            tangent_space_info: self.tangent_space_info, // / rhs.tangent_space_info,
            world_pos: self.world_pos / rhs.world_pos,
            depth: self.depth / rhs.depth,
            specular_exponent: self.specular_exponent / rhs.specular_exponent,
            specular_color: self.specular_color / rhs.specular_color,
            emissive_color: self.emissive_color / rhs.emissive_color,
            alpha: self.alpha / rhs.alpha,
            // sheen: self.sheen / rhs.sheen,
            // clearcoat_thickness: self.clearcoat_thickness / rhs.clearcoat_thickness,
            // clearcoat_roughness: self.clearcoat_roughness / rhs.clearcoat_roughness,
            // anisotropy: self.anisotropy / rhs.anisotropy,
            // anisotropy_rotation: self.anisotropy_rotation / rhs.anisotropy_rotation,
        }
    }
}
