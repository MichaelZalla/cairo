#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const AlbedoFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the diffuse color for this fragment.

        Color {
            r: (sample.diffuse.x as f32 * 255.0) as u8,
            g: (sample.diffuse.y as f32 * 255.0) as u8,
            b: (sample.diffuse.z as f32 * 255.0) as u8,
            a: 255 as u8,
        }
    };
