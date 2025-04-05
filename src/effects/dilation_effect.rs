use crate::{buffer::Buffer2D, color::Color, effect::Effect};

pub struct DilationEffect {
    rounds: u32,
    outline_color: Color,
    outline_color_u32: u32,
    key_color: Color,
    key_color_u32: u32,
    swap_a: Buffer2D,
    swap_b: Buffer2D,
}

impl DilationEffect {
    pub fn new(outline_color: Color, key_color: Color, rounds: Option<u32>) -> Self {
        let rounds = rounds.unwrap_or(1);

        Self {
            rounds,
            outline_color,
            outline_color_u32: outline_color.to_u32(),
            key_color,
            key_color_u32: key_color.to_u32(),
            swap_a: Default::default(),
            swap_b: Default::default(),
        }
    }
}

impl Effect for DilationEffect {
    fn apply(&mut self, buffer: &mut Buffer2D) {
        // Initialize our swap-buffers (if we are doing multiple rounds).

        // Resize our effect's swap buffers, if needed.

        self.swap_a.resize(buffer.width, buffer.height);
        self.swap_b.resize(buffer.width, buffer.height);

        // Source and destination buffers.

        buffer.dilate(&mut self.swap_a, self.key_color_u32, self.outline_color_u32);

        //

        let mut src_ref: &mut Buffer2D = &mut self.swap_a;
        let mut dest_ref: &mut Buffer2D = &mut self.swap_b;

        for _ in 1..self.rounds {
            // Perform a round of dilation.

            src_ref.dilate(dest_ref, self.key_color_u32, self.outline_color_u32);

            // Swaps the 2 mutable buffers in memory.

            std::mem::swap(&mut src_ref, &mut dest_ref);
        }

        buffer.copy(&src_ref.data);
    }
}
