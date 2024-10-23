use crate::{buffer::Buffer2D, color::Color, effect::Effect};

#[derive(Default)]
pub struct InvertEffect {}

impl Effect for InvertEffect {
    fn apply(&mut self, buffer: &mut Buffer2D) {
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let color = Color::from_u32(*buffer.get(x, y));

                let inverse = Color::rgb(
                    255 - color.r as u8,
                    255 - color.g as u8,
                    255 - color.b as u8,
                );

                buffer.set(x, y, inverse.to_u32());
            }
        }
    }
}
