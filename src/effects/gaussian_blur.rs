use std::fmt::Debug;

use crate::{buffer::Buffer2D, effect::Effect, vec::vec3::Vec3};

#[derive(Debug, Clone)]
pub struct GaussianBlurEffect {
    passes: u8,
    weights: [f32; 5],
    swap: Buffer2D<Vec3>,
}

impl Default for GaussianBlurEffect {
    fn default() -> Self {
        Self {
            passes: 6,
            weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
            swap: Default::default(),
        }
    }
}

impl GaussianBlurEffect {
    pub fn new(passes: u8) -> Self {
        Self {
            passes,
            ..Default::default()
        }
    }
}

impl Effect<Vec3> for GaussianBlurEffect {
    fn apply(&mut self, buffer: &mut Buffer2D<Vec3>) {
        // Two-pass gaussian blur with ping-pong buffers.

        // Resize our effect's swap buffer, if needed.

        self.swap.resize(buffer.width, buffer.height);

        for _ in 0..self.passes {
            // 1. Blur horizontally into the destination buffer.

            buffer.blur(&mut self.swap, &self.weights, true);

            // 2. Blur vertically back into the source buffer.

            self.swap.blur(buffer, &self.weights, false);
        }
    }
}
