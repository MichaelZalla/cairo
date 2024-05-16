#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec4::Vec4,
};

pub static NormalFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the world-space normal (RBG space) for this fragment.

        let world_normal_vec4 = Vec4::new(sample.world_normal, 1.0);

        Color {
            r: world_normal_vec4.x,
            g: world_normal_vec4.y,
            b: (1.0 - world_normal_vec4.z),
            a: 1.0,
        }
    };
