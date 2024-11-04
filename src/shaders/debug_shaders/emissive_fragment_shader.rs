#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub static EmissiveFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit only the emissive color for this fragment.

        sample.emissive_color
    };
