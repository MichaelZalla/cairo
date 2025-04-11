use crate::{buffer::Buffer2D, shader::geometry::sample::GeometrySample};

#[derive(Default, Debug, Clone)]
pub struct GBuffer(pub Buffer2D<GeometrySample>);

impl GBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = Buffer2D::new(width, height, None);

        Self(buffer)
    }

    pub fn get(&self, x: u32, y: u32) -> &GeometrySample {
        self.0.get(x, y)
    }

    pub fn set(&mut self, x: u32, y: u32, sample: GeometrySample) {
        self.0.set(x, y, sample);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, GeometrySample> {
        self.0.iter()
    }
}
