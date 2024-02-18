#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub const AlbedoFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the diffuse color for this fragment.

        Color::from_vec3(sample.diffuse)
    };
