#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    texture::sample::sample_bilinear,
    vec::vec3::Vec3,
};

pub const UvTestFragmentShader: FragmentShaderFn = |context: &ShaderContext,
                                                    sample: &GeometrySample|
 -> Color {
    // Emit an RGB representation of this fragment's interpolated UV.

    let r: u8;
    let g: u8;
    let b: u8;

    match &context.active_uv_test_texture_map {
        Some(handle) => match &context.resources {
            Some(resources) => match resources.borrow().texture.borrow().get(&handle) {
                Ok(entry) => {
                    let map = &entry.item;

                    (r, g, b) = sample_bilinear(sample.uv, map, None);

                    return Color::from_vec3(Vec3 {
                        x: r as f32 / 255.0,
                        y: g as f32 / 255.0,
                        z: b as f32 / 255.0,
                    });
                }
                Err(err) => panic!("Failed to get TextureMap from Arena: {:?}: {}", handle, err),
            },
            None => (),
        },
        None => (),
    }

    Color::from_vec3(Vec3 {
        x: sample.uv.x,
        y: sample.uv.y,
        z: sample.uv.z,
    })
};
