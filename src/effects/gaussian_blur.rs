use std::fmt::Debug;

use crate::{buffer::Buffer2D, effect::Effect, vec::vec3::Vec3};

#[derive(Debug, Clone)]
pub struct GaussianBlurEffect {
    strength: u8,
    passes: u8,
    weights: [f32; 5],
    swap_a: Buffer2D<Vec3>,
    swap_b: Buffer2D<Vec3>,
}

impl Default for GaussianBlurEffect {
    fn default() -> Self {
        Self {
            strength: 5,
            passes: 6,
            weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
            swap_a: Default::default(),
            swap_b: Default::default(),
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

        // Resize our effect's swap buffers, if needed.

        self.swap_a.resize(buffer.width, buffer.height);
        self.swap_b.resize(buffer.width, buffer.height);

        // Source and destination buffers.

        let mut src_ref: &mut Buffer2D<Vec3> = &mut self.swap_b;
        let mut dest_ref: &mut Buffer2D<Vec3> = &mut self.swap_a;

        for pass in 0..self.passes {
            dest_ref.clear(None);

            let src = if pass == 0 { &buffer } else { &src_ref };

            // 1. Blur horizontally into the destination buffer.

            src.blur(dest_ref, &self.weights, self.strength, true);

            // 2. Blur vertically back into the source buffer.

            src.blur(dest_ref, &self.weights, self.strength, false);

            // 3. Swap buffers.

            std::mem::swap(&mut src_ref, &mut dest_ref);
        }

        if self.passes != 1 {
            buffer.copy(dest_ref.data.as_slice());
        }
    }
}
