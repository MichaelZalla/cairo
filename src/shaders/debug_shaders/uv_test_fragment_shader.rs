#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    texture::sample::sample_bilinear_u8,
    vec::vec3::Vec3,
};

pub static UvTestFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit an RGB representation of this fragment's interpolated UV.

        let r: u8;
        let g: u8;
        let b: u8;

        match &context.active_uv_test_texture_map {
            Some(handle) => match resources.texture_u8.borrow().get(handle) {
                Ok(entry) => {
                    let map = &entry.item;

                    (r, g, b) = sample_bilinear_u8(sample.uv, map, None);

                    return Vec3 {
                        x: r as f32 / 255.0,
                        y: g as f32 / 255.0,
                        z: b as f32 / 255.0,
                    };
                }
                Err(err) => panic!("Failed to get TextureMap from Arena: {:?}: {}", handle, err),
            },
            None => (),
        }

        Vec3 {
            x: sample.uv.x,
            y: sample.uv.y,
            z: sample.uv.z,
        }
    };
