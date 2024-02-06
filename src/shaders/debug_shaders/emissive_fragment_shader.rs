#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const EmissiveFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the emissive color for this fragment.

        return Color {
            r: (sample.emissive.x as f32 * 255.0) as u8,
            g: (sample.emissive.y as f32 * 255.0) as u8,
            b: (sample.emissive.z as f32 * 255.0) as u8,
            a: 255 as u8,
        };
    };
