use crate::vec::vec3::Vec3;
use crate::vec::vec4::Vec4;

#[derive(Debug, Copy, Clone)]
pub struct AmbientLight {
    pub intensities: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    pub direction: Vec4,
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub intensities: Vec3,
    pub position: Vec3,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct SpotLight {
    pub intensities: Vec3,
    pub position: Vec3,
    pub direction: Vec3,
    pub inner_cutoff_angle: f32,
    pub outer_cutoff_angle: f32,
    pub constant_attenuation: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
}
