use std::f32::consts::PI;

use crate::vec::vec3::{self, Vec3};

// Distribution

pub fn distribution_ggx(normal: &Vec3, halfway: &Vec3, roughness: f32) -> f32 {
    let a = roughness.powi(2);
    let a2 = a.powi(2);

    let likeness = normal.dot(*halfway).max(0.0);
    let likeness2 = likeness.powi(2);

    let numerator = a2;

    let denominator = likeness2 * (a2 - 1.0) + 1.0;
    let denominator = PI * denominator.powi(2);

    numerator / denominator
}

// Geometry

pub fn geometry_schlick_ggx(normal_dot_view: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = r.powi(2) / 8.0;

    let numerator = normal_dot_view;
    let denominator = normal_dot_view * (1.0 - k) + k;

    numerator / denominator
}

pub fn geometry_smith(normal: &Vec3, view: &Vec3, light: &Vec3, roughness: f32) -> f32 {
    let normal_dot_light = normal.dot(*light).max(0.0);
    let ggx1 = geometry_schlick_ggx(normal_dot_light, roughness);

    let normal_dot_view = normal.dot(*view).max(0.0);
    let ggx2 = geometry_schlick_ggx(normal_dot_view, roughness);

    ggx1 * ggx2
}

// Fresnel

pub fn fresnel_schlick_direct(likeness: f32, f0: &Vec3) -> Vec3 {
    *f0 + (vec3::ONES - *f0) * (1.0 - likeness).clamp(0.0, 1.0).powi(5)
}

pub fn fresnel_schlick_indirect(likeness: f32, f0: &Vec3, roughness: f32) -> Vec3 {
    *f0 + (vec3::ONES * (1.0 - roughness) - *f0) * (1.0 - likeness).clamp(0.0, 1.0).powi(5)
}
