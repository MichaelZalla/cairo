#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const DepthFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the linear depth value (in RGB space) for this fragment.

        return Color {
            r: (sample.depth * 255.0) as u8,
            g: (sample.depth * 255.0) as u8,
            b: (sample.depth * 255.0) as u8,
            a: 255 as u8,
        };
    };
