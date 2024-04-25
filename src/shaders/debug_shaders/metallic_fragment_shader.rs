#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub const MetallicFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the metallic for this fragment.

        Color::from_vec3(Vec3::ones() * sample.metallic)
    };
