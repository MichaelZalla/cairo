#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::{self, Vec3},
};

pub static SpecularRoughnessFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit only the specular roughness (exponent) for this fragment.

        static SCALE: f32 = 1.0 / 64.0;

        let scaled_roughness = 1.0 - (SCALE * sample.specular_exponent as f32).max(0.0);

        vec3::ONES * scaled_roughness
    };
