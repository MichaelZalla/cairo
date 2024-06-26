#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub static StencilFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the stencil value for this fragment (black or white).

        let value = if sample.stencil { 1.0_f32 } else { 0.0_f32 };

        Color {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    };
