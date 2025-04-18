use std::f32::consts::PI;

use crate::{
    physics::pbr::{self, brdf::cook_torrance_direct},
    shader::geometry::sample::GeometrySample,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub mod ambient_light;
pub mod attenuation;
pub mod directional_light;
pub mod point_light;
pub mod spot_light;

pub mod shadow;

fn contribute_pbr(
    sample: &GeometrySample,
    normal: &Vec3,
    light_intensities: &Vec3,
    direction_to_light: &Vec3,
    direction_to_view: Vec3,
    f0: &Vec3,
) -> Vec3 {
    let likeness_to_light_direction = normal.dot(*direction_to_light).max(0.0);

    if likeness_to_light_direction > 0.0 {
        let radiance = *light_intensities;

        let halfway = (direction_to_view + direction_to_light).as_normal();

        let halfway_likeness_to_view = halfway.dot(direction_to_view);

        let fresnel = pbr::brdf::fresnel_schlick_direct(halfway_likeness_to_view, f0);

        // Rendering equation

        // Ratio of reflected light energy.
        let k_s = fresnel;

        // Ratio of refracted light energy.
        let k_d = (vec3::ONES - k_s) * (1.0 - sample.metallic);

        // BRDF

        let diffuse = k_d * (sample.albedo / PI);

        let likeness_to_view_direction = normal.dot(direction_to_view).max(0.0);

        let specular = cook_torrance_direct(
            sample,
            &halfway,
            &direction_to_view,
            likeness_to_view_direction,
            direction_to_light,
            likeness_to_light_direction,
            &fresnel,
        );

        (diffuse + specular) * radiance * likeness_to_light_direction
    } else {
        Default::default()
    }
}

pub(in crate::scene::light) fn contribute_pbr_tangent_space(
    sample: &GeometrySample,
    light_intensities: &Vec3,
    direction_to_light_tangent_space: &Vec3,
    f0: &Vec3,
) -> Vec3 {
    let tangent_space_info = sample.tangent_space_info;

    let normal = &tangent_space_info.normal;

    let direction_to_view =
        (tangent_space_info.view_position - tangent_space_info.fragment_position).as_normal();

    contribute_pbr(
        sample,
        normal,
        light_intensities,
        direction_to_light_tangent_space,
        direction_to_view,
        f0,
    )
}

pub(in crate::scene::light) fn contribute_pbr_world_space(
    sample: &GeometrySample,
    light_intensities: &Vec3,
    direction_to_light_world_space: &Vec3,
    f0: &Vec3,
    view_position: &Vec4,
) -> Vec3 {
    let normal = &sample.normal_world_space;

    let direction_to_view = (view_position.to_vec3() - sample.position_world_space).as_normal();

    contribute_pbr(
        sample,
        normal,
        light_intensities,
        direction_to_light_world_space,
        direction_to_view,
        f0,
    )
}
