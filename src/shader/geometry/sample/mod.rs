use crate::vec::{vec2::Vec2, vec3::Vec3};

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
    // @TODO reconstruct from depth sample + pixel coordinate (index)
    pub world_pos: Vec3, // 12 bytes
    // Non-linear Z in world-view-projection space
    pub depth: f32,
    // @TODO could be an i8
    pub specular_exponent: i32, //  4 bytes
    // @TODO could be an i8 (0 -> 255, 0.0 -> 1.0)
    pub specular_intensity: f32, //  4 bytes
    pub emissive: Vec3,          // 12 bytes
}
