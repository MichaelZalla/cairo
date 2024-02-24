#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub const DepthFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, sample: &GeometrySample| -> Color {
        // Emit only the linear depth value (in RGB space) for this fragment.

        Color::from_vec3(Vec3 {
            x: sample.depth,
            y: sample.depth,
            z: sample.depth,
        })
    };
