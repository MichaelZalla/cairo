#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const SpecularRoughnessFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the specular roughness (exponent) for this fragment.

        let value = 1.0 - (1.0 / 64.0 * sample.specular_exponent as f32).max(0.0);

        Color {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    };
