use std::sync::RwLock;

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
    texture::{sample::sample_bilinear, TextureMap},
};

pub struct UvTestFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
    texture_map: Option<TextureMap>,
}

impl<'a> UvTestFragmentShader<'a> {
    pub fn from_texture_map(context: &'a RwLock<ShaderContext>, texture_map: TextureMap) -> Self {
        let mut shader = UvTestFragmentShader::new(context);

        shader.texture_map = Some(texture_map);

        shader
    }
}

impl<'a> FragmentShader<'a> for UvTestFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self {
            context,
            texture_map: None,
        }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // Emit an RGB representation of this fragment's interpolated UV.

        let r: u8;
        let g: u8;
        let b: u8;

        match &self.texture_map {
            Some(texture) => (r, g, b) = sample_bilinear(sample.uv, texture, None),
            None => {
                r = (sample.uv.x * 255.0) as u8;
                g = (sample.uv.y * 255.0) as u8;
                b = (sample.uv.z * 255.0) as u8;
            }
        }

        return Color::rgb(r, g, b);
    }
}
