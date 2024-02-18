#![allow(non_upper_case_globals)]

use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
};

pub const StencilFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the stencil value for this fragment (black or white).

        let value = if sample.stencil {
            1.0 as f32
        } else {
            0.0 as f32
        };

        Color {
            r: value,
            g: value,
            b: value,
            a: 1.0,
        }
    };
