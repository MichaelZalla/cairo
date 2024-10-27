#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub static DirectionalShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        let fragment_position_projection_space = Vec4::new(sample.position_world_space, 1.0)
            * context.view_inverse_transform
            * context.projection_transform;

        let fragment_depth_ndc_space =
            fragment_position_projection_space.z / fragment_position_projection_space.w;

        Color::from_vec3(Vec3::ones() * fragment_depth_ndc_space)
    };
