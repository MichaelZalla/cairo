use std::f32::consts::PI;

use crate::{
    shader::geometry::sample::GeometrySample,
    vec::vec3::{self, Vec3},
};

// Normal distribution function

pub fn distribution_ggx(normal: &Vec3, halfway: &Vec3, roughness: f32) -> f32 {
    let a = roughness.powi(2);
    let a2 = a.powi(2);

    let likeness_to_halfway = normal.dot(*halfway).max(0.0);
    let likeness_to_halfway_squared = likeness_to_halfway.powi(2);

    let numerator = a2;

    let denominator = likeness_to_halfway_squared * (a2 - 1.0) + 1.0;
    let denominator = PI * denominator.powi(2);

    numerator / denominator
}

// Geometry

fn geometry_schlick_ggx_direct(likeness_to_view_direction: f32, roughness: f32) -> f32 {
    let a = roughness + 1.0;
    let k = a.powi(2) / 8.0;

    let numerator = likeness_to_view_direction;
    let denominator = likeness_to_view_direction * (1.0 - k) + k;

    numerator / denominator
}

pub fn geometry_smith_direct(normal: &Vec3, view: &Vec3, light: &Vec3, roughness: f32) -> f32 {
    let normal_dot_light = normal.dot(*light).max(0.0);
    let ggx1 = geometry_schlick_ggx_direct(normal_dot_light, roughness);

    let normal_dot_view = normal.dot(*view).max(0.0);
    let ggx2 = geometry_schlick_ggx_direct(normal_dot_view, roughness);

    ggx1 * ggx2
}

// Fresnel

pub fn fresnel_schlick_direct(halfway_likeness_to_view: f32, f0: &Vec3) -> Vec3 {
    *f0 + (vec3::ONES - *f0) * (1.0 - halfway_likeness_to_view).clamp(0.0, 1.0).powi(5)
}

pub fn fresnel_schlick_indirect(likeness: f32, f0: &Vec3, roughness: f32) -> Vec3 {
    *f0 + (vec3::ONES * (1.0 - roughness) - *f0) * (1.0 - likeness).clamp(0.0, 1.0).powi(5)
}

// Cook-Torrance BRDF

pub fn cook_torrance_direct(
    sample: &GeometrySample,
    halfway: &Vec3,
    direction_to_view_position: &Vec3,
    likeness_to_view_direction: f32,
    direction_to_light: &Vec3,
    likeness_to_light_direction: f32,
    fresnel: &Vec3,
) -> Vec3 {
    let normal = &sample.tangent_space_info.normal;

    let distribution = distribution_ggx(normal, halfway, sample.roughness);

    let geometry = geometry_smith_direct(
        normal,
        direction_to_view_position,
        direction_to_light,
        sample.roughness,
    );

    // Specular reflection contribution.

    let numerator = *fresnel * distribution * geometry;
    let denominator = 4.0 * likeness_to_view_direction * likeness_to_light_direction + 0.0001;

    numerator / denominator
}
