use std::f32::consts::TAU;

use crate::vec::{vec2::Vec2, vec3::Vec3};

pub fn importance_sample_ggx(x_i: Vec2, normal: &Vec3, roughness: f32) -> Vec3 {
    let alpha = roughness * roughness;

    let phi = TAU * x_i.x;

    let cosine_theta = ((1.0 - x_i.y) / (1.0 + (alpha * alpha - 1.0) * x_i.y)).sqrt();

    let sin_theta = (1.0 - cosine_theta * cosine_theta).sqrt();

    // Spherical coordinates to tangent space.

    let tangent_space_halfway = Vec3 {
        x: phi.cos() * sin_theta,
        y: phi.sin() * sin_theta,
        z: cosine_theta,
    };

    // Tangent space to world space.

    let (forward, right, up) = normal.basis();

    let biased_sample = right * tangent_space_halfway.x
        + up * tangent_space_halfway.y
        + forward * tangent_space_halfway.z;

    biased_sample.as_normal()
}
