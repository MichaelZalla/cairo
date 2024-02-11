use crate::{
    buffer::Buffer2D,
    color::Color,
    effect::{kernel::get_coordinates, Effect},
};

pub struct DilationEffect {
    rounds: u32,
    outline_color: Color,
    outline_color_u32: u32,
    key_color: Color,
    key_color_u32: u32,
}

impl DilationEffect {
    pub fn new(outline_color: Color, key_color: Color, rounds: Option<u32>) -> Self {
        let rounds = match rounds {
            Some(value) => value,
            None => 1,
        };

        Self {
            rounds,
            outline_color,
            outline_color_u32: outline_color.to_u32(),
            key_color,
            key_color_u32: key_color.to_u32(),
        }
    }

    fn dilate(&self, src: &Buffer2D, dest: &mut Buffer2D) {
        for y in 0..src.height as i32 {
            for x in 0..src.width as i32 {
                let color = src.get(x as u32, y as u32);

                if *color != self.key_color_u32 {
                    dest.set(x as u32, y as u32, *color);

                    for (index, (n_x, n_y)) in
                        get_coordinates(x as i32, y as i32).iter().enumerate()
                    {
                        if index == 4 {
                            // Skips center coordinate (4).
                            continue;
                        }

                        // Perform bounds-checking.
                        if *n_x < 0
                            || *n_x > (src.width - 1) as i32
                            || *n_y < 0
                            || *n_y > (src.height - 1) as i32
                        {
                            continue;
                        }

                        // Perform dilation (but only outside of the drawn objects).
                        if *src.get(*n_x as u32, *n_y as u32) == self.key_color_u32 {
                            dest.set(*n_x as u32, *n_y as u32, self.outline_color_u32)
                        }
                    }
                }
            }
        }
    }
}

impl Effect for DilationEffect {
    fn apply(&self, buffer: &mut Buffer2D) {
        // Initialize our swap-buffers (if we are doing multiple rounds).

        let mut swap_a = Buffer2D::new(buffer.width, buffer.height, Some(self.key_color_u32));

        let mut swap_b: Buffer2D = if self.rounds == 1 {
            Buffer2D::new(0, 0, None)
        } else {
            Buffer2D::new(buffer.width, buffer.height, Some(self.key_color_u32))
        };

        // Immutable source buffer, mutable destination buffer.

        let src_ref: &mut Buffer2D = &mut swap_b;
        let dest_ref: &mut Buffer2D = &mut swap_a;

        for round in 0..self.rounds {
            // Perform a round of dilation.

            self.dilate(if round == 0 { buffer } else { src_ref }, dest_ref);

            if self.rounds == 1 {
                // No swapping needed.

                buffer.blit(0, 0, buffer.width, buffer.height, &dest_ref.data, None);

                return;
            }

            // Swaps the 2 mutable buffers in memory.

            std::mem::swap(src_ref, dest_ref);
        }

        buffer.blit(0, 0, buffer.width, buffer.height, &src_ref.data, None);
    }
}
