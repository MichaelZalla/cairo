#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub static NormalFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the world-space normal (RBG space) for this fragment.

        Color {
            r: sample.normal_world_space.x,
            g: sample.normal_world_space.y,
            b: (1.0 - sample.normal_world_space.z),
            a: 1.0,
        }
    };
