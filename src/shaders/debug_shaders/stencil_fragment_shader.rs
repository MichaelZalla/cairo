use std::sync::RwLockReadGuard;

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
};

pub const StencilFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // Emit only the stencil value for this fragment (black or white).

        let value = if sample.stencil { 255 as u8 } else { 0 as u8 };

        Color {
            r: value,
            g: value,
            b: value,
            a: 255 as u8,
        }
    };
