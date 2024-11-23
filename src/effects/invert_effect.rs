use crate::{buffer::Buffer2D, color::Color, effect::Effect};

#[derive(Default)]
pub struct InvertEffect {}

impl Effect for InvertEffect {
    fn apply(&mut self, buffer: &mut Buffer2D) {
        for color_u32 in buffer.data.iter_mut() {
            let color = Color::from_u32(*color_u32);

            let inverse = Color::rgb(
                (255.0 - color.r) as u8,
                (255.0 - color.g) as u8,
                (255.0 - color.b) as u8,
            );

            *color_u32 = inverse.to_u32();
        }
    }
}
