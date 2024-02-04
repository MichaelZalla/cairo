use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
    texture::sample::sample_bilinear,
};

pub const UvTestFragmentShader: FragmentShaderFn =
    |context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit an RGB representation of this fragment's interpolated UV.

        let r: u8;
        let g: u8;
        let b: u8;

        match context.active_test_uv_texture_map {
            Some(map_raw_mut) => unsafe {
                let map = &(*map_raw_mut);
                (r, g, b) = sample_bilinear(sample.uv, map, None)
            },
            None => {
                r = (sample.uv.x * 255.0) as u8;
                g = (sample.uv.y * 255.0) as u8;
                b = (sample.uv.z * 255.0) as u8;
            }
        }

        Color::rgb(r, g, b)
    };
