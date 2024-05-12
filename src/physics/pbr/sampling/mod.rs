use std::f32::consts::PI;

use crate::vec::{
    vec2::Vec2,
    vec3::{self, Vec3},
};

pub fn importance_sample_ggx(x_i: Vec2, normal: &Vec3, roughness: f32) -> Vec3 {
    let alpha = roughness * roughness;

    let phi = 2.0 * PI * x_i.x;

    let cosine_theta = ((1.0 - x_i.y) / (1.0 + (alpha * alpha - 1.0) * x_i.y)).sqrt();

    let sin_theta = (1.0 - cosine_theta * cosine_theta).sqrt();

    // Spherical coordinates to tangent space.

    let tangent_space_halfway = Vec3 {
        x: phi.cos() * sin_theta,
        y: phi.sin() * sin_theta,
        z: cosine_theta,
    };

    // Tangent space to world space.

    let up = if normal.z.abs() < 0.999 {
        vec3::UP
    } else {
        vec3::RIGHT
    };

    let tangent = up.cross(*normal).as_normal();

    let bitangent = normal.cross(tangent);

    let biased_sample = tangent * tangent_space_halfway.x
        + bitangent * tangent_space_halfway.y
        + *normal * tangent_space_halfway.z;

    biased_sample.as_normal()
}
