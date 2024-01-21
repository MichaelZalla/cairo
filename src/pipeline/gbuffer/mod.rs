use crate::shader::geometry::sample::GeometrySample;

pub struct GBuffer {
    pub width: u32,
    pub height: u32,
    pub samples: Vec<GeometrySample>,
}

impl GBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let mut samples: Vec<GeometrySample> = vec![];

        samples.resize(width as usize * height as usize, Default::default());

        Self {
            width,
            height,
            samples,
        }
    }

    pub fn clear(&mut self) {
        for sample in &mut self.samples {
            sample.stencil = false;
        }
    }

    pub fn set(&mut self, index: usize, sample: GeometrySample) {
        self.samples[index] = sample;
        self.samples[index].stencil = true;
    }
}
