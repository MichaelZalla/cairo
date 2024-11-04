#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::Vec3,
};

pub static NormalFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit only the world-space normal for this fragment.

        Vec3 {
            x: sample.normal_world_space.x,
            y: sample.normal_world_space.y,
            z: (1.0 - sample.normal_world_space.z),
        }
    };
