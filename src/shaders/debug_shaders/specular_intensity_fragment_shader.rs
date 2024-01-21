use std::sync::RwLock;

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
};

pub struct SpecularIntensityFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for SpecularIntensityFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // Emit only the specular intensity for this fragment.

        return Color {
            r: (sample.specular_intensity * 255.0) as u8,
            g: (sample.specular_intensity * 255.0) as u8,
            b: (sample.specular_intensity * 255.0) as u8,
            a: 255 as u8,
        };
    }
}
