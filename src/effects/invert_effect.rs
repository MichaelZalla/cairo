use crate::{buffer::Buffer2D, color::Color, effect::Effect};

pub struct InvertEffect {}

impl InvertEffect {
    pub fn new() -> Self {
        Self {}
    }
}

impl Effect for InvertEffect {
    fn apply(&self, buffer: &mut Buffer2D) {
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
