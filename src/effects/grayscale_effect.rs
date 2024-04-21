use crate::{buffer::Buffer2D, color::Color, effect::Effect, vec::vec3::Vec3};

#[derive(Default)]
pub struct GrayscaleEffect {}

impl Effect for GrayscaleEffect {
    fn apply(&self, buffer: &mut Buffer2D) {
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let color = Color::from_u32(*buffer.get(x, y));

                let mut color_vec3 = color.to_vec3() / 255.0;

                // Transform to linear space.
                color_vec3 *= color_vec3;

                // Apply channel-weighted grayscale transform.
                let weighted_average =
                    color_vec3.x * 0.2126 + color_vec3.y * 0.7152 + color_vec3.z * 0.0722;

                let mut grayscale_vec3 = Vec3 {
                    x: weighted_average,
                    y: weighted_average,
                    z: weighted_average,
                };

                // Transform back to non-linear (gamma) space.
                grayscale_vec3 = Vec3 {
                    x: grayscale_vec3.x.sqrt(),
                    y: grayscale_vec3.y.sqrt(),
                    z: grayscale_vec3.z.sqrt(),
                };

                let grayscale = Color::from_vec3(grayscale_vec3 * 255.0);

                buffer.set(x, y, grayscale.to_u32());
            }
        }
    }
}
