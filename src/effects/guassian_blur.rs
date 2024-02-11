use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul},
};

use crate::{buffer::Buffer2D, effect::Effect, vec::vec3::Vec3};

pub struct GaussianBlurEffect {
    passes: u8,
    weights: [f32; 5],
}

impl GaussianBlurEffect {
    pub fn new(passes: u8) -> Self {
        Self {
            passes,
            weights: [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
        }
    }

    fn blur<
        T: Default
            + PartialEq
            + Copy
            + Clone
            + Debug
            + Add<Output = T>
            + AddAssign
            + Mul<f32, Output = T>,
    >(
        &self,
        src: &Buffer2D<T>,
        dest: &mut Buffer2D<T>,
        horizontal: bool,
    ) {
        for y in 0..src.height {
            for x in 0..src.width {
                let mut result = *src.get(x, y) * self.weights[0];

                for i in 1..5 {
                    if horizontal {
                        if x >= i {
                            result += *src.get(x - i, y) * self.weights[i as usize];
                        }
                        if x + i < src.width {
                            result += *src.get(x + i, y) * self.weights[i as usize];
                        }
                    } else {
                        if y >= i {
                            result += *src.get(x, y - i) * self.weights[i as usize];
                        }
                        if y + i < src.height {
                            result += *src.get(x, y + i) * self.weights[i as usize];
                        }
                    }
                }

                dest.set(x, y, result);
            }
        }
    }
}

impl Effect<Vec3> for GaussianBlurEffect {
    fn apply(&self, buffer: &mut Buffer2D<Vec3>) {
        // Two-pass Guassian blur with ping-pong buffers.

        // Initialize our swap-buffers (if we are doing multiple rounds).

        let mut swap_a = Buffer2D::new(buffer.width, buffer.height, None);

        let mut swap_b: Buffer2D<Vec3> = if self.passes == 1 {
            Buffer2D::<Vec3>::new(0, 0, None)
        } else {
            Buffer2D::<Vec3>::new(buffer.width, buffer.height, None)
        };

        // Immutable source buffer, mutable destination buffer.

        let src_ref: &mut Buffer2D<Vec3> = &mut swap_b;
        let dest_ref: &mut Buffer2D<Vec3> = &mut swap_a;

        for pass in 0..self.passes {
            dest_ref.clear(None);

            // 1. Blur horizontally into the destination buffer.

            self.blur::<Vec3>(if pass == 0 { buffer } else { src_ref }, dest_ref, true);

            // 2. Blur vertically back into the source buffer.

            self.blur::<Vec3>(dest_ref, if pass == 0 { buffer } else { src_ref }, false);

            // 3. Swap buffers.

            std::mem::swap(src_ref, dest_ref);
        }

        if self.passes != 1 {
            buffer.blit(0, 0, buffer.width, buffer.height, &dest_ref.data, None);
        }
    }
}
