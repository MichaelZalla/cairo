#![allow(non_upper_case_globals)]

use crate::{
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec3::{self, Vec3},
};

pub static StencilFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit only the stencil value for this fragment (set or not set).

        if sample.stencil {
            vec3::ONES
        } else {
            Default::default()
        }
    };
