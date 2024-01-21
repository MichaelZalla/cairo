use std::sync::RwLock;

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
};

pub struct SpecularRoughnessFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for SpecularRoughnessFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // Emit only the specular roughness (exponent) for this fragment.

        let value = (255.0 - (255.0 / 64.0 * sample.specular_exponent as f32).max(0.0)) as u8;

        return Color {
            r: value,
            g: value,
            b: value,
            a: 255 as u8,
        };
    }
}
