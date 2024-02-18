#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
    texture::sample::sample_bilinear,
    vec::vec3::Vec3,
};

pub const UvTestFragmentShader: FragmentShaderFn =
    |context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit an RGB representation of this fragment's interpolated UV.

        let r: u8;
        let g: u8;
        let b: u8;

        match context.active_uv_test_texture_map {
            Some(map_raw_mut) => unsafe {
                let map = &(*map_raw_mut);
                (r, g, b) = sample_bilinear(sample.uv, map, None);

                Color::from_vec3(Vec3 {
                    x: r as f32 / 255.0,
                    y: g as f32 / 255.0,
                    z: b as f32 / 255.0,
                })
            },
            None => Color::from_vec3(Vec3 {
                x: sample.uv.x,
                y: sample.uv.y,
                z: sample.uv.z,
            }),
        }
    };
