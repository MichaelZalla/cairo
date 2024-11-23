use crate::{buffer::Buffer2D, color::Color, effect::Effect, vec::vec3};

#[derive(Default)]
pub struct GrayscaleEffect {}

impl Effect for GrayscaleEffect {
    fn apply(&mut self, buffer: &mut Buffer2D) {
        for color_u32 in buffer.data.iter_mut() {
            let color = Color::from_u32(*color_u32);

            let mut color_vec3 = color.to_vec3() / 255.0;

            // Transform to linear space.

            color_vec3 *= color_vec3;

            // Apply channel-weighted grayscale transform.

            let weighted_average_linear =
                color_vec3.x * 0.2126 + color_vec3.y * 0.7152 + color_vec3.z * 0.0722;

            // Transform back to non-linear (gamma) space.

            let weighted_average_nonlinear = weighted_average_linear.sqrt();

            let grayscale = Color::from_vec3(vec3::ONES * weighted_average_nonlinear * 255.0);

            *color_u32 = grayscale.to_u32();
        }
    }
}
