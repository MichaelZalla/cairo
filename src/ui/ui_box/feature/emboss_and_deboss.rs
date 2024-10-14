use std::mem;

use crate::{buffer::Buffer2D, color, ui::ui_box::UIBox};

impl UIBox {
    pub(in crate::ui::ui_box) fn emboss_and_deboss(&self, target: &mut Buffer2D) {
        let (x, y) = self.get_pixel_coordinates();
        let (width, height) = self.get_computed_pixel_size();

        let (x1, y1, x2, y2) = (
            x as i32,
            y as i32,
            (x + width - 1) as i32,
            (y + height - 1) as i32,
        );

        let (mut top_left, mut bottom_right) = (color::WHITE, color::BLACK);

        // Emboss-deboss.

        if self.active {
            mem::swap(&mut top_left, &mut bottom_right);
        }

        // Top edge.
        target.horizontal_line_unsafe(x1 as u32, x2 as u32, y1 as u32, top_left.to_u32());

        // Bottom edge.
        target.horizontal_line_unsafe(x1 as u32, x2 as u32, y2 as u32, bottom_right.to_u32());

        // Left edge.
        target.vertical_line_unsafe(x1 as u32, y1 as u32, y2 as u32, top_left.to_u32());

        // Right edge.
        target.vertical_line_unsafe(x2 as u32, y1 as u32, y2 as u32, bottom_right.to_u32());
    }
}
