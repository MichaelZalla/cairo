#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const SpecularIntensityFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the specular intensity for this fragment.

        return Color {
            r: (sample.specular_intensity * 255.0) as u8,
            g: (sample.specular_intensity * 255.0) as u8,
            b: (sample.specular_intensity * 255.0) as u8,
            a: 255 as u8,
        };
    };
