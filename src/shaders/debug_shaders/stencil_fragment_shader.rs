use std::{marker::PhantomData, sync::RwLock};

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
};

pub struct StencilFragmentShader<'a> {
    _phantom: PhantomData<&'a bool>,
}

impl<'a> FragmentShader<'a> for StencilFragmentShader<'a> {
    fn new(_context: &'a RwLock<ShaderContext>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // Emit only the stencil value for this fragment (black or white).

        let value = if sample.stencil { 255 as u8 } else { 0 as u8 };

        return Color {
            r: value,
            g: value,
            b: value,
            a: 255 as u8,
        };
    }
}
