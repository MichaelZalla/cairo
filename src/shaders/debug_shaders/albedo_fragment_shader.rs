use std::sync::RwLock;

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
};

pub struct AlbedoFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for AlbedoFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // let context: std::sync::RwLockReadGuard<'_, ShaderContext> = self.context.read().unwrap();

        // Emit only the diffuse color for this fragment.

        return Color {
            r: (sample.diffuse.x as f32 * 255.0) as u8,
            g: (sample.diffuse.y as f32 * 255.0) as u8,
            b: (sample.diffuse.z as f32 * 255.0) as u8,
            a: 255 as u8,
        };
    }
}
