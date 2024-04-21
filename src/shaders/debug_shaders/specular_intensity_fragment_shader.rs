#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub const SpecularIntensityFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the specular intensity for this fragment.

        return Color {
            r: sample.specular_intensity,
            g: sample.specular_intensity,
            b: sample.specular_intensity,
            a: 1.0,
        };
    };
