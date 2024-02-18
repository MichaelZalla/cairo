#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub const SpecularIntensityFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the specular intensity for this fragment.

        return Color {
            r: sample.specular_intensity,
            g: sample.specular_intensity,
            b: sample.specular_intensity,
            a: 1.0,
        };
    };
