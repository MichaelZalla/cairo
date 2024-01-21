use std::sync::RwLock;

use crate::{
    color::Color,
    shader::{fragment::FragmentShader, geometry::sample::GeometrySample, ShaderContext},
};

pub struct DepthFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

impl<'a> FragmentShader<'a> for DepthFragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, sample: &GeometrySample) -> Color {
        // Emit only the linear depth value (in RGB space) for this fragment.

        let non_linear_depth: f32 = sample.depth;

        //               nlz = (1/z - 1/n) / (1/f - 1/n)
        // nlz * (1/f - 1/n) = 1/z - 1/n
        //               1/z = nlz * (1/f - 1/n) + 1/n
        //                 z = 1 / (nlz * (1/f - 1/n) + 1/n)

        static NEAR: f32 = 0.3;
        static FAR: f32 = 1000.0;

        let ndc = non_linear_depth * 2.0 - 1.0;

        let linear_depth = (2.0 * NEAR * FAR) / (FAR + NEAR - ndc * (FAR - NEAR));

        // [0, 1] -> [10, 0]
        let adjusted_linear_depth = 10.0 - (linear_depth * 10.0);

        // @NOTE May need to account for diplacement map in the future.
        return Color {
            r: (adjusted_linear_depth * 255.0) as u8,
            g: (adjusted_linear_depth * 255.0) as u8,
            b: (adjusted_linear_depth * 255.0) as u8,
            a: 255 as u8,
        };
    }
}
