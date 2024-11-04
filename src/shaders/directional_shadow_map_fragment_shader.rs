#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub static DirectionalShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        let fragment_position_projection_space = Vec4::new(sample.position_world_space, 1.0)
            * context.view_inverse_transform
            * context.projection_transform;

        let fragment_depth_ndc_space =
            fragment_position_projection_space.z / fragment_position_projection_space.w;

        vec3::ONES * fragment_depth_ndc_space
    };
